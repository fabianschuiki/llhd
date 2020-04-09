// Copyright (c) 2017-2020 Fabian Schuiki

//! Representation of LLHD functions, processes, and entitites.
//!
//! This module implements the intermediate representation around which the rest
//! of the framework is built.

use crate::{impl_table_key, ty::Type};

mod cfg;
mod dfg;
mod entity;
mod function;
mod inst;
mod layout;
mod module;
pub mod prelude;
mod process;
mod sig;
mod unit;

pub use self::cfg::*;
pub use self::dfg::*;
pub use self::entity::*;
pub use self::function::*;
pub use self::inst::*;
pub use self::layout::*;
pub use self::module::*;
pub use self::process::*;
pub use self::sig::*;
pub use self::unit::*;

/// The position where new instructions will be inserted into a `Function` or
/// `Process`.
#[derive(Clone, Copy)]
enum FunctionInsertPos {
    None,
    Append(Block),
    Prepend(Block),
    After(Inst),
    Before(Inst),
}

impl FunctionInsertPos {
    /// Insert an instruction and update the insertion postition.
    fn add_inst(&mut self, inst: Inst, layout: &mut FunctionLayout) {
        use FunctionInsertPos::*;
        match *self {
            None => panic!("no block selected to insert instruction"),
            Append(bb) => layout.append_inst(inst, bb),
            Prepend(bb) => {
                layout.prepend_inst(inst, bb);
                *self = After(inst);
            }
            After(other) => {
                layout.insert_inst_after(inst, other);
                *self = After(inst);
            }
            Before(other) => layout.insert_inst_before(inst, other),
        }
    }

    /// Update the insertion position in response to removing an instruction.
    fn remove_inst(&mut self, inst: Inst, layout: &FunctionLayout) {
        use FunctionInsertPos::*;
        match *self {
            // If we inserted after i, now insert before i's successor, or if i
            // was the last inst in the block, at the end of the block.
            After(i) if i == inst => {
                *self = layout
                    .next_inst(i)
                    .map(Before)
                    .unwrap_or(Append(layout.inst_block(i).unwrap()))
            }
            // If we inserted before i, now insert after i's predecessor, or if
            // i was the first inst in the block, at the beginning of the block.
            Before(i) if i == inst => {
                *self = layout
                    .prev_inst(i)
                    .map(After)
                    .unwrap_or(Prepend(layout.inst_block(i).unwrap()))
            }
            // Everything else we just keep as is.
            _ => (),
        }
    }
}

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
    pub fn dump(self, dfg: &DataFlowGraph) -> ValueDumper {
        ValueDumper(self, dfg)
    }
}

/// Temporary object to dump a `Value` in human-readable form for debugging.
pub struct ValueDumper<'a>(Value, &'a DataFlowGraph);

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
    pub fn dump(self, cfg: &ControlFlowGraph) -> BlockDumper {
        BlockDumper(self, cfg)
    }
}

/// Temporary object to dump a `Block` in human-readable form for debugging.
pub struct BlockDumper<'a>(Block, &'a ControlFlowGraph);

impl std::fmt::Display for BlockDumper<'_> {
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
