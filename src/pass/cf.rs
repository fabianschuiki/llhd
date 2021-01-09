// Copyright (c) 2017-2021 Fabian Schuiki

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
    fn run_on_inst(_ctx: &PassContext, inst: Inst, unit: &mut UnitBuilder) -> bool {
        run_on_inst(unit, inst)
    }
}

/// Fold a single instruction.
///
/// Returns `true` if the unit that contains the instruction was modified.
pub fn run_on_inst(unit: &mut UnitBuilder, inst: Inst) -> bool {
    unit.insert_before(inst);

    // Don't bother folding instructions which don't yield a result.
    if !unit.has_result(inst) {
        return false;
    }

    // Fold all other instructions.
    let value = unit.inst_result(inst);
    let ty = unit.value_type(value);
    let data = &unit[inst];
    let replacement = match data.opcode() {
        Opcode::InsSlice => fold_ins_slice(unit, inst),
        Opcode::ExtSlice => fold_ext_slice(unit, inst),
        Opcode::ExtField => fold_ext_field(unit, inst),
        Opcode::Shl | Opcode::Shr => fold_shift(unit, inst, &ty),
        Opcode::Mux => fold_mux(unit, inst),
        _ => match *data {
            InstData::Unary { opcode, args, .. } => fold_unary(unit, opcode, ty.clone(), args[0]),
            InstData::Binary { opcode, args, .. } => fold_binary(unit, opcode, ty.clone(), args),
            _ => None,
        },
    };
    if let Some(replacement) = replacement {
        let new_ty = unit.value_type(replacement);
        assert_eq!(
            ty,
            new_ty,
            "types before (lhs) and after (rhs) folding must match (before: {}, after: {})",
            inst.dump(&unit),
            unit.get_value_inst(replacement)
                .map(|v| v.dump(&unit).to_string())
                .unwrap_or_else(|| replacement.dump(&unit).to_string())
        );
        if let Some(name) = unit.get_name(value).map(String::from) {
            unit.set_name(replacement, name);
            unit.clear_name(value);
        }
        unit.replace_use(value, replacement);
        // unit.prune_if_unused(inst);
        true
    } else {
        false
    }
}

/// Fold a value.
///
/// If the value is an instruction, folds it.
pub fn run_on_value(unit: &mut UnitBuilder, value: Value) -> bool {
    if let Some(inst) = unit.get_value_inst(value) {
        run_on_inst(unit, inst)
    } else {
        false
    }
}

/// Fold a unary instruction.
fn fold_unary(unit: &mut UnitBuilder, op: Opcode, ty: Type, arg: Value) -> Option<Value> {
    if ty.is_int() {
        fold_unary_int(unit, op, arg)
    } else {
        None
    }
}

/// Fold a unary instruction on integers.
fn fold_unary_int(unit: &mut UnitBuilder, op: Opcode, arg: Value) -> Option<Value> {
    let imm = unit.get_const_int(arg)?;
    let result = IntValue::try_unary_op(op, imm)?;
    Some(unit.ins().const_int(result))
}

/// Fold a binary instruction.
fn fold_binary(unit: &mut UnitBuilder, op: Opcode, ty: Type, args: [Value; 2]) -> Option<Value> {
    if ty.is_int() {
        fold_binary_int(unit, op, ty.unwrap_int(), args)
    } else {
        None
    }
}

/// Fold a binary instruction on integers.
fn fold_binary_int(
    unit: &mut UnitBuilder,
    op: Opcode,
    width: usize,
    args: [Value; 2],
) -> Option<Value> {
    let imm0 = unit.get_const_int(args[0]);
    let imm1 = unit.get_const_int(args[1]);

    // Handle symmetric operations between a constant and a variable argument.
    let (arg_kon, arg_var) = match (imm0, imm1) {
        (None, Some(_)) => (imm1, args[0]),
        (Some(_), None) => (imm0, args[1]),
        _ => (None, args[0]),
    };
    if let Some(a) = arg_kon {
        match op {
            Opcode::And | Opcode::Smul | Opcode::Umul if a.is_zero() => {
                return Some(unit.ins().const_int(IntValue::zero(width)))
            }
            Opcode::Or | Opcode::Xor | Opcode::Add | Opcode::Sub if a.is_zero() => {
                return Some(arg_var)
            }
            Opcode::Smul | Opcode::Umul if a.is_one() => return Some(arg_var),
            Opcode::Or if a.is_all_ones() => {
                return Some(unit.ins().const_int(IntValue::all_ones(width)))
            }
            Opcode::And if a.is_all_ones() => return Some(arg_var),
            Opcode::Xor if a.is_all_ones() => return Some(unit.ins().not(arg_var)),
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
                return Some(unit.ins().const_int(IntValue::zero(width)))
            }
            _ => (),
        }
    }

    // Try full constant folding.
    let (imm0, imm1) = (imm0?, imm1?);
    let result = None
        .or_else(|| IntValue::try_binary_op(op, imm0, imm1))
        .or_else(|| IntValue::try_compare_op(op, imm0, imm1))?;
    Some(unit.ins().const_int(result))
}

/// Fold a shift instruction.
fn fold_shift(unit: &mut UnitBuilder, inst: Inst, ty: &Type) -> Option<Value> {
    let base = unit[inst].args()[0];
    let hidden = unit[inst].args()[1];
    let amount = unit[inst].args()[2];

    let const_amount = unit.get_const_int(amount);
    let left = unit[inst].opcode() == Opcode::Shl;

    // Handle the trivial case where the shift amount is zero.
    if const_amount.map(IntValue::is_zero).unwrap_or(false) {
        return Some(base);
    }

    // Don't bother trying to optimize shifted signals and pointers.
    if unit.value_type(base).is_signal() || unit.value_type(base).is_pointer() {
        return None;
    }

    // Handle the case where the shift amount is constant.
    if let Some(amount) = const_amount {
        let amount = amount.to_usize();
        let base_width = unit.value_type(base).len();
        let hidden_width = unit.value_type(hidden).len();
        let amount = min(amount, hidden_width);
        trace!(
            "Fold const shift `{}` (amount: {}, base_width: {}, hidden_width: {})",
            inst.dump(&unit),
            amount,
            base_width,
            hidden_width
        );

        // Handle the case where the amount fully shifts out the base.
        if amount >= base_width {
            let offset = if left {
                hidden_width - amount
            } else {
                amount - base_width
            };
            trace!("  Base fully shifted out; hidden offset {}", offset);
            let r = unit.ins().ext_slice(hidden, offset, base_width);
            return Some(fold_ext_slice(unit, unit.value_inst(r)).unwrap_or(r));
        }
        // Handle the case where the result is a mixture of the base and the
        // hidden value.
        else {
            let (b, h, z0, z1, z2) = if left {
                let b = unit.ins().ext_slice(base, 0, base_width - amount);
                let h = unit.ins().ext_slice(hidden, hidden_width - amount, amount);
                let z0 = unit.ins().const_zero(ty);
                let z1 = unit.ins().ins_slice(z0, b, amount, base_width - amount);
                let z2 = unit.ins().ins_slice(z1, h, 0, amount);
                (b, h, z0, z1, z2)
            } else {
                let h = unit.ins().ext_slice(hidden, 0, amount);
                let b = unit.ins().ext_slice(base, amount, base_width - amount);
                let z0 = unit.ins().const_zero(ty);
                let z1 = unit.ins().ins_slice(z0, h, base_width - amount, amount);
                let z2 = unit.ins().ins_slice(z1, b, 0, base_width - amount);
                (b, h, z0, z1, z2)
            };
            run_on_value(unit, h);
            run_on_value(unit, b);
            run_on_value(unit, z0);
            run_on_value(unit, z1);
            return Some(fold_ins_slice(unit, unit.value_inst(z2)).unwrap_or(z2));
        }
    }

    None
}

/// Fold a slice insertion instruction.
fn fold_ins_slice(unit: &mut UnitBuilder, inst: Inst) -> Option<Value> {
    let data = &unit[inst];
    let target = data.args()[0];
    let value = data.args()[1];
    let len = data.imms()[1];

    // Handle the trivial cases where we override the entire value, or nothing
    // at all.
    match unit.value_type(target).as_ref() {
        IntType(_) | ArrayType(..) if len == 0 => return Some(target),
        IntType(w) | ArrayType(w, _) if len == *w => return Some(value),
        _ => (),
    }

    // Handle the case where both operands are constant integers.
    if let (Some(target), Some(value)) = (unit.get_const_int(target), unit.get_const_int(value)) {
        let mut r = target.clone();
        r.insert_slice(data.imms()[0], len, value);
        return Some(unit.ins().const_int(r));
    }

    None
}

/// Fold a slice extraction instruction.
fn fold_ext_slice(unit: &mut UnitBuilder, inst: Inst) -> Option<Value> {
    let data = &unit[inst];
    let ty = &unit.inst_type(inst);
    let target = data.args()[0];
    let len = data.imms()[1];

    // Handle the trivial case where we extract the entire value, or nothing
    // at all.
    match unit.value_type(target).as_ref() {
        IntType(..) | ArrayType(..) if len == 0 => return Some(unit.ins().const_zero(ty)),
        IntType(w) | ArrayType(w, _) if len == *w => return Some(target),
        _ => (),
    }

    // Handle the case where the target is a constant integer.
    if let Some(imm) = unit.get_const_int(target) {
        let r = imm.extract_slice(data.imms()[0], len);
        return Some(unit.ins().const_int(r));
    }

    None
}

/// Fold a field extraction instruction.
fn fold_ext_field(unit: &mut UnitBuilder, inst: Inst) -> Option<Value> {
    let data = &unit[inst];
    let target = data.args()[0];
    let target_inst = unit.get_value_inst(target)?;
    let target_data = &unit[target_inst];
    let offset = data.imms()[0];
    match target_data.opcode() {
        Opcode::ArrayUniform => Some(target_data.args()[0]),
        Opcode::Array | Opcode::Struct if offset < target_data.args().len() => {
            Some(target_data.args()[offset])
        }
        _ => None,
    }
}

/// Fold a mux instruction.
fn fold_mux(unit: &mut UnitBuilder, inst: Inst) -> Option<Value> {
    let choices = unit[inst].args()[0];
    let sel = unit[inst].args()[1];
    let const_sel = unit.get_const_int(sel)?.to_usize();
    Some(unit.ins().ext_field(choices, const_sel))
}
