// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

use std;
use visit::Visitor;
use std::io::Write;
use unit::*;
use block::*;
use value::*;
use std::rc::Rc;
use std::collections::HashMap;


/// Emits a module as human-readable assembly code that can be parsed again
/// later.
pub struct Writer<'twr> {
	sink: &'twr mut Write,
	/// A table of uniquified names assigned to values.
	name_table: HashMap<ValueId, Rc<String>>,
	/// A stack of maps that keep a counter for every string encountered in
	/// a namespace. Currently there are only two namespaces: modules and units.
	name_stack: Vec<(usize, HashMap<String, usize>)>,
}

impl<'twr> Writer<'twr> {
	/// Create a new assembly writer that will emit code into the provided sink.
	pub fn new(sink: &mut Write) -> Writer {
		Writer {
			sink: sink,
			name_table: HashMap::new(),
			name_stack: vec![(0, HashMap::new())],
		}
	}

	/// Determine a unique name for a value. Either returns the value's name, or
	/// extends it with a monotonically increasing number. If the value has no
	/// name assigned, returns a unique temporary name.
	fn uniquify(&mut self, value: &Value) -> Rc<String> {
		let id = value.id();
		if let Some(name) = self.name_table.get(&id).cloned() {
			name
		} else {
			let name = self.uniquify_name(value.name());
			self.name_table.insert(id, name.clone());
			name
		}
	}

	/// Ensures that a name is unique within the current name stack, extending
	/// the name with a monotonically increasing number or using a temporary
	/// name altogether, if need be.
	fn uniquify_name(&mut self, name: Option<&str>) -> Rc<String> {
		if let Some(name) = name {
			// First handle the easy case where the namespace at the top of the
			// stack already has a uniquification counter for the name. If that
			// is the case, increment it and append the old count to the
			// requested name.
			if let Some(index) = self.name_stack.last_mut().unwrap().1.get_mut(name) {
				let n = Rc::new(format!("{}{}", name, index));
				*index += 1;
				return n;
			}

			// Traverse the name stack top to bottom, looking for an existing
			// uniquification counter for the name. If we find one, store the
			// count and break out of the loop. We'll use this count to uniquify
			// the name.
			let mut index: Option<usize> = None;
			for &(_, ref names) in self.name_stack.iter().rev().skip(1) {
				if let Some(&i) = names.get(name) {
					index = Some(i);
					break;
				}
			}

			// Insert the name into the topmost namespace, either using the
			// existing uniquification count we'found, or 0.
			self.name_stack.last_mut().unwrap().1.insert(name.to_owned(), index.unwrap_or(0));

			// If we have found a uniquification count, append that to the name.
			// Otherwise just use the name as it is.
			Rc::new(index.map(|i| format!("{}{}", name, i)).unwrap_or(name.to_owned()))

		} else {
			// We arrive here if `None` was passed as a name. This case is
			// trivial. We simply increment the index for unnamed values, and
			// convert the old index to a string to be used as the value's name.
			let ref mut index = self.name_stack.last_mut().unwrap().0;
			let name = Rc::new(format!("{}", index));
			*index += 1;
			name
		}
	}

	/// Add an empty namespace to the top of the name stack. Future temporary
	/// and uniquified names will be stored there.
	fn push(&mut self) {
		let index = self.name_stack.last().unwrap().0;
		self.name_stack.push((index, HashMap::new()))
	}

	/// Remove the topmost namespace from the name stack.
	fn pop(&mut self) {
		self.name_stack.pop();
		assert!(!self.name_stack.is_empty())
	}
}

impl<'twr> Visitor for Writer<'twr> {
	fn visit_function(&mut self, func: &Function) {
		let ctx = FunctionContext::new(func);
		self.push();
		write!(self.sink, "func @{} (", func.name()).unwrap();
		self.visit_arguments(func.args());
		write!(self.sink, ") {} {{\n", func.return_ty()).unwrap();
		for block in func.blocks() {
			self.visit_block(&ctx, block);
		}
		write!(self.sink, "}}\n").unwrap();
		self.pop();
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

	fn visit_block(&mut self, ctx: &Context, block: &Block) {
		let name = self.uniquify(block);
		write!(self.sink, "%{}:\n", name).unwrap();
	}
}
