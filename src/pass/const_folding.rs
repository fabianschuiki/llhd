// Copyright (c) 2017-2019 Fabian Schuiki

//! Constant Folding

use crate::ir::prelude::*;
use crate::opt::prelude::*;
use crate::{ir::InstData, ty::*, value::IntValue};
use std::cmp::min;

/// Constant Folding
///
/// This pass implements constant folding. It replaces instructions with
/// constant arguments with the corresponding result.
pub struct ConstFolding;

impl Pass for ConstFolding {
    fn run_on_inst(_ctx: &PassContext, inst: Inst, unit: &mut impl UnitBuilder) -> bool {
        run_on_inst(unit, inst)
    }
}

/// Fold a single instruction.
///
/// Returns `true` if the unit that contains the instruction was modified.
pub fn run_on_inst(builder: &mut impl UnitBuilder, inst: Inst) -> bool {
    builder.insert_before(inst);

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
    let data = &builder.dfg()[inst];
    let replacement = match data.opcode() {
        Opcode::InsSlice => fold_ins_slice(builder, inst),
        Opcode::ExtSlice => fold_ext_slice(builder, inst),
        Opcode::Shl | Opcode::Shr => fold_shift(builder, inst, &ty),
        _ => match *data {
            InstData::Unary { opcode, args, .. } => {
                fold_unary(builder, opcode, ty.clone(), args[0])
            }
            InstData::Binary { opcode, args, .. } => fold_binary(builder, opcode, ty.clone(), args),
            _ => None,
        },
    };
    if let Some(replacement) = replacement {
        let new_ty = builder.dfg().value_type(replacement);
        assert_eq!(
            ty,
            new_ty,
            "types before (lhs) and after (rhs) folding must match (before: {})",
            inst.dump(builder.dfg(), builder.try_cfg()),
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

/// Fold a value.
///
/// If the value is an instruction, folds it.
pub fn run_on_value(builder: &mut impl UnitBuilder, value: Value) -> bool {
    if let Some(inst) = builder.dfg().get_value_inst(value) {
        run_on_inst(builder, inst)
    } else {
        false
    }
}

/// Fold a unary instruction.
fn fold_unary(builder: &mut impl UnitBuilder, op: Opcode, ty: Type, arg: Value) -> Option<Value> {
    if ty.is_int() {
        fold_unary_int(builder, op, arg)
    } else {
        None
    }
}

/// Fold a unary instruction on integers.
fn fold_unary_int(builder: &mut impl UnitBuilder, op: Opcode, arg: Value) -> Option<Value> {
    let imm = builder.dfg().get_const_int(arg)?;
    let result = IntValue::try_unary_op(op, imm)?;
    Some(builder.ins().const_int(result))
}

/// Fold a binary instruction.
fn fold_binary(
    builder: &mut impl UnitBuilder,
    op: Opcode,
    ty: Type,
    args: [Value; 2],
) -> Option<Value> {
    if ty.is_int() {
        fold_binary_int(builder, op, ty.unwrap_int(), args)
    } else {
        None
    }
}

/// Fold a binary instruction on integers.
fn fold_binary_int(
    builder: &mut impl UnitBuilder,
    op: Opcode,
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
        match op {
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
        match op {
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
        .or_else(|| IntValue::try_binary_op(op, imm0, imm1))
        .or_else(|| IntValue::try_compare_op(op, imm0, imm1))?;
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

/// Fold a shift instruction.
fn fold_shift(builder: &mut impl UnitBuilder, inst: Inst, ty: &Type) -> Option<Value> {
    let dfg = builder.dfg();
    let base = dfg[inst].args()[0];
    let hidden = dfg[inst].args()[1];
    let amount = dfg[inst].args()[2];

    let const_amount = dfg.get_const_int(amount);
    let left = dfg[inst].opcode() == Opcode::Shl;

    // Handle the trivial case where the shift amount is zero.
    if const_amount.map(IntValue::is_zero).unwrap_or(false) {
        return Some(base);
    }

    // Handle the case where the shfit amount is constant.
    if let Some(amount) = const_amount {
        let amount = amount.to_usize();
        let base_width = dfg.value_type(base).len();
        let hidden_width = dfg.value_type(hidden).len();
        let amount = min(amount, hidden_width);

        // Handle the case where the amount fully shifts out the base.
        if amount >= base_width {
            let offset = if left {
                amount - base_width
            } else {
                hidden_width + base_width - amount
            };
            let r = builder.ins().ext_slice(hidden, offset, base_width);
            Some(fold_ext_slice(builder, builder.dfg().value_inst(r)).unwrap_or(r));
        }
        // Handle the case where the result is a mixture of the base and the
        // hidden value.
        else {
            let (b, h, z0, z1, z2) = if left {
                let b = builder.ins().ext_slice(base, 0, base_width - amount);
                let h = builder
                    .ins()
                    .ext_slice(hidden, hidden_width - amount, amount);
                let z0 = builder.ins().const_zero(ty);
                let z1 = builder.ins().ins_slice(z0, b, amount, base_width - amount);
                let z2 = builder.ins().ins_slice(z1, h, 0, amount);
                (b, h, z0, z1, z2)
            } else {
                let h = builder.ins().ext_slice(hidden, 0, amount);
                let b = builder.ins().ext_slice(base, amount, base_width - amount);
                let z0 = builder.ins().const_zero(ty);
                let z1 = builder.ins().ins_slice(z0, h, base_width - amount, amount);
                let z2 = builder.ins().ins_slice(z1, b, 0, base_width - amount);
                (b, h, z0, z1, z2)
            };
            run_on_value(builder, h);
            run_on_value(builder, b);
            run_on_value(builder, z0);
            run_on_value(builder, z1);
            return Some(fold_ins_slice(builder, builder.dfg().value_inst(z2)).unwrap_or(z2));
        }
    }

    None
}

/// Fold a slice insertion instruction.
fn fold_ins_slice(builder: &mut impl UnitBuilder, inst: Inst) -> Option<Value> {
    let dfg = builder.dfg();
    let data = &dfg[inst];
    let target = data.args()[0];
    let value = data.args()[1];
    let len = data.imms()[1];

    // Handle the trivial cases where we override the entire value, or nothing
    // at all.
    match dfg.value_type(target).as_ref() {
        IntType(_) | ArrayType(..) if len == 0 => return Some(target),
        IntType(w) | ArrayType(w, _) if len == *w => return Some(value),
        _ => (),
    }

    // Handle the case where both operands are constant integers.
    if let (Some(target), Some(value)) = (dfg.get_const_int(target), dfg.get_const_int(value)) {
        let mut r = target.clone();
        r.insert_slice(data.imms()[0], len, value);
        return Some(builder.ins().const_int(r));
    }

    None
}

/// Fold a slice extraction instruction.
fn fold_ext_slice(builder: &mut impl UnitBuilder, inst: Inst) -> Option<Value> {
    let dfg = builder.dfg();
    let data = &dfg[inst];
    let ty = &dfg.inst_type(inst);
    let target = data.args()[0];
    let len = data.imms()[1];

    // Handle the trivial case where we extract the entire value, or nothing
    // at all.
    match dfg.value_type(target).as_ref() {
        IntType(..) | ArrayType(..) if len == 0 => return Some(builder.ins().const_zero(ty)),
        IntType(w) | ArrayType(w, _) if len == *w => return Some(target),
        _ => (),
    }

    // Handle the case where the target is a constant integer.
    if let Some(imm) = dfg.get_const_int(target) {
        let r = imm.extract_slice(data.imms()[0], len);
        return Some(builder.ins().const_int(r));
    }

    None
}
