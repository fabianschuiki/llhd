// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

pub mod module;
pub mod ty;
#[macro_use]
pub mod value;
pub mod block;
pub mod inst;
pub mod unit;
// pub mod bitcode;
pub mod argument;
pub mod assembly;
pub mod entity;
pub mod function;
pub mod konst;
pub mod opt;
pub mod process;
pub mod seq_body;
pub mod util;
mod visit;

pub use crate::{
    argument::Argument,
    block::Block,
    entity::{Entity, EntityContext},
    function::{Function, FunctionContext},
    inst::*,
    konst::*,
    module::{Module, ModuleContext},
    process::{Process, ProcessContext},
    ty::*,
    unit::UnitContext,
    value::{Value, ValueId, ValueRef},
    visit::Visitor,
};
