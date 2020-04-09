// Copyright (c) 2017-2020 Fabian Schuiki

//! Process Lowering

use crate::ir::prelude::*;
use crate::ir::ModUnitData;
use crate::opt::prelude::*;
use rayon::prelude::*;

/// Process Lowering
///
/// This pass implements lowering of suitable processes to entities.
pub struct ProcessLowering;

impl Pass for ProcessLowering {
    fn run_on_module(ctx: &PassContext, module: &mut Module) -> bool {
        info!("ProcLower");
        module
            .units
            .storage
            .par_iter_mut()
            .map(|(_, unit)| lower_mod_unit(ctx, unit))
            .reduce(|| false, |a, b| a || b)
    }
}

fn lower_mod_unit(ctx: &PassContext, mod_unit: &mut ModUnitData) -> bool {
    // Check if this is a process and it is suitable for lowering.
    let process = match mod_unit {
        ModUnitData::Process(ref mut u) => {
            if !is_suitable(ctx, &mut ProcessBuilder::new(u)) {
                return false;
            }
            std::mem::replace(u, Process::new(u.name().clone(), u.sig().clone()))
        }
        _ => return false,
    };

    // Lower the process to an entity.
    trace!("Lowering {} to an entity", process.name());
    let term = process.layout.terminator(process.layout.entry());
    let mut entity = Entity {
        dfg: process.dfg,
        cfg: process.cfg,
        layout: process.layout,
        name: process.name,
        sig: process.sig,
    };
    EntityBuilder::new(&mut entity).remove_inst(term);
    *mod_unit = ModUnitData::Entity(entity);

    true
}

/// Check if a process is suitable for lowering to an entity.
fn is_suitable(_ctx: &PassContext, ub: &mut ProcessBuilder) -> bool {
    let dfg = ub.dfg();
    let layout = ub.func_layout();

    // Ensure that there is only one basic block.
    if layout.blocks().count() != 1 {
        trace!("Skipping {} (not just one block)", ub.unit().name());
        return false;
    }
    let bb = layout.entry();

    // Ensure that the terminator instruction is a wait/halt.
    let term = layout.terminator(bb);
    match ub[term].opcode() {
        Opcode::Wait | Opcode::WaitTime | Opcode::Halt => (),
        op => {
            trace!("Skipping {} (wrong terminator {})", ub.unit().name(), op);
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
                ub.unit().name(),
                inst.dump(ub.dfg(), ub.try_cfg())
            );
            return false;
        }
    }

    // Ensure that all input arguments that are used are also contained in the
    // wait instruction's sensitivity list.
    match ub[term].opcode() {
        Opcode::Wait | Opcode::WaitTime => {
            for arg in ub.unit().sig().inputs() {
                let value = ub.unit().arg_value(arg);
                if ub.unit().has_uses(value) && !ub[term].args().contains(&value) {
                    trace!(
                        "Skipping {} ({} not in wait sensitivity list)",
                        ub.unit().name(),
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
