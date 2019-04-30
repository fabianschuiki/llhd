// Copyright (c) 2017 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

use crate::{konst::ConstTime, ty::Type};

pub(crate) mod ast;
pub(crate) mod grammar;
mod reader;
mod writer;

pub use self::{reader::parse_str, writer::Writer};
use crate::{Module, Visitor};

/// Emit assembly for a module.
///
/// # Example
/// ```
/// # use llhd::{Module, Entity, entity_ty, assembly::write};
/// // Create a module.
/// let mut module = Module::new();
/// module.add_entity(Entity::new("foo", entity_ty(vec![], vec![])));
///
/// // Write to a vector of bytes and convert into a string.
/// let mut asm = vec![];
/// write(&mut asm, &module);
/// let asm = String::from_utf8(asm).unwrap();
///
/// assert_eq!(asm, "entity @foo () () {\n}\n");
/// ```
pub fn write(sink: &mut impl std::io::Write, module: &Module) {
    Writer::new(sink).visit_module(module);
}

/// Emit assembly for a module as string.
///
/// # Example
/// ```
/// # use llhd::{Module, Entity, entity_ty, assembly::write_string};
/// // Create a module.
/// let mut module = Module::new();
/// module.add_entity(Entity::new("foo", entity_ty(vec![], vec![])));
///
/// assert_eq!(write_string(&module), "entity @foo () () {\n}\n");
/// ```
pub fn write_string(module: &Module) -> String {
    let mut asm = vec![];
    write(&mut asm, &module);
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
