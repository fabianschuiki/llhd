// Copyright (c) 2017-2021 Fabian Schuiki

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgMatches};
use llhd::{assembly::parse_module_unchecked, verifier::Verifier};

fn main() {
    // Configure the logger.
    pretty_env_logger::init_custom_env("LLHD_LOG");

    // Parse the command line arguments.
    let matches = app_from_crate!()
        .about("A tool to verify the internal consistency of LLHD assembly.")
        .arg(
            Arg::with_name("inputs")
                .multiple(true)
                .help("LLHD files to verify"),
        )
        .arg(
            Arg::with_name("dump")
                .short("d")
                .long("dump")
                .help("Dump parsed LLHD to stdout"),
        )
        .arg(
            Arg::with_name("emit-trg")
                .long("emit-trg")
                .help("Analyze and emit the temporal regions"),
        )
        .get_matches();

    let mut num_errors = 0;
    for path in matches.values_of("inputs").into_iter().flat_map(|x| x) {
        debug!("Parsing {}", path);
        match process_input(path, &matches) {
            Ok(()) => (),
            Err(e) => {
                println!("{}: {:#}", path, e);
                num_errors += 1;
            }
        }
    }

    std::process::exit(num_errors);
}

fn process_input(path: &str, matches: &ArgMatches) -> Result<()> {
    // Parse the input.
    let input = std::fs::read_to_string(path).context("Reading file failed")?;
    let module = parse_module_unchecked(&input)
        .map_err(|msg| anyhow!(msg))
        .context("Parsing failed")?;

    // Dump the module to stdout if requested by the user.
    if matches.is_present("dump") {
        println!("{}:", path);
        println!("{}", module.dump());
    }

    // Verify the module.
    let mut verifier = Verifier::new();
    verifier.verify_module(&module);
    verifier
        .finish()
        .map_err(|errs| anyhow!("Verification failed:\n{}", errs))?;

    // Dump the temporal regions if requested by the user.
    if matches.is_present("emit-trg") {
        println!("Temporal Regions:");
        for u in module.units() {
            if u.is_entity() {
                continue;
            }
            let trg = u.trg();
            println!("  {}:", u.name());
            println!("    Blocks:");
            for bb in u.blocks() {
                println!("      - {} = {}", bb.dump(&u), trg[bb]);
            }
            for tr in trg.regions() {
                println!("    {}:", tr.id);
                if tr.entry {
                    println!("      **entry**");
                }
                println!(
                    "      Head Blocks: {}",
                    tr.head_blocks()
                        .map(|bb| bb.dump(&u).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("      Head Insts:");
                for inst in tr.head_insts() {
                    println!("        - {}", inst.dump(&u));
                }
                println!("      Head tight: {}", tr.head_tight);
                println!(
                    "      Tail Blocks: {}",
                    tr.tail_blocks()
                        .map(|bb| bb.dump(&u).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("      Tail Insts:");
                for inst in tr.tail_insts() {
                    println!("        - {}", inst.dump(&u));
                }
                println!("      Tail tight: {}", tr.tail_tight);
            }
        }
    }

    Ok(())
}
