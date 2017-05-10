// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! Modules in LLHD encapsulate a design hierarchy and its data dependency and
//! control flow graphs.

use ty::*;
use unit::Function;

pub struct Module {

}

impl Module {
	/// Create a new empty module.
	pub fn new() -> Module {
		Module {}
	}

	/// Create a void type.
	pub fn void_ty(&self) -> Type {
		Type::new(VoidType)
	}

	/// Create an integer type of the requested size.
	pub fn int_ty(&self, size: usize) -> Type {
		Type::new(IntType(size))
	}

	/// Create a function type with the given arguments and return type.
	pub fn func_ty(&self, args: Vec<Type>, ret: Type) -> Type {
		Type::new(FuncType(args, ret))
	}

	pub fn add_function<N: Into<String>>(&self, name: N, ty: Type) {
		Function::new(name.into(), ty);
	}
}
