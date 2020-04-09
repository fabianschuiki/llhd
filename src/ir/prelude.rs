// Copyright (c) 2017-2020 Fabian Schuiki

//! Re-exports of commonly used IR items.

#[allow(deprecated)]
pub use crate::ir::{
    Arg, Block, DeclData, Inst, ModUnit, Module, Opcode, RegMode, RegTrigger, Signature, Unit,
    UnitBuilder, UnitData, UnitDataBuilder, UnitKind, UnitName, Value,
};
