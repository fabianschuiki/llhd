// Copyright (c) 2017 Fabian Schuiki

use crate::argument::*;
use crate::inst::*;
use crate::value::{ArgumentRef, Context, InstRef};

/// A context wrapping a unit.
pub trait UnitContext: Context + AsUnitContext {
    /// Resolve a `InstRef` to an actual `&Inst` reference.
    fn inst(&self, inst: InstRef) -> &Inst;
    /// Resolve a `ArgumentRef` to an actual `&Argument` reference.
    fn argument(&self, argument: ArgumentRef) -> &Argument;
}

pub trait AsUnitContext {
    fn as_unit_context(&self) -> &UnitContext;
}

impl<T: UnitContext> AsUnitContext for T {
    fn as_unit_context(&self) -> &UnitContext {
        self
    }
}
