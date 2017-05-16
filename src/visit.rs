// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! The visitor pattern implemented for the LLHD graph.

use unit::*;
use module::Module;
use block::Block;
use inst::Inst;
use function::{Function, FunctionContext};
use process::{Process, ProcessContext};
use entity::{Entity, EntityContext};
use value::ValueRef;
use argument::Argument;


/// A trait to implement the visitor pattern on an LLHD graph.
pub trait Visitor {
	fn visit_module(&mut self, module: &Module) {
		self.walk_module(module)
	}

	fn visit_module_value(&mut self, module: &Module, value: &ValueRef) {
		match *value {
			ValueRef::Function(r) => self.visit_function(module.function(r)),
			ValueRef::Process(r) => self.visit_process(module.process(r)),
			ValueRef::Entity(r) => self.visit_entity(module.entity(r)),
			_ => panic!("invalid value in module")
		}
	}

	fn visit_function(&mut self, func: &Function) {
		self.walk_function(func)
	}

	fn visit_process(&mut self, prok: &Process) {
		self.walk_process(prok)
	}

	fn visit_entity(&mut self, entity: &Entity) {
		self.walk_entity(entity)
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


	fn walk_module(&mut self, module: &Module) {
		for value in module.values() {
			self.visit_module_value(module, value);
		}
	}

	fn walk_function(&mut self, func: &Function) {
		let ctx = FunctionContext::new(func);
		self.visit_arguments(func.args());
		for block in func.body().blocks() {
			self.visit_block(&ctx, block);
		}
	}

	fn walk_process(&mut self, prok: &Process) {
		let ctx = ProcessContext::new(prok);
		self.visit_arguments(prok.inputs());
		self.visit_arguments(prok.outputs());
		for block in prok.body().blocks() {
			self.visit_block(&ctx, block);
		}
	}

	fn walk_entity(&mut self, entity: &Entity) {
		let ctx = EntityContext::new(entity);
		self.visit_arguments(entity.inputs());
		self.visit_arguments(entity.outputs());
		let uctx = ctx.as_unit_context();
		for inst in entity.insts() {
			self.visit_inst(uctx, inst);
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
