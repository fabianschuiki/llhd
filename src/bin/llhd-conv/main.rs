// Copyright (c) 2017-2021 Fabian Schuiki

//! A tool to convert between LLHD and other formats

#![deny(missing_docs)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use anyhow::{anyhow, bail, Context, Result};
use clap::Arg;
use llhd::ir::Module;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};

mod liberty;
pub mod verilog;

fn main() -> Result<()> {
    // Configure the logger.
    pretty_env_logger::init_custom_env("LLHD_LOG");

    // Parse the command line arguments.
    let matches = app_from_crate!()
        .about("A tool to convert between LLHD and various other formats.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .takes_value(true)
                .help("File to read input from; stdin if omitted"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("File to write output to; stdout if omitted"),
        )
        .arg(
            Arg::with_name("input-format")
                .long("input-format")
                .takes_value(true)
                .help("Format of the input; auto-detected if omitted"),
        )
        .arg(
            Arg::with_name("output-format")
                .long("output-format")
                .takes_value(true)
                .help("Format of the output; auto-detected if omitted"),
        )
        .arg(
            Arg::with_name("dump")
                .long("--dump")
                .help("Dump the intermediate LLHD"),
        )
        .get_matches();

    // Setup the input reader.
    let input_path = matches.value_of("input").map(Path::new);
    let input_stream: Box<dyn Read> = match input_path {
        Some(path) => {
            debug!("Reading from `{}`", path.display());
            Box::new(
                File::open(path)
                    .with_context(|| format!("Failed to open input `{}`", path.display()))?,
            )
        }
        None => {
            debug!("Reading from stdin");
            Box::new(std::io::stdin())
        }
    };
    let input_name = match input_path {
        Some(path) => format!("`{}`", path.display()),
        None => format!("stdin"),
    };

    // Setup the output writer.
    let output_path = matches.value_of("output").map(Path::new);
    let output_stream: Box<dyn Write> = match output_path {
        Some(path) => {
            debug!("Writing to `{}`", path.display());
            Box::new(
                File::create(path)
                    .with_context(|| format!("Failed to create output `{}`", path.display()))?,
            )
        }
        None => {
            debug!("Writing to stdout");
            Box::new(std::io::stdout())
        }
    };
    let output_name = match output_path {
        Some(path) => format!("`{}`", path.display()),
        None => format!("stdout"),
    };

    // Detect the input format.
    let input_format = match matches.value_of("input-format") {
        Some(fmt) => {
            Format::from_str(fmt).map_err(|_| anyhow!("Unknown input format `{}`", fmt))?
        }
        None => match input_path {
            Some(path) => match path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(|f| Format::from_str(f).ok())
            {
                Some(f) => f,
                None => bail!(
                    "Failed to detect input format from path `{}`",
                    path.display()
                ),
            },
            None => bail!("`--input-format` must be specified when reading from stdin"),
        },
    };
    debug!("Input format `{}`", input_format);

    // Detect the output format.
    let output_format = match matches.value_of("output-format") {
        Some(fmt) => {
            Format::from_str(fmt).map_err(|_| anyhow!("Unknown output format `{}`", fmt))?
        }
        None => match output_path {
            Some(path) => match path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(|f| Format::from_str(f).ok())
            {
                Some(f) => f,
                None => bail!(
                    "Failed to detect output format from path `{}`",
                    path.display(),
                ),
            },
            None => bail!("`--output-format` must be specified when writing to stdout"),
        },
    };
    debug!("Output format `{}`", output_format);

    // Process the input.
    let module = read_input(
        &mut BufReader::with_capacity(1 << 20, input_stream),
        input_format,
    )
    .with_context(|| format!("Failed to read input from {}", input_name))?;

    // Dump the IR if requested.
    if matches.is_present("dump") {
        eprintln!("{}", module.dump());
    }

    // Generate the output.
    write_output(
        &module,
        &mut BufWriter::with_capacity(1 << 20, output_stream),
        output_format,
    )
    .with_context(|| format!("Failed to write output to {}", output_name))?;

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum Format {
    Assembly,
    Bitcode,
    Verilog,
    Vhdl,
    Firrtl,
    Edif,
    Liberty,
    Mlir,
}

impl FromStr for Format {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Format, ()> {
        match s {
            "llhd" => Ok(Format::Assembly),
            "bc" => Ok(Format::Bitcode),
            "v" => Ok(Format::Verilog),
            "vhdl" | "vhd" => Ok(Format::Vhdl),
            "fir" => Ok(Format::Firrtl),
            "edif" => Ok(Format::Edif),
            "lib" => Ok(Format::Liberty),
            "mlir" => Ok(Format::Mlir),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Format::Assembly => write!(f, "LLHD assembly"),
            Format::Bitcode => write!(f, "LLHD bitcode"),
            Format::Verilog => write!(f, "Verilog"),
            Format::Vhdl => write!(f, "VHDL"),
            Format::Firrtl => write!(f, "FIRRTL"),
            Format::Edif => write!(f, "EDIF netlist"),
            Format::Liberty => write!(f, "LIB file"),
            Format::Mlir => write!(f, "MLIR assembly"),
        }
    }
}

fn read_input(input: &mut impl Read, format: Format) -> Result<llhd::ir::Module> {
    match format {
        Format::Assembly => {
            let mut contents = String::new();
            input.read_to_string(&mut contents)?;
            Ok(llhd::assembly::parse_module(&contents).map_err(|e| anyhow!("{}", e))?)
        }
        Format::Liberty => {
            let mut lexer = liberty::Lexer::new(input.bytes());
            let mut module = Module::new();
            let mut visitor = liberty::RootVisitor::new(&mut module);
            liberty::parse(&mut lexer, &mut visitor);
            Ok(module)
        }
        f => bail!("{} inputs not supported", f),
    }
}

fn write_output(module: &llhd::ir::Module, output: &mut impl Write, format: Format) -> Result<()> {
    match format {
        Format::Assembly => {
            llhd::assembly::write_module(output, module);
            Ok(())
        }
        Format::Verilog => {
            crate::verilog::write(output, module)?;
            Ok(())
        }
        Format::Mlir => {
            llhd::mlir::write_module(output, module);
            Ok(())
        }
        f => bail!("{} outputs not supported", f),
    }
}
