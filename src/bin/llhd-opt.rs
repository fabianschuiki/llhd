// Copyright (c) 2017-2019 Fabian Schuiki

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::Arg;
use llhd::{assembly::parse_module, opt::prelude::*, verifier::Verifier};
use std::{
    fs::File,
    io::{BufWriter, Read},
    result::Result,
    sync::atomic::Ordering,
};

fn main() {
    match main_inner() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn main_inner() -> Result<(), String> {
    let matches = app_from_crate!()
        .about("Optimizes LLHD assembly.")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help(HELP_VERBOSITY.lines().next().unwrap())
                .long_help(HELP_VERBOSITY),
        )
        .arg(
            Arg::with_name("input")
                .help("LLHD file to optimize")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("File to write output to; stdout if omitted"),
        )
        .arg(
            Arg::with_name("time-passes")
                .short("t")
                .long("time")
                .help("Print execution time statistics per pass"),
        )
        .arg(
            Arg::with_name("single-threaded")
                .short("s")
                .long("--no-parallel")
                .help("Do not parallelize execution"),
        )
        .arg(
            Arg::with_name("passes")
                .short("p")
                .long("pass")
                .value_name("PASS")
                .takes_value(true)
                .multiple(true)
                .help(HELP_PASSES.lines().next().unwrap())
                .long_help(HELP_PASSES),
        )
        .get_matches();

    // Configure the logger.
    let verbose = std::cmp::max(1, matches.occurrences_of("verbosity") as usize) - 1;
    let quiet = !matches.is_present("verbosity");
    stderrlog::new()
        .module("llhd")
        .module("llhd_opt")
        .quiet(quiet)
        .verbosity(verbose)
        .init()
        .unwrap();

    // Configure rayon to be single-threaded if requested.
    if matches.is_present("single-threaded") {
        info!("Limiting to one rayon worker thread");
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    // Prepare the time tracking.
    let mut times = vec![];
    let tinit = time::precise_time_ns();

    // Read the input.
    let t0 = time::precise_time_ns();
    let mut module = {
        let path = matches.value_of("input").unwrap();
        let mut input = File::open(path).map_err(|e| format!("{}", e))?;
        let mut contents = String::new();
        input
            .read_to_string(&mut contents)
            .map_err(|e| format!("{}", e))?;
        let module = parse_module(&contents).map_err(|e| format!("{}", e))?;
        let mut verifier = Verifier::new();
        verifier.verify_module(&module);
        verifier.finish().map_err(|errs| format!("{}", errs))?;
        module
    };
    let t1 = time::precise_time_ns();
    times.push(("parse".to_owned(), t1 - t0));

    // Determine the optimization passes to be run.
    let passes: Vec<_> = if let Some(passes) = matches.values_of("passes") {
        passes.collect()
    } else {
        vec![
            "cf",
            "vtpp",
            "dce",
            "gcse",
            "licm",
            "tcm",
            "licm",
            "tcm",
            "gcse",
            "tcm",
            "cf",
            "licm",
            "gcse",
            "insim",
            "dce",
            "cfs",
            "insim",
            "dce",
            "proclower",
        ]
    };

    // Apply optimization passes.
    debug!("Running {:?}", passes);
    let ctx = PassContext;
    for &pass in &passes {
        trace!("Running pass {}", pass);
        let t0 = time::precise_time_ns();
        let _changes = match pass {
            "cf" => llhd::pass::ConstFolding::run_on_module(&ctx, &mut module),
            "cfs" => llhd::pass::ControlFlowSimplification::run_on_module(&ctx, &mut module),
            "dce" => llhd::pass::DeadCodeElim::run_on_module(&ctx, &mut module),
            "gcse" => llhd::pass::GlobalCommonSubexprElim::run_on_module(&ctx, &mut module),
            "insim" => llhd::pass::InstSimplification::run_on_module(&ctx, &mut module),
            "licm" => llhd::pass::LoopIndepCodeMotion::run_on_module(&ctx, &mut module),
            "tcm" => llhd::pass::TemporalCodeMotion::run_on_module(&ctx, &mut module),
            "vtpp" => llhd::pass::VarToPhiPromotion::run_on_module(&ctx, &mut module),
            "proclower" => llhd::pass::ProcessLowering::run_on_module(&ctx, &mut module),
            "deseq" => llhd::pass::Desequentialization::run_on_module(&ctx, &mut module),
            "verify" => {
                let mut verifier = Verifier::new();
                verifier.verify_module(&module);
                match verifier.finish() {
                    Ok(_) => (),
                    Err(errs) => error!("Verification failed:\n{}", errs),
                }
                false // no changes
            }
            _ => {
                error!("Unknown pass `{}`", pass);
                continue;
            }
        };
        let t1 = time::precise_time_ns();
        times.push((pass.to_owned(), t1 - t0));
    }

    // Verify modified module.
    let t0 = time::precise_time_ns();
    let mut verifier = Verifier::new();
    verifier.verify_module(&module);
    verifier
        .finish()
        .map_err(|errs| format!("Verification failed after optimization:\n{}", errs))?;
    let t1 = time::precise_time_ns();
    times.push(("verify".to_owned(), t1 - t0));

    // Write the output.
    let t0 = time::precise_time_ns();
    if let Some(path) = matches.value_of("output") {
        let output = File::create(path).map_err(|e| format!("{}", e))?;
        let output = BufWriter::with_capacity(1 << 20, output);
        llhd::assembly::write_module(output, &module);
    } else {
        llhd::assembly::write_module(std::io::stdout().lock(), &module);
    }
    let t1 = time::precise_time_ns();
    times.push(("output".to_owned(), t1 - t0));

    // Final time stat.
    let tfinal = time::precise_time_ns();
    times.push(("total".to_owned(), tfinal - tinit));

    // Print execution time statistics if requested by the user.
    if matches.is_present("time-passes") {
        eprintln!("Execution Time Statistics:");
        for (mut name, ns) in times {
            name.push(':');
            eprintln!("  {:10}  {:8.3} ms", name, ns as f64 * 1.0e-6);
        }
        eprintln!("");
        eprintln!("Structure Statistics:");
        eprintln!(
            "  Dominator Tree Construction: {:8.3} ms",
            llhd::pass::gcse::DOMINATOR_TREE_TIME.load(Ordering::SeqCst) as f64 * 1.0e-6
        );
    }

    // Dump some threading statistics.
    info!("Used {} rayon worker threads", rayon::current_num_threads());

    Ok(())
}

static HELP_VERBOSITY: &str = "Increase message verbosity

This option can be specified multiple times to increase the level of verbosity \
in the output:

-v      Only print errors
-vv     Also print warnings
-vvv    Also print info messages
-vvvv   Also print debug messages
-vvvvv  Also print detailed tracing messages
";

static HELP_PASSES: &str = "Exact order of passes to run

This option specifies the exact order of passes to be executed. The admissible \
passes are as follows:

cf          Constant folding
cfs         Control Flow Simplification
dce         Dead Code Elimination
gcse        Global Common Subexpression Elimination
insim       Instruction Simplification
licm        Loop-Invariant Code Motion
tcm         Temporal Code Motion
vtpp        Var-to-Phi Promotion
proclower   Process Lowering
deseq       Desequentialization
";
