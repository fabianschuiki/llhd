// Copyright (c) 2017-2019 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

use crate::{ir::Module, konst::ConstTime, ty::Type};

pub(crate) mod ast;
pub(crate) mod grammar;
mod writer;

/// Emit assembly for a module.
pub fn write_module(sink: impl std::io::Write, module: &Module) {
    writer::Writer::new(sink).write_module(module).unwrap();
}

/// Emit assembly for a module as string.
pub fn write_module_string(module: &Module) -> String {
    let mut asm = vec![];
    write_module(&mut asm, &module);
    String::from_utf8(asm).expect("writer should emit proper utf8")
}

/// Parse a type.
///
/// Parses the `input` string into a type.
pub fn parse_type(input: impl AsRef<str>) -> Result<Type, String> {
    grammar::TypeParser::new()
        .parse(input.as_ref())
        .map_err(|e| format!("{}", e))
}

/// Parse a time.
///
/// Parses the `input` string into a time constant.
pub fn parse_time(input: impl AsRef<str>) -> Result<ConstTime, String> {
    grammar::ConstTimeParser::new()
        .parse(input.as_ref())
        .map_err(|e| format!("{}", e))
}

/// Parse a module.
///
/// Parses the `input` string into a module.
pub fn parse_module(input: impl AsRef<str>) -> Result<crate::ir::Module, String> {
    grammar::ModuleParser::new()
        .parse(input.as_ref())
        .map_err(|e| format!("{}", e))
}
