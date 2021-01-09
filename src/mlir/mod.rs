// Copyright (c) 2017-2021 Fabian Schuiki

//! Facilities to emit a module as CIRCT IR.

use crate::ir::Module;

mod writer;

/// Emit CIRCT IR for a module.
pub fn write_module(sink: impl std::io::Write, module: &Module) {
    writer::Writer::new(sink).write_module(module).unwrap();
}

/// Emit CIRCT IR for a module as string.
pub fn write_module_string(module: &Module) -> String {
    let mut asm = vec![];
    write_module(&mut asm, &module);
    String::from_utf8(asm).expect("writer should emit proper utf8")
}
