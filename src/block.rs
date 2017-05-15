// Copyright (c) 2017 Fabian Schuiki

//! This module implements basic blocks in a function or process body.

// use inst::InstRef;
use value::*;
use ty::*;
use unit::*;
use inst::*;


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

	/// Obtain an iterator over the instructions in this block.
	pub fn insts<'a>(&'a self, ctx: &'a UnitContext) -> InstIter<'a> {
		InstIter::new(self.insts.iter(), ctx)
	}

	/// Obtain an iterator over references to the instructions in this block.
	pub fn inst_refs(&self) -> std::slice::Iter<InstRef> {
		self.insts.iter()
	}

	/// Insert an instruction into this block as dictated by the requested
	/// position. `Begin` and `End` are treated as synonyms to `BlockBegin` and
	/// `BlockEnd`. Panics if the referred instruction is not part of this
	/// block.
	pub fn insert_inst(&mut self, inst: InstRef, pos: InstPosition) {
		let index = match pos {
			InstPosition::Begin => 0,
			InstPosition::End => self.insts.len(),
			InstPosition::Before(i) => self.inst_pos(i),
			InstPosition::After(i) => self.inst_pos(i) + 1,
			InstPosition::BlockBegin(b) => {
				assert_eq!(self.id, b);
				0
			}
			InstPosition::BlockEnd(b) => {
				assert_eq!(self.id, b);
				self.insts.len()
			}
		};
		self.insts.insert(index, inst)
	}

	/// Detach an instruction from this block. Panics if the instruction is not
	/// part of this block.
	pub fn detach_inst(&mut self, inst: InstRef) {
		let pos = self.inst_pos(inst);
		self.insts.remove(pos);
	}

	/// Determine the index at which a certain position is located. Panics if
	/// the instruction is not part of the block.
	fn inst_pos(&self, inst: InstRef) -> usize {
		self.insts.iter().position(|&i| i == inst).expect("basic block does not contain inst")
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


/// A relative position of a block. Used to insert or move a block to a position
/// relative to the surrounding unit or another block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BlockPosition {
	/// The very first position in the function/process.
	Begin,
	/// The very last position in the function/process.
	End,
	/// The position just before another block.
	Before(BlockRef),
	/// The position just after another block.
	After(BlockRef),
}
