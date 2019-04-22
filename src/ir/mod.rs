// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD functions, processes, and entitites.
//!
//! This module implements the intermediate representation around which the rest
//! of the framework is built.

use crate::{impl_table_key, ty::Type};

mod dfg;
mod inst;
mod layout;
mod sig;

pub use self::dfg::*;
pub use self::inst::*;
pub use self::layout::*;
pub use self::sig::*;

impl_table_key! {
    /// An instruction.
    struct Inst(u32) as "i";

    /// A value.
    struct Value(u32) as "v";

    /// A basic block.
    struct Block(u32) as "bb";

    /// An argument of a `Function`, `Process`, or `Entity`.
    struct Arg(u32) as "arg";

    /// An external `Function`, `Process` or `Entity`.
    struct ExtUnit(u32) as "ext";
}

impl Value {
    /// A placeholder for invalid values.
    ///
    /// This is used for unused instruction arguments.
    fn invalid() -> Self {
        Value(std::u32::MAX)
    }
}

/// Internal table storage for values.
#[derive(Debug)]
pub enum ValueData {
    /// The value is the result of an instruction.
    Inst { ty: Type, inst: Inst },
    /// The value is an argument of the `Function`, `Process`, or `Entity`.
    Arg { ty: Type, arg: Arg },
}

/// Another unit referenced within a `Function`, `Process`, or `Entity`.
///
/// The linker will hook up external units to the actual counterparts as
/// appropriate.
#[derive(Debug)]
pub struct ExtUnitData {
    /// The name of the referenced unit.
    pub name: UnitName,
    /// The signature of the referenced unit.
    pub sig: Signature,
}
