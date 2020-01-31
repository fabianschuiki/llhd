// Copyright (c) 2017-2019 Fabian Schuiki

#[macro_use]
extern crate clap;
use clap::Arg;
use llhd::{assembly::parse_module, verifier::Verifier};
use std::{fs::File, io::Read, result::Result};

fn main() {
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
        .get_matches();

    let mut num_errors = 0;
    for path in matches.values_of("inputs").into_iter().flat_map(|x| x) {
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
