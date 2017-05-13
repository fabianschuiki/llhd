// Copyright (c) 2017 Fabian Schuiki

//! This module implements basic blocks in a function or process body.

// use inst::InstRef;
use value::*;
use ty::*;
use unit::*;
use inst::*;
use inst::InstRef;
use std::collections::HashMap;


/// A basic block.
pub struct Block {
	id: BlockRef,
	name: Option<String>,
	insts: Vec<InstRef>,
}

impl Block {
	/// Create a new empty basic block with an optional name (aka label).
	pub fn new(name: Option<String>) -> Block {
		Block {
			id: BlockRef(ValueId::alloc()),
			name: name,
			insts: Vec::new(),
		}
	}

	/// Obtain a reference to this block.
	pub fn as_ref(&self) -> BlockRef {
		self.id
	}

	pub fn insts<'a>(&'a self, ctx: &'a UnitContext) -> InstIter<'a> {
		InstIter::new(self.insts.iter(), ctx)
	}

	pub fn append_inst(&mut self, inst: InstRef) {
		self.insts.push(inst);
	}

	pub fn remove_inst(&mut self, inst: InstRef) {
		let pos = self.insts.iter().position(|&i| i == inst).expect("basic block does not contain inst");
		self.insts.remove(pos);
	}
}

impl Value for Block {
	fn id(&self) -> ValueId {
		self.id.into()
	}

	fn ty(&self) -> Type {
		void_ty()
	}

	fn name(&self) -> Option<&str> {
		self.name.as_ref().map(|x| x as &str)
	}

	fn is_global(&self) -> bool {
		false
	}
}

declare_ref!(BlockRef, Block);



pub struct BlockPool(HashMap<BlockRef, Block>);
