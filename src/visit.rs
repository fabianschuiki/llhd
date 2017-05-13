// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! The visitor pattern implemented for the LLHD graph.

use unit::*;
// use value::*;
use block::Block;
use inst::Inst;
use function::{Function, FunctionContext};
use argument::Argument;


/// A trait to implement the visitor pattern on an LLHD graph.
pub trait Visitor {
	fn visit_function(&mut self, func: &Function) {
		self.walk_function(func)
	}

	fn visit_arguments(&mut self, args: &[Argument]) {
		self.walk_arguments(args)
	}

	fn visit_argument(&mut self, &Argument) {
	}

	fn visit_block(&mut self, ctx: &SequentialContext, block: &Block) {
		self.walk_block(ctx, block)
	}

	fn visit_inst(&mut self, ctx: &UnitContext, inst: &Inst) {
	}

	fn walk_function(&mut self, func: &Function) {
		let ctx = FunctionContext::new(func);
		self.visit_arguments(func.args());
		for block in func.blocks() {
			self.visit_block(&ctx, block);
		}
	}

	fn walk_arguments(&mut self, args: &[Argument]) {
		for arg in args {
			self.visit_argument(arg);
		}
	}

	fn walk_block(&mut self, ctx: &SequentialContext, block: &Block) {
		let uctx = ctx.as_unit_context();
		for inst in block.insts(uctx) {
			self.visit_inst(uctx, inst);
		}
	}
}
