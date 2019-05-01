// Copyright (c) 2017 Fabian Schuiki

use crate::inst::*;
use crate::value::{Context, InstRef};

/// A context wrapping a unit.
pub trait UnitContext: Context + AsUnitContext {
    /// Resolve a `InstRef` to an actual `&Inst` reference.
    fn inst(&self, inst: InstRef) -> &Inst;
}

pub trait AsUnitContext {
    fn as_unit_context(&self) -> &UnitContext;
}

impl<T: UnitContext> AsUnitContext for T {
    fn as_unit_context(&self) -> &UnitContext {
        self
    }
}
