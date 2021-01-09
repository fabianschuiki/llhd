// Copyright (c) 2017-2021 Fabian Schuiki

//! Instruction Simplification

use crate::ir::prelude::*;
use crate::opt::prelude::*;

/// Instruction Simplification
///
/// This pass implements various instruction combinations and simplifications.
pub struct InstSimplification;

impl Pass for InstSimplification {
    fn run_on_inst(ctx: &PassContext, inst: Inst, unit: &mut UnitBuilder) -> bool {
        unit.insert_after(inst);
        match unit[inst].opcode() {
            // drv ... if 0 -> removed
            // drv ... if 1 -> drv ...
            Opcode::DrvCond => {
                if let Some(konst) = unit.get_const_int(unit[inst].args()[3]) {
                    if konst.is_one() {
                        let signal = unit[inst].args()[0];
                        let value = unit[inst].args()[1];
                        let delay = unit[inst].args()[2];
                        unit.ins().drv(signal, value, delay);
                    }
                    unit.delete_inst(inst);
                }
            }
            _ => (),
        }
        let value = match unit.get_inst_result(inst) {
            Some(value) => value,
            None => return false,
        };
        match unit[inst].opcode() {
            // and %a, %a -> %a
            // or %a, %a -> %a
            Opcode::And | Opcode::Or if unit[inst].args()[0] == unit[inst].args()[1] => {
                replace(inst, value, unit[inst].args()[0], unit)
            }
            // xor %a, %a -> 0
            // [us]rem %a, %a -> 0
            // [us]mod %a, %a -> 0
            Opcode::Xor | Opcode::Umod | Opcode::Urem | Opcode::Smod | Opcode::Srem
                if unit[inst].args()[0] == unit[inst].args()[1] =>
            {
                let ty = unit.value_type(value);
                let zero = unit.ins().const_zero(&ty);
                replace(inst, value, zero, unit)
            }
            Opcode::Mux => simplify_mux(ctx, inst, value, unit),
            _ => false,
        }
    }
}

fn replace(from_inst: Inst, from_value: Value, to: Value, unit: &mut UnitBuilder) -> bool {
    debug!("Replace {} with {}", from_inst.dump(&unit), to.dump(&unit));
    unit.replace_use(from_value, to) > 0
}

fn simplify_mux(_ctx: &PassContext, inst: Inst, value: Value, unit: &mut UnitBuilder) -> bool {
    // Check if all options are identical, in which case simply replace us with
    // the option directly.
    let array = unit[inst].args()[0];
    if let Some(array_inst) = unit.get_value_inst(array) {
        let mut iter = unit[array_inst].args().iter().cloned();
        let first = match iter.next() {
            Some(first) => first,
            None => return false,
        };
        let identical = iter.all(|a| a == first);
        if identical {
            return replace(inst, value, first, unit);
        }
    }

    false
}
