// Copyright (c) 2017-2021 Fabian Schuiki

//! Process Lowering

use crate::{ir::prelude::*, opt::prelude::*};

/// Process Lowering
///
/// This pass implements lowering of suitable processes to entities.
pub struct ProcessLowering;

impl Pass for ProcessLowering {
    fn run_on_cfg(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        if !unit.is_process() || !is_suitable(ctx, &unit) {
            return false;
        }
        info!("ProcLower [{}]", unit.name());
        unit.data().kind = UnitKind::Entity;
        unit.delete_inst(unit.terminator(unit.entry()));
        unit.insert_at_end();
        unit.ins().halt();
        true
    }
}

/// Check if a process is suitable for lowering to an entity.
fn is_suitable(_ctx: &PassContext, unit: &Unit) -> bool {
    // Ensure that there is only one basic block.
    if unit.blocks().count() != 1 {
        trace!("Skipping {} (not just one block)", unit.name());
        return false;
    }
    let bb = unit.entry();

    // Ensure that the terminator instruction is a wait/halt.
    let term = unit.terminator(bb);
    match unit[term].opcode() {
        Opcode::Wait | Opcode::WaitTime | Opcode::Halt => (),
        op => {
            trace!("Skipping {} (wrong terminator {})", unit.name(), op);
            return false;
        }
    }

    // Ensure that all other instructions are allowed in an entity.
    for inst in unit.insts(bb) {
        if inst == term {
            continue;
        }
        if !unit[inst].opcode().valid_in_entity() {
            trace!(
                "Skipping {} ({} not allowed in entity)",
                unit.name(),
                inst.dump(&unit)
            );
            return false;
        }
    }

    // Ensure that all input arguments that are used are also contained in the
    // wait instruction's sensitivity list.
    match unit[term].opcode() {
        Opcode::Wait | Opcode::WaitTime => {
            for arg in unit.sig().inputs() {
                let value = unit.arg_value(arg);
                if unit.has_uses(value) && !unit[term].args().contains(&value) {
                    trace!(
                        "Skipping {} ({} not in wait sensitivity list)",
                        unit.name(),
                        value.dump(&unit)
                    );
                    return false;
                }
            }
        }
        _ => (),
    }

    true
}
