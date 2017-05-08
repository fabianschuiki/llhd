// Copyright (c) 2017 Fabian Schuiki

//! The Low Level Hardware Description language. This library provides tools to
//! create, modify, store, and load LLHD graphs.

pub mod module;
pub mod ty;
pub mod value;
// pub mod ins;
// pub mod bitcode;
// pub mod assembly;

pub use module::Module;

#[test]
fn it_works() {
}
