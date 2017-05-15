// Copyright (c) 2017 Fabian Schuiki

use value::*;
use unit::*;
use ty::*;
use argument::*;
use block::*;
use inst::*;
use seq_body::*;


/// A process. Sequentially executes instructions to react to changes in input
/// signals. Implements *control flow* and *timed execution*.
pub struct Process {
	id: ValueId,
	global: bool,
	name: String,
	ty: Type,
	ins: Vec<Argument>,
	outs: Vec<Argument>,
	body: SeqBody,
}

impl Process {
	/// Create a new process with the given name and type signature. Anonymous
	/// arguments are created for each input and output in the type signature.
	/// Use the `args_mut` function get a hold of these arguments and assign
	/// names and additional data to them.
	pub fn new(name: String, ty: Type) -> Process {
		let (ins, outs) = {
			let (in_tys, out_tys) = ty.as_entity();
			let to_arg = |t: &Type| Argument::new(t.clone());
			(in_tys.iter().map(&to_arg).collect(), out_tys.iter().map(&to_arg).collect())
		};
		Process {
			id: ValueId::alloc(),
			global: true,
			name: name,
			ty: ty,
			ins: ins,
			outs: outs,
			body: SeqBody::new(),
		}
	}

	/// Get the name of the process.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get a reference to the input arguments of the process.
	pub fn inputs(&self) -> &[Argument] {
		&self.ins
	}

	/// Get a mutable reference to the input arguments of the process.
	pub fn inputs_mut(&mut self) -> &mut [Argument] {
		&mut self.ins
	}

	/// Get a reference to the output arguments of the process.
	pub fn outputs(&self) -> &[Argument] {
		&self.outs
	}

	/// Get a mutable reference to the output arguments of the process.
	pub fn outputs_mut(&mut self) -> &mut [Argument] {
		&mut self.outs
	}

	/// Get a reference to the sequential body of the process.
	pub fn body(&self) -> &SeqBody {
		&self.body
	}

	/// Get a mutable reference to the sequential body of the process.
	pub fn body_mut(&mut self) -> &mut SeqBody {
		&mut self.body
	}
}

impl Value for Process {
	fn id(&self) -> ValueId {
		self.id.into()
	}

	fn ty(&self) -> Type {
		self.ty.clone()
	}

	fn name(&self) -> Option<&str> {
		Some(&self.name)
	}

	fn is_global(&self) -> bool {
		self.global
	}
}



pub struct ProcessContext<'tctx> {
	// module: &'tctx ModuleContext,
	process: &'tctx Process,
}

impl<'tctx> ProcessContext<'tctx> {
	pub fn new(process: &Process) -> ProcessContext {
		ProcessContext {
			process: process,
		}
	}
}

impl<'tctx> Context for ProcessContext<'tctx> {
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

impl<'tctx> UnitContext for ProcessContext<'tctx> {
	fn inst(&self, inst: InstRef) -> &Inst {
		self.process.body.inst(inst)
	}

	fn argument(&self, argument: ArgumentRef) -> &Argument {
		if let Some(arg) = self.process.ins.iter().find(|x| argument == x.as_ref()) {
			return arg;
		}
		if let Some(arg) = self.process.outs.iter().find(|x| argument == x.as_ref()) {
			return arg;
		}
		panic!("unknown argument");
	}
}

impl<'tctx> SequentialContext for ProcessContext<'tctx> {
	fn block(&self, block: BlockRef) -> &Block {
		self.process.body.block(block)
	}
}
