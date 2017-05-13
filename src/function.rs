// Copyright (c) 2017 Fabian Schuiki

use std::collections::HashMap;
use value::*;
use unit::*;
use ty::*;
use argument::*;
use block::*;
use inst::*;


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
		BlockIter::new(self.block_seq.iter(), &self.blocks)
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
			ValueRef::Inst(id) => Some(self.inst(id)),
			ValueRef::Block(id) => Some(self.block(id)),
			ValueRef::Argument(id) => Some(self.argument(id)),
			_ => None,
		}
	}
}

impl<'tctx> UnitContext for FunctionContext<'tctx> {
	fn inst(&self, inst: InstRef) -> &Inst {
		self.function.insts.get(&inst).unwrap()
	}

	fn argument(&self, argument: ArgumentRef) -> &Argument {
		self.function.args.iter().find(|x| argument == x.as_ref()).unwrap()
	}
}

impl<'tctx> SequentialContext for FunctionContext<'tctx> {
	fn block(&self, block: BlockRef) -> &Block {
		self.function.blocks.get(&block).unwrap()
	}
}
