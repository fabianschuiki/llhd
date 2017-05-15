// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! Modules in LLHD encapsulate a design hierarchy and its data dependency and
//! control flow graphs.

use ty::*;
use function::Function;
use process::Process;

pub struct Module {

}

impl Module {
	/// Create a new empty module.
	pub fn new() -> Module {
		Module {}
	}

	/// Create a new function in the module.
	pub fn add_function<N: Into<String>>(&self, name: N, ty: Type) -> Function {
		Function::new(name.into(), ty)
	}

	/// Create a new process in the module.
	pub fn add_process<N: Into<String>>(&self, name: N, ty: Type) -> Process {
		Process::new(name.into(), ty)
	}
}
