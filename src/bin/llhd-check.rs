// Copyright (c) 2017-2020 Fabian Schuiki

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::Arg;
use llhd::{assembly::parse_module, pass::tcm::TemporalRegionGraph, verifier::Verifier};
use std::{fs::File, io::Read, result::Result};

fn main() {
    let matches = app_from_crate!()
        .about("A tool to verify the internal consistency of LLHD assembly.")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
        )
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

    // Configure the logger.
    let verbose = std::cmp::max(1, matches.occurrences_of("verbosity") as usize) - 1;
    let quiet = !matches.is_present("verbosity");
    stderrlog::new()
        .module("llhd")
        .module("llhd_check")
        .quiet(quiet)
        .verbosity(verbose)
        .init()
        .unwrap();

    let mut num_errors = 0;
    for path in matches.values_of("inputs").into_iter().flat_map(|x| x) {
        debug!("Parsing {}", path);
        let module = match parse_and_verify(path) {
            Ok(module) => module,
            Err(msg) => {
                println!("{}:", path);
                println!("{}", msg);
                num_errors += 1;
                continue;
            }
        };

        // Dump the module to stdout if requested by the user.
        if matches.is_present("dump") {
            println!("{}", module.dump());
        }

        // Dump the temporal regions if requested by the user.
        if matches.is_present("emit-trg") {
            println!("Temporal Regions:");
            for u in module.units() {
                if u.is_entity() {
                    continue;
                }
                let trg = TemporalRegionGraph::new(&u);
                println!("  {}:", u.name());
                println!("    Blocks:");
                for bb in u.blocks() {
                    println!("      - {} = {}", bb.dump(&u), trg[bb]);
                }
                for (tr, data) in trg.regions() {
                    println!("    {}:", tr);
                    if data.entry {
                        println!("      **entry**");
                    }
                    println!(
                        "      Head Blocks: {}",
                        data.head_blocks()
                            .map(|bb| bb.dump(&u).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    println!("      Head Insts:");
                    for inst in data.head_insts() {
                        println!("        - {}", inst.dump(&u));
                    }
                    println!("      Head tight: {}", data.head_tight);
                    println!(
                        "      Tail Blocks: {}",
                        data.tail_blocks()
                            .map(|bb| bb.dump(&u).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    println!("      Tail Insts:");
                    for inst in data.tail_insts() {
                        println!("        - {}", inst.dump(&u));
                    }
                    println!("      Tail tight: {}", data.tail_tight);
                }
            }
        }
    }

    std::process::exit(num_errors);
}

fn parse_and_verify(path: &str) -> Result<llhd::ir::Module, String> {
    let mut input = File::open(path).map_err(|e| format!("{}", e))?;
    let mut contents = String::new();
    input
        .read_to_string(&mut contents)
        .map_err(|e| format!("{}", e))?;
    let module = parse_module(&contents).map_err(|e| format!("{}", e))?;
    let mut verifier = Verifier::new();
    verifier.verify_module(&module);
    match verifier.finish() {
        Ok(()) => Ok(module),
        Err(errs) => Err(format!("{}", errs)),
    }
}
