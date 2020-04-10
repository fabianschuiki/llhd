// Copyright (c) 2017-2020 Fabian Schuiki

//! Process Lowering

use crate::{ir::prelude::*, opt::prelude::*};
use rayon::prelude::*;

/// Process Lowering
///
/// This pass implements lowering of suitable processes to entities.
pub struct ProcessLowering;

impl Pass for ProcessLowering {
    fn run_on_module(ctx: &PassContext, module: &mut Module) -> bool {
        info!("ProcLower");
        module
            .par_units_mut()
            .map(|mut unit| lower_unit(ctx, &mut unit))
            .reduce(|| false, |a, b| a || b)
    }
}

fn lower_unit(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
    if !unit.is_process() || !is_suitable(ctx, unit.unit()) {
        return false;
    }
    let data = UnitData::new(UnitKind::Process, unit.name().clone(), unit.sig().clone());
    let process = std::mem::replace(unit.data(), data);

    // Lower the process to an entity.
    trace!("Lowering {} to an entity", process.name);
    let term = process.layout.terminator(process.layout.entry());
    let mut entity = UnitData {
        kind: UnitKind::Entity,
        dfg: process.dfg,
        cfg: process.cfg,
        layout: process.layout,
        name: process.name,
        sig: process.sig,
    };
    UnitBuilder::new_anonymous(&mut entity).remove_inst(term);
    *unit.data() = entity;

    true
}

/// Check if a process is suitable for lowering to an entity.
fn is_suitable(_ctx: &PassContext, ub: Unit) -> bool {
    let dfg = ub.dfg();
    let layout = ub.func_layout();

    // Ensure that there is only one basic block.
    if layout.blocks().count() != 1 {
        trace!("Skipping {} (not just one block)", ub.name());
        return false;
    }
    let bb = layout.entry();

    // Ensure that the terminator instruction is a wait/halt.
    let term = layout.terminator(bb);
    match ub[term].opcode() {
        Opcode::Wait | Opcode::WaitTime | Opcode::Halt => (),
        op => {
            trace!("Skipping {} (wrong terminator {})", ub.name(), op);
            return false;
        }
    }

    // Ensure that all other instructions are allowed in an entity.
    for inst in layout.insts(bb) {
        if inst == term {
            continue;
        }
        if !ub[inst].opcode().valid_in_entity() {
            trace!(
                "Skipping {} ({} not allowed in entity)",
                ub.name(),
                inst.dump(ub.dfg(), ub.try_cfg())
            );
            return false;
        }
    }

    // Ensure that all input arguments that are used are also contained in the
    // wait instruction's sensitivity list.
    match ub[term].opcode() {
        Opcode::Wait | Opcode::WaitTime => {
            for arg in ub.sig().inputs() {
                let value = ub.arg_value(arg);
                if ub.has_uses(value) && !ub[term].args().contains(&value) {
                    trace!(
                        "Skipping {} ({} not in wait sensitivity list)",
                        ub.name(),
                        value.dump(dfg)
                    );
                    return false;
                }
            }
        }
        _ => (),
    }

    true
}
