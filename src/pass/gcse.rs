// Copyright (c) 2017-2021 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::{
    ir::{prelude::*, InstData},
    opt::prelude::*,
};
use std::collections::{HashMap, HashSet};

/// Global Common Subexpression Elimination
///
/// This pass implements global common subexpression elimination. It tries to
/// eliminate redundant instructions.
pub struct GlobalCommonSubexprElim;

impl Pass for GlobalCommonSubexprElim {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("GCSE [{}]", unit.name());

        // Build the predecessor table and dominator tree.
        let pred = unit.predtbl();
        let dt = unit.domtree_with_predtbl(&pred);

        // Build the temporal predecessor table and dominator tree.
        let temp_pt = unit.temporal_predtbl();
        let temp_dt = unit.domtree_with_predtbl(&temp_pt);

        // Compute the TRG to allow for `prb` instructions to be eliminated.
        let trg = unit.trg();

        // Collect instructions.
        let mut insts = vec![];
        for bb in unit.blocks() {
            for inst in unit.insts(bb) {
                insts.push(inst);
            }
        }

        // Perform GCSE.
        let mut modified = false;
        let mut values = HashMap::<InstData, HashSet<Value>>::new();
        'outer: for inst in insts {
            // Don't mess with instructions that produce no result or have side
            // effects.
            let opcode = unit[inst].opcode();
            if !unit.has_result(inst)
                || opcode == Opcode::Ld
                || opcode == Opcode::Var
                || opcode == Opcode::Sig
            {
                continue;
            }
            let value = unit.inst_result(inst);
            trace!("Examining {}", inst.dump(&unit));

            // Try the candidates.
            if let Some(aliases) = values.get_mut(&unit[inst]) {
                'inner: for &cv in aliases.iter() {
                    trace!("  Trying {}", cv.dump(&unit));
                    let cv_inst = unit.value_inst(cv);
                    let inst_bb = unit.inst_block(inst).unwrap();
                    let cv_bb = unit.inst_block(cv_inst).unwrap();

                    // Make sure that we don't merge `prb` instructions in
                    // different temporal regions.
                    if trg[inst_bb] != trg[cv_bb] {
                        trace!("    Skipping because in other temporal region");
                        continue;
                    }

                    // Decide which dominator tree to use.
                    let which_dt = if opcode == Opcode::Prb { &temp_dt } else { &dt };

                    // Replace the current inst with the recorded value if the
                    // latter dominates the former.
                    if which_dt.dominates(cv_bb, inst_bb) {
                        debug!("Replace {} with {}", inst.dump(&unit), cv.dump(&unit),);
                        unit.replace_use(value, cv);
                        unit.prune_if_unused(inst);
                        modified = true;
                        continue 'outer;
                    }

                    // Replace the recorded value with the current inst if the
                    // latter dominates the former.
                    if which_dt.dominates(inst_bb, cv_bb) {
                        debug!("Replace {} with {}", cv.dump(&unit), value.dump(&unit),);
                        unit.replace_use(cv, value);
                        unit.prune_if_unused(cv_inst);
                        aliases.remove(&cv); // crazy that this works; NLL <3
                        modified = true;
                        break 'inner;
                    }

                    // Try to merge the current inst with the recorded value by
                    // hoisting the instruction up to an earlier BB. We know
                    // know that such a BB exists because being in this loop
                    // body means that both instructions have the same args,
                    // which is only possible if they are both dominated by the
                    // args.

                    trace!(
                        "    Intersect(Dom({}), Dom({})):",
                        inst_bb.dump(&unit),
                        cv_bb.dump(&unit)
                    );
                    for bb in which_dt
                        .dominators(inst_bb)
                        .intersection(&which_dt.dominators(cv_bb))
                    {
                        trace!("      {}", bb.dump(&unit));
                    }
                    let target_bb = which_dt
                        .dominators(inst_bb)
                        .intersection(which_dt.dominators(cv_bb))
                        .max_by(|&&bb_a, &&bb_b| {
                            if which_dt.dominates(bb_a, bb_b) {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        });
                    let target_bb = match target_bb {
                        Some(&bb) => bb,
                        None => continue,
                    };
                    trace!("    Latest common dominator: {}", target_bb.dump(&unit));

                    // Hoist the instruction up into the target block.
                    debug!(
                        "Hoist {} up into {}",
                        inst.dump(&unit),
                        target_bb.dump(&unit)
                    );
                    let term = unit.terminator(target_bb);
                    unit.remove_inst(inst);
                    unit.insert_inst_before(inst, term);

                    // Replace all uses of the recorded value with the inst.
                    debug!("Replace {} with {}", cv.dump(&unit), value.dump(&unit),);
                    unit.replace_use(cv, value);
                    unit.prune_if_unused(cv_inst);
                    aliases.remove(&cv); // crazy that this works; NLL <3
                    modified = true;
                    break 'inner;
                }
            }

            // Insert the instruction into the table.
            // trace!("Recording {}", inst.dump(&unit));
            values
                .entry(unit[inst].clone())
                .or_insert_with(Default::default)
                .insert(value);
        }
        modified
    }
}
