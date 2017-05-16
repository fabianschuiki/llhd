// Copyright (c) 2017 Fabian Schuiki

use block::*;
use inst::*;
use value::{Context, ArgumentRef, BlockRef, InstRef};
use argument::*;


/// A context wrapping a unit.
pub trait UnitContext : Context + AsUnitContext {
	/// Resolve a `InstRef` to an actual `&Inst` reference.
	fn inst(&self, inst: InstRef) -> &Inst;
	/// Resolve a `ArgumentRef` to an actual `&Argument` reference.
	fn argument(&self, argument: ArgumentRef) -> &Argument;
}

pub trait AsUnitContext {
	fn as_unit_context(&self) -> &UnitContext;
}

impl<T: UnitContext> AsUnitContext for T {
	fn as_unit_context(&self) -> &UnitContext { self }
}


/// A context wrapping a unit that uses basic blocks to group a sequence of
/// instructions.
pub trait SequentialContext: UnitContext {
	/// Resolve a `BlockRef` to an actual `&Block` reference.
	fn block(&self, block: BlockRef) -> &Block;
}
