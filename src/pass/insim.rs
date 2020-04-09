// Copyright (c) 2017-2020 Fabian Schuiki

//! Instruction Simplification

use crate::ir::prelude::*;
use crate::opt::prelude::*;

/// Instruction Simplification
///
/// This pass implements various instruction combinations and simplifications.
pub struct InstSimplification;

impl Pass for InstSimplification {
    fn run_on_inst(ctx: &PassContext, inst: Inst, ub: &mut impl UnitBuilder) -> bool {
        ub.insert_after(inst);
        match ub[inst].opcode() {
            // drv ... if 0 -> removed
            // drv ... if 1 -> drv ...
            Opcode::DrvCond => {
                if let Some(konst) = ub.unit().get_const_int(ub[inst].args()[3]) {
                    if konst.is_one() {
                        let signal = ub[inst].args()[0];
                        let value = ub[inst].args()[1];
                        let delay = ub[inst].args()[2];
                        ub.ins().drv(signal, value, delay);
                    }
                    ub.remove_inst(inst);
                }
            }
            _ => (),
        }
        let value = match ub.unit().get_inst_result(inst) {
            Some(value) => value,
            None => return false,
        };
        match ub[inst].opcode() {
            // and %a, %a -> %a
            // or %a, %a -> %a
            Opcode::And | Opcode::Or if ub[inst].args()[0] == ub[inst].args()[1] => {
                replace(inst, value, ub[inst].args()[0], ub)
            }
            // xor %a, %a -> 0
            // [us]rem %a, %a -> 0
            // [us]mod %a, %a -> 0
            Opcode::Xor | Opcode::Umod | Opcode::Urem | Opcode::Smod | Opcode::Srem
                if ub[inst].args()[0] == ub[inst].args()[1] =>
            {
                let ty = ub.unit().value_type(value);
                let zero = ub.ins().const_zero(&ty);
                replace(inst, value, zero, ub)
            }
            Opcode::Mux => simplify_mux(ctx, inst, value, ub),
            _ => false,
        }
    }
}

fn replace(from_inst: Inst, from_value: Value, to: Value, ub: &mut impl UnitBuilder) -> bool {
    debug!(
        "Replace {} with {}",
        from_inst.dump(ub.dfg(), ub.try_cfg()),
        to.dump(ub.dfg())
    );
    ub.replace_use(from_value, to) > 0
}

fn simplify_mux(_ctx: &PassContext, inst: Inst, value: Value, ub: &mut impl UnitBuilder) -> bool {
    let dfg = ub.dfg();

    // Check if all options are identical, in which case simply replace us with
    // the option directly.
    let array = dfg[inst].args()[0];
    if let Some(array_inst) = ub.unit().get_value_inst(array) {
        let mut iter = dfg[array_inst].args().iter().cloned();
        let first = match iter.next() {
            Some(first) => first,
            None => return false,
        };
        let identical = iter.all(|a| a == first);
        if identical {
            return replace(inst, value, first, ub);
        }
    }

    false
}
