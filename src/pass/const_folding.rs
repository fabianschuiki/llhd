// Copyright (c) 2017-2019 Fabian Schuiki

//! Constant Folding
//!
//! This module implements constant folding. It replaces instructions with
//! constant arguments with the corresponding result.

use crate::ir::prelude::*;
use crate::{
    ir::{InstData, ModUnitData},
    ty::Type,
    value::IntValue,
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
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold a process.
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
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold an entity.
///
/// Returns `true` if the entity was modified.
pub fn run_on_entity(entity: &mut Entity) -> bool {
    let mut builder = EntityBuilder::new(entity);
    let mut modified = false;
    for inst in builder.entity.layout.insts().collect::<Vec<_>>() {
        modified |= run_on_inst(&mut builder, inst);
    }
    modified
}

/// Fold a single instruction.
///
/// Returns `true` if the unit that contains the instruction was modified.
pub fn run_on_inst(builder: &mut impl UnitBuilder, inst: Inst) -> bool {
    builder.insert_after(inst);

    // Fold branches.
    if let InstData::Branch {
        opcode: Opcode::BrCond,
        args,
        bbs,
    } = builder.dfg()[inst]
    {
        return fold_branch(builder, inst, args[0], bbs).unwrap_or(false);
    }

    // Don't bother folding instructions which don't yield a result.
    if !builder.dfg().has_result(inst) {
        return false;
    }

    // Fold all other instructions.
    let value = builder.dfg().inst_result(inst);
    let ty = builder.dfg().value_type(value);
    let replacement = match builder.dfg()[inst] {
        InstData::Unary { opcode, args, .. } => fold_unary(builder, opcode, ty.clone(), args[0]),
        InstData::Binary { opcode, args, .. } => fold_binary(builder, opcode, ty.clone(), args),
        _ => None,
    };
    if let Some(replacement) = replacement {
        let new_ty = builder.dfg().value_type(replacement);
        assert_eq!(
            ty, new_ty,
            "types before (lhs) and after (rhs) folding must match"
        );
        let dfg = builder.unit_mut().dfg_mut();
        if let Some(name) = dfg.get_name(value).map(String::from) {
            dfg.set_name(replacement, name);
            dfg.clear_name(value);
        }
        dfg.replace_use(value, replacement);
        // builder.prune_if_unused(inst);
        true
    } else {
        false
    }
}

/// Fold a unary instruction.
fn fold_unary(
    builder: &mut impl UnitBuilder,
    opcode: Opcode,
    ty: Type,
    arg: Value,
) -> Option<Value> {
    if ty.is_int() {
        fold_unary_int(builder, opcode, arg)
    } else {
        None
    }
}

/// Fold a unary instruction on integers.
fn fold_unary_int(builder: &mut impl UnitBuilder, opcode: Opcode, arg: Value) -> Option<Value> {
    let imm = builder.dfg().get_const_int(arg)?;
    let result = IntValue::try_unary_op(opcode, imm)?;
    Some(builder.ins().const_int(result))
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
    let imm0 = builder.dfg().get_const_int(args[0]);
    let imm1 = builder.dfg().get_const_int(args[1]);

    // Handle symmetric operations between a constant and a variable argument.
    let (arg_kon, arg_var) = match (imm0, imm1) {
        (None, Some(_)) => (imm1, args[0]),
        (Some(_), None) => (imm0, args[1]),
        _ => (None, args[0]),
    };
    if let Some(a) = arg_kon {
        match opcode {
            Opcode::And | Opcode::Smul | Opcode::Umul if a.is_zero() => {
                return Some(builder.ins().const_int(IntValue::zero(width)))
            }
            Opcode::Or | Opcode::Xor | Opcode::Add | Opcode::Sub if a.is_zero() => {
                return Some(arg_var)
            }
            Opcode::Smul | Opcode::Umul if a.is_one() => return Some(arg_var),
            Opcode::Or if a.is_all_ones() => {
                return Some(builder.ins().const_int(IntValue::all_ones(width)))
            }
            Opcode::And if a.is_all_ones() => return Some(arg_var),
            Opcode::Xor if a.is_all_ones() => return Some(builder.ins().not(arg_var)),
            _ => (),
        }
    }

    // Handle asymmetric operations between a variable argument on the left and
    // a constant argument on the right.
    let (arg_kon, arg_var) = match (imm0, imm1) {
        (None, Some(_)) => (imm1, args[0]),
        _ => (None, args[0]),
    };
    if let Some(a) = arg_kon {
        match opcode {
            Opcode::Sdiv | Opcode::Udiv if a.is_one() => return Some(arg_var),
            Opcode::Smod | Opcode::Umod | Opcode::Srem | Opcode::Urem if a.is_one() => {
                return Some(builder.ins().const_int(IntValue::zero(width)))
            }
            _ => (),
        }
    }

    // Try full constant folding.
    let (imm0, imm1) = (imm0?, imm1?);
    let result = None
        .or_else(|| IntValue::try_binary_op(opcode, imm0, imm1))
        .or_else(|| IntValue::try_compare_op(opcode, imm0, imm1))?;
    Some(builder.ins().const_int(result))
}

/// Fold a branch instruction.
fn fold_branch(
    builder: &mut impl UnitBuilder,
    inst: Inst,
    arg: Value,
    bbs: [Block; 2],
) -> Option<bool> {
    let imm = builder.dfg().get_const_int(arg)?;
    let bb = bbs[!imm.is_zero() as usize];
    builder.ins().br(bb);
    builder.remove_inst(inst);
    // builder.prune_if_unused(arg_inst);
    Some(true)
}
