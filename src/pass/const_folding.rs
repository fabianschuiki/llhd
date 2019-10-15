// Copyright (c) 2017-2019 Fabian Schuiki

//! Constant Folding
//!
//! This module implements constant folding. It replaces instructions with
//! constant arguments with the corresponding result.

use crate::ir::prelude::*;
use crate::{
    ir::{InstData, ModUnitData},
    ty::Type,
};
use num::{BigInt, Zero};

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
        builder.prune_if_unused(inst);
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
        fold_unary_int(builder, opcode, ty.unwrap_int(), arg)
    } else {
        None
    }
}

/// Fold a unary instruction on integers.
fn fold_unary_int(
    builder: &mut impl UnitBuilder,
    opcode: Opcode,
    width: usize,
    arg: Value,
) -> Option<Value> {
    let inst = builder.dfg().get_value_inst(arg)?;
    let imm = builder.dfg()[inst].get_const_int()?;
    let result = match opcode {
        Opcode::Not => (BigInt::from(1) << width) - 1 - imm,
        Opcode::Neg => -imm,
        _ => return None,
    };
    Some(builder.ins().const_int(width, result))
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
    let inst0 = builder.dfg().get_value_inst(args[0])?;
    let inst1 = builder.dfg().get_value_inst(args[1])?;
    let imm0 = builder.dfg()[inst0].get_const_int()?;
    let imm1 = builder.dfg()[inst1].get_const_int()?;
    let result = match opcode {
        Opcode::Add => imm0 + imm1,
        Opcode::Sub => imm0 - imm1,
        Opcode::And => imm0 & imm1,
        Opcode::Or => imm0 | imm1,
        Opcode::Xor => imm0 ^ imm1,
        Opcode::Smul => imm0 * imm1,
        Opcode::Sdiv => imm0 / imm1,
        Opcode::Smod => imm0 % imm1,
        Opcode::Srem => imm0 % imm1,
        Opcode::Umul => (imm0.to_biguint()? * imm1.to_biguint()?).into(),
        Opcode::Udiv => (imm0.to_biguint()? / imm1.to_biguint()?).into(),
        Opcode::Umod => (imm0.to_biguint()? % imm1.to_biguint()?).into(),
        Opcode::Urem => (imm0.to_biguint()? % imm1.to_biguint()?).into(),
        Opcode::Eq => ((imm0 == imm1) as usize).into(),
        Opcode::Neq => ((imm0 != imm1) as usize).into(),
        Opcode::Slt => ((imm0 < imm1) as usize).into(),
        Opcode::Sgt => ((imm0 > imm1) as usize).into(),
        Opcode::Sle => ((imm0 <= imm1) as usize).into(),
        Opcode::Sge => ((imm0 >= imm1) as usize).into(),
        Opcode::Ult => ((imm0.to_biguint()? < imm1.to_biguint()?) as usize).into(),
        Opcode::Ugt => ((imm0.to_biguint()? > imm1.to_biguint()?) as usize).into(),
        Opcode::Ule => ((imm0.to_biguint()? <= imm1.to_biguint()?) as usize).into(),
        Opcode::Uge => ((imm0.to_biguint()? >= imm1.to_biguint()?) as usize).into(),
        _ => return None,
    };
    Some(builder.ins().const_int(width, result))
}

/// Fold a branch instruction.
fn fold_branch(
    builder: &mut impl UnitBuilder,
    inst: Inst,
    arg: Value,
    bbs: [Block; 2],
) -> Option<bool> {
    let arg_inst = builder.dfg().get_value_inst(arg)?;
    let imm = builder.dfg()[arg_inst].get_const_int()?;
    let bb = bbs[!imm.is_zero() as usize];
    builder.ins().br(bb);
    builder.remove_inst(inst);
    builder.prune_if_unused(arg_inst);
    Some(true)
}
