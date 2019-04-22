// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD functions, processes, and entitites.
//!
//! This module implements the intermediate representation around which the rest
//! of the framework is built.

use crate::{impl_table_key};

mod layout;
mod sig;

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
