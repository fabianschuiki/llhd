// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

#[macro_use]
extern crate combine;
extern crate num;

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
pub mod visit;

pub use argument::Argument;
pub use block::Block;
pub use entity::{Entity, EntityContext};
pub use function::{Function, FunctionContext};
pub use inst::*;
pub use konst::*;
pub use module::{Module, ModuleContext};
pub use process::{Process, ProcessContext};
pub use ty::*;
pub use unit::UnitContext;
pub use value::{Value, ValueId, ValueRef};
