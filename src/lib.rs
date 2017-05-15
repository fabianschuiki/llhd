// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

extern crate num;

pub mod module;
pub mod ty;
#[macro_use]
pub mod value;
pub mod unit;
pub mod block;
pub mod inst;
// pub mod bitcode;
pub mod function;
pub mod process;
pub mod entity;
pub mod argument;
pub mod assembly;
pub mod visit;
pub mod util;
pub mod konst;
pub mod seq_body;

pub use module::Module;
pub use value::ValueRef;
pub use ty::*;
pub use konst::*;
pub use function::Function;
pub use process::Process;
pub use entity::Entity;
