// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code)]

use inst::Inst;
use std::collections::HashMap;
use ty::*;

pub struct Block(Vec<InstRef>);


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
	inst_pool: InstPool,
	block_pool: BlockPool,
	block_seq: Vec<BlockRef>,
}

pub struct Argument {
	ty: Type,
	name: Option<String>,
}


#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InstRef(usize);

pub struct InstPool(HashMap<InstRef, Inst>);


#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockRef(usize);

pub struct BlockPool(HashMap<BlockRef, Block>);

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
			inst_pool: InstPool(HashMap::new()),
			block_pool: BlockPool(HashMap::new()),
			block_seq: Vec::new(),
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

	/// Get a reference to the arguments of the function.
	pub fn args(&self) -> &[Argument] {
		&self.args
	}

	/// Get a mutable reference to the arguments of the function.
	pub fn args_mut(&mut self) -> &mut [Argument] {
		&mut self.args
	}
}

impl Argument {
	/// Create a new argument of the given type.
	pub fn new(ty: Type) -> Argument {
		Argument {
			ty: ty,
			name: None,
		}
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
