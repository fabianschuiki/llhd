// Copyright (c) 2017 Fabian Schuiki

use crate::value::Context;

/// A context wrapping a unit.
pub trait UnitContext: Context + AsUnitContext {}

pub trait AsUnitContext {
    fn as_unit_context(&self) -> &UnitContext;
}

impl<T: UnitContext> AsUnitContext for T {
    fn as_unit_context(&self) -> &UnitContext {
        self
    }
}
