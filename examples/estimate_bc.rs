// Copyright (c) 2017-2019 Fabian Schuiki

#[macro_use]
extern crate clap;

use clap::Arg;
use llhd::{
    assembly::parse_module,
    ir::{prelude::*, ModUnitData},
    verifier::Verifier,
};
use std::{fs::File, io::Read};

fn main() {
    let matches = app_from_crate!()
        .arg(Arg::with_name("input").required(true))
        .get_matches();

    let module = {
        let path = matches.value_of("input").unwrap();
        let mut input = File::open(path).unwrap();
        let mut contents = String::new();
        input.read_to_string(&mut contents).unwrap();
        let module = parse_module(&contents).unwrap();
        let mut verifier = Verifier::new();
        verifier.verify_module(&module);
        verifier.finish().unwrap();
        module
    };

    let mut num_bytes = 0;
    for unit in module.units() {
        eprintln!("Estimating {}", module.unit_name(unit));
        let mut insts = vec![];
        let mut blocks = vec![];

        match module[unit] {
            ModUnitData::Entity(ref e) => insts.extend(e.inst_layout().insts()),
            ModUnitData::Process(ref p) => {
                for b in p.func_layout().blocks() {
                    blocks.push(b);
                    insts.extend(p.func_layout().insts(b));
                }
            }
            ModUnitData::Function(ref f) => {
                for b in f.func_layout().blocks() {
                    blocks.push(b);
                    insts.extend(f.func_layout().insts(b));
                }
            }
            _ => (),
        }

        num_bytes += module.unit_name(unit).to_string().len();
        let sig = module.unit_sig(unit);
        num_bytes += sig.args().count() * 8;

        for &_ in &blocks {
            num_bytes += 16; // name estimate
            num_bytes += 2; // identifier
            num_bytes += 4; // block size
        }

        let dfg = module[unit].get_dfg().unwrap();
        for &inst in &insts {
            num_bytes += 2; // opcode
            num_bytes += 2; // type
            num_bytes += dfg[inst].args().len() * 2;
            num_bytes += dfg[inst].blocks().len() * 2;
            num_bytes += dfg[inst].imms().len() * 4; // average estimate
        }
    }

    println!("{} bytes", num_bytes);
}
