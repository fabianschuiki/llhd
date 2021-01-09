// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of LLHD functions, processes, and entitites.
//!
//! This module implements the intermediate representation around which the rest
//! of the framework is built.
#![deny(missing_docs)]

use crate::{impl_table_key, ty::Type};

mod cfg;
mod dfg;
mod inst;
mod layout;
mod module;
pub mod prelude;
mod sig;
mod unit;

use self::cfg::*;
use self::dfg::*;
pub use self::inst::*;
use self::layout::*;
pub use self::module::*;
pub use self::sig::*;
pub use self::unit::*;

impl_table_key! {
    /// An instruction.
    struct Inst(u32) as "i";

    /// A value.
    struct Value(u32) as "v";

    /// A basic block.
    struct Block(u32) as "bb";

    /// An argument of a `Function`, `Process`, or `Entity`.
    struct Arg(u32) as "arg";

    /// An external `Function`, `Process` or `Entity`.
    struct ExtUnit(u32) as "ext";
}

impl Value {
    /// A placeholder for invalid values.
    ///
    /// This is used for unused instruction arguments.
    pub(crate) fn invalid() -> Self {
        Value(std::u32::MAX)
    }

    /// Check if this is a placeholder for invalid values.
    pub fn is_invalid(&self) -> bool {
        self.0 == std::u32::MAX
    }
}

impl Block {
    /// A placeholder for invalid blocks.
    ///
    /// This is used for unused instruction arguments.
    pub(crate) fn invalid() -> Self {
        Block(std::u32::MAX)
    }

    /// Check if this is a placeholder for invalid blocks.
    pub fn is_invalid(&self) -> bool {
        self.0 == std::u32::MAX
    }
}

/// Internal table storage for values.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ValueData {
    /// The invalid value placeholder.
    Invalid,
    /// The value is the result of an instruction.
    Inst { ty: Type, inst: Inst },
    /// The value is an argument of the `Function`, `Process`, or `Entity`.
    Arg { ty: Type, arg: Arg },
    /// The value is a placeholder. Used during PHI node construction.
    Placeholder { ty: Type },
}

impl ValueData {
    /// Check if the value is a placeholder.
    pub fn is_placeholder(&self) -> bool {
        match self {
            ValueData::Placeholder { .. } => true,
            _ => false,
        }
    }
}

impl Default for ValueData {
    fn default() -> ValueData {
        ValueData::Invalid
    }
}

/// Internal table storage for blocks.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BlockData {
    /// The name of the block.
    pub name: Option<String>,
}

/// Another unit referenced within a `Function`, `Process`, or `Entity`.
///
/// The linker will hook up external units to the actual counterparts as
/// appropriate.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtUnitData {
    /// The name of the referenced unit.
    pub name: UnitName,
    /// The signature of the referenced unit.
    pub sig: Signature,
}

impl Default for ExtUnitData {
    fn default() -> ExtUnitData {
        ExtUnitData {
            name: UnitName::Anonymous(0),
            sig: Signature::default(),
        }
    }
}

/// Any one of the table keys in this module.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyObject {
    Inst(Inst),
    Value(Value),
    Block(Block),
    Arg(Arg),
}

impl From<Inst> for AnyObject {
    fn from(x: Inst) -> Self {
        AnyObject::Inst(x)
    }
}

impl From<Value> for AnyObject {
    fn from(x: Value) -> Self {
        AnyObject::Value(x)
    }
}

impl From<Block> for AnyObject {
    fn from(x: Block) -> Self {
        AnyObject::Block(x)
    }
}

impl From<Arg> for AnyObject {
    fn from(x: Arg) -> Self {
        AnyObject::Arg(x)
    }
}

impl std::fmt::Display for AnyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnyObject::Inst(x) => write!(f, "{}", x),
            AnyObject::Value(x) => write!(f, "{}", x),
            AnyObject::Block(x) => write!(f, "{}", x),
            AnyObject::Arg(x) => write!(f, "{}", x),
        }
    }
}

impl Value {
    /// Dump the value in human-readable form.
    pub fn dump<'a>(self, unit: &Unit<'a>) -> ValueDumper<'a> {
        ValueDumper(self, *unit)
    }
}

/// Temporary object to dump a `Value` in human-readable form for debugging.
pub struct ValueDumper<'a>(Value, Unit<'a>);

impl std::fmt::Display for ValueDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.is_invalid() {
            write!(f, "%<invalid>")
        } else if let Some(name) = self.1.get_name(self.0) {
            write!(f, "%{}", name)
        } else if let Some(index) = self.1.get_anonymous_hint(self.0) {
            write!(f, "%{}", index)
        } else {
            write!(f, "%{}", self.0)
        }
    }
}

impl Block {
    /// Dump the basic block in human-readable form.
    pub fn dump<'a>(self, unit: &Unit<'a>) -> BlockDumper<'a> {
        BlockDumper(self, *unit)
    }
}

/// Temporary object to dump a `Block` in human-readable form for debugging.
pub struct BlockDumper<'a>(Block, Unit<'a>);

impl std::fmt::Display for BlockDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.is_invalid() {
            write!(f, "<invalid>")
        } else if let Some(name) = self.1.get_block_name(self.0) {
            write!(f, "{}", name)
        } else if let Some(index) = self.1.get_anonymous_block_hint(self.0) {
            write!(f, "{}", index)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
