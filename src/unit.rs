// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code)]

use std;
use block::*;
use inst::*;
use value::Context;
use argument::*;


pub struct Entity {
	// inst_pool: InstPool,
	inst_seq: Vec<InstRef>,
}


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


pub struct BlockIter<'tf> {
	refs: std::slice::Iter<'tf, BlockRef>,
	blocks: &'tf std::collections::HashMap<BlockRef, Block>,
}

impl<'tf> BlockIter<'tf> {
	pub fn new(refs: std::slice::Iter<'tf, BlockRef>, blocks: &'tf std::collections::HashMap<BlockRef, Block>) -> BlockIter<'tf> {
		BlockIter {
			refs: refs,
			blocks: blocks,
		}
	}
}

impl<'tf> std::iter::Iterator for BlockIter<'tf> {
	type Item = &'tf Block;

	fn next(&mut self) -> Option<&'tf Block> {
		let n = self.refs.next();
		n.map(|r| self.blocks.get(r).unwrap())
	}
}
