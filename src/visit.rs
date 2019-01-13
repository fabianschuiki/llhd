// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! The visitor pattern implemented for the LLHD graph.

use argument::Argument;
use block::Block;
use entity::{Entity, EntityContext};
use function::{Function, FunctionContext};
use inst::Inst;
use module::{Module, ModuleContext};
use process::{Process, ProcessContext};
use unit::*;
use value::ValueRef;

/// A trait to implement the visitor pattern on an LLHD graph.
pub trait Visitor {
    fn visit_module(&mut self, module: &Module) {
        self.walk_module(module)
    }

    fn visit_module_value(&mut self, ctx: &ModuleContext, value: &ValueRef) {
        match *value {
            ValueRef::Function(r) => self.visit_function(ctx, ctx.function(r)),
            ValueRef::Process(r) => self.visit_process(ctx, ctx.process(r)),
            ValueRef::Entity(r) => self.visit_entity(ctx, ctx.entity(r)),
            _ => panic!("invalid value in module"),
        }
    }

    fn visit_function(&mut self, ctx: &ModuleContext, func: &Function) {
        self.walk_function(ctx, func)
    }

    fn visit_process(&mut self, ctx: &ModuleContext, prok: &Process) {
        self.walk_process(ctx, prok)
    }

    fn visit_entity(&mut self, ctx: &ModuleContext, entity: &Entity) {
        self.walk_entity(ctx, entity)
    }

    fn visit_arguments(&mut self, args: &[Argument]) {
        self.walk_arguments(args)
    }

    fn visit_argument(&mut self, &Argument) {}

    fn visit_block(&mut self, ctx: &SequentialContext, block: &Block) {
        self.walk_block(ctx, block)
    }

    fn visit_inst(&mut self, ctx: &UnitContext, inst: &Inst) {}

    fn walk_module(&mut self, module: &Module) {
        let ctx = ModuleContext::new(module);
        for value in module.values() {
            self.visit_module_value(&ctx, value);
        }
    }

    fn walk_function(&mut self, ctx: &ModuleContext, func: &Function) {
        let ctx = FunctionContext::new(ctx, func);
        self.visit_arguments(func.args());
        for block in func.body().blocks() {
            self.visit_block(&ctx, block);
        }
    }

    fn walk_process(&mut self, ctx: &ModuleContext, prok: &Process) {
        let ctx = ProcessContext::new(ctx, prok);
        self.visit_arguments(prok.inputs());
        self.visit_arguments(prok.outputs());
        for block in prok.body().blocks() {
            self.visit_block(&ctx, block);
        }
    }

    fn walk_entity(&mut self, ctx: &ModuleContext, entity: &Entity) {
        let ctx = EntityContext::new(ctx, entity);
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
