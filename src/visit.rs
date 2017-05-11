// Copyright (c) 2017 Fabian Schuiki

//! The visitor pattern implemented for the LLHD graph.

use unit::{Function, Argument};

pub trait Visitor {
	fn visit_function(&mut self, func: &Function) {
		self.walk_function(func)
	}

	fn visit_arguments(&mut self, args: &[Argument]) {
		self.walk_arguments(args)
	}

	fn visit_argument(&mut self, arg: &Argument) {
	}

	fn walk_function(&mut self, func: &Function) {
		self.visit_arguments(func.args());
	}

	fn walk_arguments(&mut self, args: &[Argument]) {
		for arg in args {
			self.visit_argument(arg);
		}
	}
}
