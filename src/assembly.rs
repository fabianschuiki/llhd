// Copyright (c) 2017 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

use std;
use visit::Visitor;
use std::io::Write;
use unit::{Function, Argument};

/// Emits a module as human-readable assembly code that can be parsed again
/// later.
pub struct Writer<'twr> {
	sink: &'twr mut Write,
}

impl<'twr> Writer<'twr> {
	/// Create a new assembly writer that will emit code into the provided sink.
	pub fn new(sink: &mut Write) -> Writer {
		Writer {
			sink: sink,
		}
	}
}

impl<'twr> Visitor for Writer<'twr> {
	fn visit_function(&mut self, func: &Function) {
		write!(self.sink, "func @{} (", func.name()).unwrap();
		self.visit_arguments(func.args());
		write!(self.sink, ") {} {{\n", func.return_ty()).unwrap();
		write!(self.sink, "}}\n").unwrap();
	}

	fn visit_arguments(&mut self, args: &[Argument]) {
		for (arg, sep) in args.iter().zip(std::iter::once("").chain(std::iter::repeat(", "))) {
			write!(self.sink, "{}", sep).unwrap();
			self.visit_argument(arg);
		}
	}

	fn visit_argument(&mut self, arg: &Argument) {
		write!(self.sink, "{}", arg.ty()).unwrap();
		if let Some(name) = arg.name() {
			write!(self.sink, " %{}", name).unwrap();
		}
	}
}
