// Copyright (c) 2017-2020 Fabian Schuiki

#[macro_use]
extern crate clap;
use clap::Arg;
use llhd::ir::Module;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::Path,
};

mod liberty;

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    // Parse the command line arguments.
    let matches = app_from_crate!()
        .arg(Arg::with_name("input").required(true))
        .arg(Arg::with_name("output").required(true))
        .arg(
            Arg::with_name("dump")
                .long("--dump")
                .help("Dump the intermediate LLHD"),
        )
        .get_matches();

    let input_path = Path::new(matches.value_of("input").unwrap());
    let output_path = Path::new(matches.value_of("output").unwrap());

    // Detect input and output formats.
    let (input_format, input_compression) = detect_format(input_path)?;
    let (output_format, output_compression) = detect_format(output_path)?;

    if input_compression != Compression::None {
        eprintln!("input compression {} not supported", input_compression);
        return Err(());
    }
    if output_compression != Compression::None {
        eprintln!("output compression {} not supported", output_compression);
        return Err(());
    }

    // Process the input.
    let module = match input_format {
        Format::Assembly => {
            let mut input = File::open(input_path).map_err(|e| eprintln!("{}", e))?;
            let mut contents = String::new();
            input
                .read_to_string(&mut contents)
                .map_err(|e| eprintln!("{}", e))?;
            llhd::assembly::parse_module(&contents).map_err(|e| eprintln!("{}", e))?
        }
        Format::Liberty => {
            let input = File::open(input_path).map_err(|e| eprintln!("{}", e))?;
            let input = BufReader::with_capacity(1 << 16, input);
            let mut lexer = liberty::Lexer::new(input.bytes());
            let mut module = Module::new();
            let mut visitor = liberty::RootVisitor::new(&mut module);
            liberty::parse(&mut lexer, &mut visitor);
            module
        }
        f => panic!("{} inputs not supported", f),
    };

    // Dump the IR if requested.
    if matches.is_present("dump") {
        eprintln!("{}", module.dump());
    }

    // Generate the output.
    match output_format {
        Format::Assembly => {
            let output = File::create(output_path).map_err(|e| eprintln!("{}", e))?;
            let output = BufWriter::with_capacity(1 << 16, output);
            llhd::assembly::write_module(output, &module);
        }
        f => panic!("{} outputs not supported", f),
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Format {
    Assembly,
    Bitcode,
    Verilog,
    Vhdl,
    Firrtl,
    Edif,
    Liberty,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Compression {
    None,
    Gzip,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Format::Assembly => write!(f, "LLHD assembly"),
            Format::Bitcode => write!(f, "LLHD bitcode"),
            Format::Verilog => write!(f, "Verilog netlist"),
            Format::Vhdl => write!(f, "VHDL netlist"),
            Format::Firrtl => write!(f, "FIRRTL"),
            Format::Edif => write!(f, "EDIF netlist"),
            Format::Liberty => write!(f, "LIB file"),
        }
    }
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Compression::None => write!(f, "none"),
            Compression::Gzip => write!(f, "gzip"),
        }
    }
}

fn detect_format(path: &Path) -> Result<(Format, Compression)> {
    // Detect compression.
    let (comp, path) = match path.extension().and_then(|e| e.to_str()) {
        Some("gz") | Some("gzip") => (Compression::Gzip, path),
        _ => (Compression::None, path),
    };

    // Detect format.
    let fmt = match path.extension().and_then(|e| e.to_str()) {
        Some("llhd") => Format::Assembly,
        Some("bc") => Format::Bitcode,
        Some("v") => Format::Verilog,
        Some("vhdl") | Some("vhd") => Format::Vhdl,
        Some("fir") => Format::Firrtl,
        Some("edif") => Format::Edif,
        Some("lib") => Format::Liberty,
        _ => {
            eprintln!("unknown file format {:?}", path);
            return Err(());
        }
    };

    Ok((fmt, comp))
}
