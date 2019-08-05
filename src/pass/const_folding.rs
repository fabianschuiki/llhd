// Copyright (c) 2017-2019 Fabian Schuiki

//! Constant Folding
//!
//! This module implements constant folding. It replaces instructions with
//! constant arguments with the corresponding result.

use crate::ir::prelude::*;
use crate::{
    ir::{InstData, ModUnitData},
    ty::{signal_ty, Type},
};

/// Fold a module.
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

/// Fold a function.
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
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold a process.
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
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold an entity.
pub fn run_on_entity(entity: &mut Entity) -> bool {
    let mut builder = EntityBuilder::new(entity);
    let mut modified = false;
    for inst in builder.entity.layout.insts().collect::<Vec<_>>() {
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold a single instruction.
pub fn run_on_inst(builder: &mut impl UnitBuilder, inst: Inst) -> bool {
    if !builder.dfg().has_result(inst) {
        return false;
    }
    builder.insert_after(inst);
    let value = builder.dfg().inst_result(inst);
    let ty = builder.dfg().value_type(value);
    let replacement = match builder.dfg()[inst] {
        InstData::Binary { opcode, args, .. } => fold_binary(builder, opcode, ty.clone(), args),
        _ => None,
    };
    if let Some(replacement) = replacement {
        let new_ty = builder.dfg().value_type(replacement);
        assert!(
            ty == new_ty || ty == signal_ty(new_ty),
            "types before (lhs) and after (rhs) folding must match"
        );
        builder.unit_mut().dfg_mut().replace_use(value, replacement);
        builder.prune_if_unused(inst);
        true
    } else {
        false
    }
}

/// Fold a binary instruction.
fn fold_binary(
    builder: &mut impl UnitBuilder,
    opcode: Opcode,
    ty: Type,
    args: [Value; 2],
) -> Option<Value> {
    if ty.is_int() {
        fold_binary_int(builder, opcode, ty.unwrap_int(), args)
    } else if ty.is_signal() && ty.unwrap_signal().is_int() {
        fold_binary_int(builder, opcode, ty.unwrap_signal().unwrap_int(), args)
    } else {
        None
    }
}

/// Fold a binary instruction on integers.
fn fold_binary_int(
    builder: &mut impl UnitBuilder,
    opcode: Opcode,
    width: usize,
    args: [Value; 2],
) -> Option<Value> {
    let inst0 = builder.dfg().get_value_inst(args[0])?;
    let inst1 = builder.dfg().get_value_inst(args[1])?;
    let imm0 = builder.dfg()[inst0].get_const_int()?;
    let imm1 = builder.dfg()[inst1].get_const_int()?;
    let result = match opcode {
        Opcode::Add => imm0 + imm1,
        _ => return None,
    };
    Some(builder.ins().const_int(width, false, result))
}
