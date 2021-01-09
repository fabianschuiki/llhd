// Copyright (c) 2017-2021 Fabian Schuiki

//! Re-exports of commonly used IR items.

#[allow(deprecated)]
pub use crate::ir::{
    Arg, Block, DeclData, DeclId, Inst, Module, Opcode, RegMode, RegTrigger, Signature, Unit,
    UnitBuilder, UnitData, UnitId, UnitKind, UnitName, Value,
};
