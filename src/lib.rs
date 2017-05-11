// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

pub mod module;
pub mod ty;
pub mod value;
pub mod unit;
pub mod inst;
// pub mod bitcode;
pub mod assembly;
pub mod visit;
pub mod util;

pub use module::Module;
pub use value::ValueRef;
