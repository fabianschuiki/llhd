// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code)]

use std::collections::HashMap;
use ty::*;
use block::*;
use inst::*;
use value::*;


pub struct Entity {
	inst_pool: InstPool,
	inst_seq: Vec<InstRef>,
}

pub struct Process {
	inst_pool: InstPool,
	block_pool: BlockPool,
	block_seq: Vec<BlockRef>,
}

pub struct Function {
	name: String,
	ty: Type,
	args: Vec<Argument>,
	blocks: HashMap<BlockRef, Block>,
	block_seq: Vec<BlockRef>,
	block_of_inst: HashMap<InstRef, BlockRef>,
	insts: HashMap<InstRef, Inst>,
	// inst_pool: InstPool,
	// block_pool: BlockPool,
	// block_seq: Vec<BlockRef>,
}

impl Function {
	/// Create a new function with the given name and type signature. Anonymous
	/// arguments are created for each argument in the type signature. Use the
	/// `arg_mut` function to get a hold of these arguments and assign names and
	/// additional data to them.
	pub fn new(name: String, ty: Type) -> Function {
		let args = {
			let (arg_tys, _) = ty.as_func();
			arg_tys.iter().map(|t| Argument::new(t.clone())).collect()
		};
		Function {
			ty: ty,
			name: name,
			args: args,
			blocks: HashMap::new(),
			block_seq: Vec::new(),
			block_of_inst: HashMap::new(),
			insts: HashMap::new(),
			// inst_pool: InstPool(HashMap::new()),
			// block_pool: BlockPool(HashMap::new()),
			// block_seq: Vec::new(),
		}
	}

	/// Get the name of the function.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the return type of the function.
	pub fn return_ty(&self) -> &Type {
		self.ty.as_func().1
	}

	pub fn arg(&self, idx: usize) -> ArgumentRef {
		self.args[idx].as_ref()
	}

	/// Get a reference to the arguments of the function.
	pub fn args(&self) -> &[Argument] {
		&self.args
	}

	/// Get a mutable reference to the arguments of the function.
	pub fn args_mut(&mut self) -> &mut [Argument] {
		&mut self.args
	}

	/// Add a basic block to the end of the function.
	pub fn add_block(&mut self, block: Block) -> BlockRef {
		let ir = block.as_ref();
		self.blocks.insert(ir, block);
		self.block_seq.push(ir);
		ir
	}

	pub fn blocks<'a>(&'a self) -> BlockIter<'a> {
		BlockIter{ refs: self.block_seq.iter(), blocks: &self.blocks }
	}

	/// Add an instruction to the function. Note that this only associates the
	/// instruction with this function. You still need to actually insert the
	/// function into a basic block.
	pub fn add_inst(&mut self, inst: Inst) -> InstRef {
		let ir = inst.as_ref();
		self.insts.insert(ir, inst);
		ir
	}

	/// Add an instruction at the end of a basic block.
	pub fn append_inst(&mut self, inst: InstRef, to: BlockRef) {
		if let Some(old) = self.block_of_inst.insert(inst, to) {
			self.blocks.get_mut(&old).unwrap().remove_inst(inst);
			panic!("inst {} already in basic block {}", inst, to);
		}
		self.blocks.get_mut(&to).expect("basic block does not exist").append_inst(inst);
	}
}


pub struct BlockIter<'tf> {
	refs: std::slice::Iter<'tf, BlockRef>,
	blocks: &'tf std::collections::HashMap<BlockRef, Block>,
}

impl<'tf> std::iter::Iterator for BlockIter<'tf> {
	type Item = &'tf Block;

	fn next(&mut self) -> Option<&'tf Block> {
		let n = self.refs.next();
		n.map(|r| self.blocks.get(r).unwrap())
	}
}


pub struct FunctionContext<'tctx> {
	// module: &'tctx ModuleContext,
	function: &'tctx Function,
}

impl<'tctx> FunctionContext<'tctx> {
	pub fn new(function: &Function) -> FunctionContext {
		FunctionContext {
			function: function,
		}
	}
}

impl<'tctx> Context for FunctionContext<'tctx> {
	// fn parent(&self) -> Option<&Context> {
	// 	Some(self.module)
	// }

	fn try_value(&self, value: &ValueRef) -> Option<&Value> {
		match *value {
			ValueRef::Inst(id) => Some(self.function.insts.get(&id).unwrap()),
			ValueRef::Block(id) => Some(self.function.blocks.get(&id).unwrap()),
			_ => None,
		}
	}
}


/// A function argument or process/entity input or output.
pub struct Argument {
	id: ArgumentRef,
	ty: Type,
	name: Option<String>,
}

impl Argument {
	/// Create a new argument of the given type.
	pub fn new(ty: Type) -> Argument {
		Argument {
			id: ArgumentRef(ValueId::alloc()),
			ty: ty,
			name: None,
		}
	}

	/// Obtain a reference to this argument.
	pub fn as_ref(&self) -> ArgumentRef {
		self.id
	}

	/// Get the type of the argument.
	pub fn ty(&self) -> &Type {
		&self.ty
	}

	/// Get the optional name of the argument.
	pub fn name(&self) -> Option<&str> {
		self.name.as_ref().map(|x| x as &str)
	}

	/// Set the name of the argument.
	pub fn set_name<S: Into<String>>(&mut self, name: S) {
		self.name = Some(name.into());
	}
}

declare_ref!(ArgumentRef, Argument);
