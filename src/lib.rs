// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

pub mod module;
pub mod ty;
#[macro_use]
pub mod value;
pub mod unit;
pub mod block;
pub mod inst;
// pub mod bitcode;
pub mod function;
pub mod argument;
pub mod assembly;
pub mod visit;
pub mod util;

pub use module::Module;
pub use value::ValueRef;
pub use ty::*;
pub use function::Function;
