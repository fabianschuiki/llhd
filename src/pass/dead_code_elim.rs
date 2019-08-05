// Copyright (c) 2017-2019 Fabian Schuiki

//! Dead Code Elimination
//!
//! This module implements dead code elimination. It removes instructions whose
//! value is never used, trivial blocks, and blocks which cannot be reached.

use crate::ir::prelude::*;
use crate::ir::ModUnitData;

/// Eliminate dead code in a module.
pub fn run_on_module(module: &mut Module) -> bool {
    let mut modified = false;
    let units: Vec<_> = module.units().collect();
    for unit in units {
        modified |= match module[unit] {
            ModUnitData::Function(ref mut u) => run_on_function(u),
            ModUnitData::Process(ref mut u) => run_on_process(u),
            ModUnitData::Entity(ref mut u) => run_on_entity(u),
            _ => false,
        };
    }
    modified
}

/// Eliminate dead code in a function.
///
/// Returns `true` if the function was modified.
pub fn run_on_function(func: &mut Function) -> bool {
    let mut builder = FunctionBuilder::new(func);
    let mut modified = false;
    let mut insts = vec![];
    for bb in builder.func.layout.blocks() {
        for inst in builder.func.layout.insts(bb) {
            insts.push(inst);
        }
    }
    for inst in insts {
        modified |= builder.prune_if_unused(inst);
    }
    modified
}

/// Eliminate dead code in a process.
///
/// Returns `true` if the process was modified.
pub fn run_on_process(prok: &mut Process) -> bool {
    let mut builder = ProcessBuilder::new(prok);
    let mut modified = false;
    let mut insts = vec![];
    for bb in builder.prok.layout.blocks() {
        for inst in builder.prok.layout.insts(bb) {
            insts.push(inst);
        }
    }
    for inst in insts {
        modified |= builder.prune_if_unused(inst);
    }
    modified
}

/// Eliminate dead code in an entity.
///
/// Returns `true` if the entity was modified.
pub fn run_on_entity(entity: &mut Entity) -> bool {
    let mut builder = EntityBuilder::new(entity);
    let mut modified = false;
    for inst in builder.entity.layout.insts().collect::<Vec<_>>() {
        modified |= builder.prune_if_unused(inst);
    }
    modified
}
