// Copyright (c) 2017-2021 Fabian Schuiki

//! Temporal Code Motion

use crate::{
    analysis::{DominatorTree, TemporalRegion, TemporalRegionGraph},
    ir::prelude::*,
    ir::InstData,
    opt::prelude::*,
    value::IntValue,
};
use itertools::Itertools;
use std::collections::HashMap;

/// Temporal Code Motion
///
/// This pass rearranges temporal instructions. It does the following:
///
/// - Merge multiple identical waits into one (in a new block).
/// - Move `prb` instructions up to the top of the time region.
/// - Move `drv` instructions down to the end of the time region, where
///   possible. Failure to do so hints at conditionally-driven signals, such as
///   storage elements.
///
pub struct TemporalCodeMotion;

impl Pass for TemporalCodeMotion {
    fn run_on_cfg(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("TCM [{}]", unit.name());
        let mut modified = false;

        // Build the temporal region graph.
        let trg = unit.trg();

        // Hoist `prb` instructions which directly operate on input signals to
        // the head block of their region.
        // TODO: Move this into the `ECM` pass.
        let temp_dt = unit.temporal_domtree();
        for tr in trg.regions() {
            if tr.head_blocks.len() != 1 {
                trace!("Skipping {} for prb move (multiple head blocks)", tr.id);
                continue;
            }
            let head_bb = tr.head_blocks().next().unwrap();
            let mut hoist = vec![];
            for bb in tr.blocks() {
                for inst in unit.insts(bb) {
                    if unit[inst].opcode() == Opcode::Prb
                        && unit.get_value_inst(unit[inst].args()[0]).is_none()
                    {
                        // Check if the new prb location would dominate its old
                        // location temporally.
                        let mut dominates = temp_dt.dominates(head_bb, bb);

                        // Only move when the move instruction would still
                        // dominate all its uses.
                        for &user_inst in unit.uses(unit.inst_result(inst)) {
                            let user_bb = unit.inst_block(user_inst).unwrap();
                            let dom = temp_dt.dominates(head_bb, user_bb);
                            dominates &= dom;
                        }
                        if dominates {
                            hoist.push(inst);
                        } else {
                            trace!(
                                "Skipping {} for prb move (would not dominate uses)",
                                inst.dump(&unit)
                            );
                        }
                    }
                }
            }
            hoist.sort();
            for inst in hoist {
                if unit.inst_block(inst) == Some(head_bb) {
                    continue;
                }
                debug!("Hoisting {} into {}", inst.dump(&unit), head_bb.dump(&unit));
                unit.remove_inst(inst);
                unit.prepend_inst(inst, head_bb);
                modified = true;
            }
        }

        // Fuse equivalent wait instructions.
        let trg = unit.trg();
        for tr in trg.regions() {
            if tr.tail_insts.len() <= 1 {
                trace!("Skipping {} for wait merge (single wait inst)", tr.id);
                continue;
            }
            let mut merge = HashMap::<&InstData, Vec<Inst>>::new();
            for inst in tr.tail_insts() {
                merge.entry(&unit[inst]).or_default().push(inst);
            }
            let merge: Vec<_> = merge.into_iter().map(|(_, is)| is).collect();
            for insts in merge {
                if insts.len() <= 1 {
                    trace!("Skipping {} (no equivalents)", insts[0].dump(&unit));
                    continue;
                }
                trace!("Merging:",);
                for i in &insts {
                    trace!("  {}", i.dump(&unit));
                }

                // Create a new basic block for the singleton wait inst.
                let unified_bb = unit.block();

                // Replace all waits with branches into the unified block.
                for &inst in &insts {
                    unit.insert_after(inst);
                    unit.ins().br(unified_bb);
                }

                // Add one of the instructions to the unified block and delete
                // the rest.
                unit.remove_inst(insts[0]);
                unit.append_inst(insts[0], unified_bb);
                for &inst in &insts[1..] {
                    unit.delete_inst(inst);
                }
                modified = true;
            }
        }

        // Introduce auxiliary exit blocks if multiple edges leave a temporal
        // region into the same target block in a different region. This is
        // needed to ensure that drives have a dedicated block to be pushed
        // down into ahead of the next temporal region.
        modified |= add_aux_blocks(ctx, unit);

        // Push `drv` instructions towards the tails of their temporal regions.
        modified |= push_drives(ctx, unit);

        // TODO: Coalesce drives to the same signal.

        modified
    }
}

/// Introduce auxiliary exit blocks if multiple edges leave a temporal region
/// into the same target block in a different region. This is needed to ensure
/// that drives have a dedicated block to be pushed down into ahead of the next
/// temporal region.
fn add_aux_blocks(_ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
    let pt = unit.predtbl();
    let trg = unit.trg();
    let mut modified = false;

    // Make a list of head blocks. This will allow us to change the unit
    // underneath.
    let head_bbs: Vec<_> = unit.blocks().filter(|&bb| trg.is_head(bb)).collect();

    // Process each block separately.
    for bb in head_bbs {
        trace!("Adding aux blocks into {}", bb.dump(&unit));
        let tr = trg[bb];

        // Gather a list of predecessor instructions per region, which branch
        // into this block.
        let mut insts_by_region = HashMap::<TemporalRegion, Vec<Inst>>::new();
        for pred in pt.pred(bb) {
            let pred_tr = trg[pred];
            if pred_tr != tr {
                let inst = unit.terminator(pred);
                insts_by_region.entry(pred_tr).or_default().push(inst);
            }
        }

        // For each entry with more than one instruction, create an auxiliary
        // entry block.
        for (src_tr, insts) in insts_by_region {
            if insts.len() < 2 {
                trace!("  Skipping {} (single head inst)", src_tr);
                continue;
            }
            let aux_bb = unit.named_block("aux");
            unit.append_to(aux_bb);
            unit.ins().br(bb);
            trace!("  Adding {} from {}", aux_bb.dump(&unit), src_tr);
            for inst in insts {
                trace!("    Replacing {} in {}", bb.dump(&unit), inst.dump(&unit));
                unit.replace_block_within_inst(bb, aux_bb, inst);
            }
            modified = true;
        }
    }

    modified
}

/// Push `drv` instructions downards into the tails of their temporal regions.
fn push_drives(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
    let mut modified = false;

    // We need the dominator tree of the current CFG.
    let pt = unit.predtbl();
    let dt = unit.domtree_with_predtbl(&pt);

    // Build an alias table of all signals, which indicates which signals are
    // aliases (e.g. extf/exts) of another. As we encounter drives, keep track
    // of their sequential dependency.
    let mut aliases = HashMap::<Value, Value>::new();
    let mut drv_seq = HashMap::<Value, Vec<Inst>>::new();
    for &bb in dt.blocks_post_order().iter().rev() {
        trace!("Checking {} for aliases", bb.dump(&unit));
        for inst in unit.insts(bb) {
            let data = &unit[inst];
            if let Opcode::Drv | Opcode::DrvCond = data.opcode() {
                // Gather drive sequences to the same signal.
                let signal = data.args()[0];
                let signal = aliases.get(&signal).cloned().unwrap_or(signal);
                trace!("  Drive {} ({})", signal.dump(&unit), inst.dump(&unit));
                drv_seq.entry(signal).or_default().push(inst);
            } else if let Some(value) = unit.get_inst_result(inst) {
                // Gather signal aliases.
                if !unit.value_type(value).is_signal() {
                    continue;
                }
                for &arg in data.args() {
                    if !unit.value_type(arg).is_signal() {
                        continue;
                    }
                    let arg = aliases.get(&arg).cloned().unwrap_or(arg);
                    trace!(
                        "  Alias {} of {} ({})",
                        value.dump(&unit),
                        arg.dump(&unit),
                        inst.dump(&unit)
                    );
                    aliases.insert(value, arg);
                }
            }
        }
    }

    // Build the temporal region graph.
    let trg = unit.trg();

    // Try to migrate drive instructions into the tails of their respective
    // temporal regions.
    for (&signal, drives) in &drv_seq {
        trace!("Moving drives on signal {}", signal.dump(&unit));
        // TODO: Don't directly move drives, but track if move is possible and what
        // the conditions are. Then do post-processing down below.
        for &drive in drives.iter().rev() {
            // Skip drives that are already in the right place.
            let drive_bb = unit.inst_block(drive).unwrap();
            if trg.is_tail(drive_bb) {
                trace!("  Skipping {} (already in tail block)", drive.dump(&unit),);
                continue;
            }
            if trg[trg[drive_bb]].tail_blocks.is_empty() {
                trace!("  Skipping {} (no tail blocks)", drive.dump(&unit),);
                continue;
            }

            // Perform the move.
            // trace!("  Checking {}", drive.dump(&unit));
            let moved = push_drive(ctx, drive, unit, &dt, &trg);
            modified |= moved;

            // If the move was not possible, abort all other drives since we
            // cannot move over them.
            if !moved {
                break;
            }
        }
    }

    // Coalesce drives. We do this one aliasing group at a time.
    for block in unit.blocks().collect::<Vec<_>>() {
        modified |= coalesce_drives(ctx, block, unit);
    }

    modified
}

fn push_drive(
    _ctx: &PassContext,
    drive: Inst,
    unit: &mut UnitBuilder,
    dt: &DominatorTree,
    trg: &TemporalRegionGraph,
) -> bool {
    let src_bb = unit.inst_block(drive).unwrap();
    let tr = trg[src_bb];
    let mut moves = Vec::new();

    // For each tail block that this drive moves to, find the branch conditions
    // along the way that lead to the drive being executed, and check if the
    // arguments for the drive are available in the destination block.
    for dst_bb in trg[tr].tail_blocks() {
        // trace!("    Will have to move to {}", dst_bb.dump(&unit));

        // First check if all arguments of the drive instruction dominate the
        // destination block. If not, the move is not possible.
        for &arg in unit[drive].args() {
            if !dt.value_dominates_block(unit, arg, dst_bb) {
                trace!(
                    "  Skipping {} ({} does not dominate {})",
                    drive.dump(&unit),
                    arg.dump(&unit),
                    dst_bb.dump(&unit)
                );
                return false;
            }
        }

        // Find the branch conditions that lead to src_bb being executed on the
        // way to dst_bb. We do this by stepping up the dominator tree until we
        // find the common dominator. For every step of src_bb, we record the
        // branch condition.
        let mut src_finger = src_bb;
        let mut dst_finger = dst_bb;
        let mut conds = Vec::<(Value, bool)>::new();
        while src_finger != dst_finger {
            let i1 = dt.block_order(src_finger);
            let i2 = dt.block_order(dst_finger);
            if i1 < i2 {
                let parent = dt.dominator(src_finger);
                if src_finger == parent {
                    break;
                }
                let term = unit.terminator(parent);
                if unit[term].opcode() == Opcode::BrCond {
                    let cond_val = unit[term].args()[0];
                    if !dt.value_dominates_block(unit, cond_val, dst_bb) {
                        trace!(
                            "  Skipping {} (branch cond {} does not dominate {})",
                            drive.dump(&unit),
                            cond_val.dump(&unit),
                            dst_bb.dump(&unit)
                        );
                        return false;
                    }
                    let cond_pol = unit[term].blocks().iter().position(|&bb| bb == src_finger);
                    if let Some(cond_pol) = cond_pol {
                        conds.push((cond_val, cond_pol != 0));
                        trace!(
                            "    {} -> {} ({} == {})",
                            parent.dump(&unit),
                            src_finger.dump(&unit),
                            cond_val.dump(&unit),
                            cond_pol
                        );
                    }
                } else {
                    trace!("    {} -> {}", parent.dump(&unit), src_finger.dump(&unit));
                }
                src_finger = parent;
            } else if i2 < i1 {
                let parent = dt.dominator(dst_finger);
                if dst_finger == parent {
                    break;
                }
                dst_finger = parent;
            }
        }
        if src_finger != dst_finger {
            trace!("  Skipping {} (no common dominator)", drive.dump(&unit));
            return false;
        }

        // Keep note of this destination block and the conditions that must
        // hold.
        moves.push((dst_bb, conds));
    }

    // If we arrive here, all moves are possible and can now be executed.
    for (dst_bb, conds) in moves {
        debug!("Moving {} to {}", drive.dump(&unit), dst_bb.dump(&unit));

        // Start by assembling the drive condition in the destination block. The
        // order is key here to allow for easy constant folding and subexpr
        // elimination: the conditions are in reverse CFG order, so and them
        // together in reverse order to reflect the CFG, which allows for most
        // of these conditions to be shared.
        unit.prepend_to(dst_bb);
        let mut cond = unit.ins().const_int(IntValue::all_ones(1));
        for (value, polarity) in conds.into_iter().rev() {
            let value = match polarity {
                true => value,
                false => unit.ins().not(value),
            };
            cond = unit.ins().and(cond, value);
        }

        // Add the drive condition, if any.
        if unit[drive].opcode() == Opcode::DrvCond {
            let arg = unit[drive].args()[3];
            cond = unit.ins().and(cond, arg);
        }

        // Insert the new drive.
        let args = unit[drive].args();
        let signal = args[0];
        let value = args[1];
        let delay = args[2];
        unit.ins().drv_cond(signal, value, delay, cond);
    }

    // Remove the old drive instruction.
    unit.delete_inst(drive);

    true
}

fn coalesce_drives(_ctx: &PassContext, block: Block, unit: &mut UnitBuilder) -> bool {
    let mut modified = false;

    // Group the drives by delay.
    let mut delay_groups = HashMap::<Value, Vec<Inst>>::new();
    for inst in unit.insts(block) {
        if let Opcode::Drv | Opcode::DrvCond = unit[inst].opcode() {
            let delay = unit[inst].args()[2];
            delay_groups.entry(delay).or_default().push(inst);
        }
    }

    // Coalesce each delay group individually. Split the instructions into runs
    // of drives to the exact same signal.
    for (delay, drives) in delay_groups {
        let runs: Vec<_> = drives
            .into_iter()
            .group_by(|&inst| unit[inst].args()[0])
            .into_iter()
            .map(|(target, drives)| (target, drives.collect::<Vec<_>>()))
            .collect();
        for (target, drives) in runs {
            if drives.len() <= 1 {
                continue;
            }
            debug!(
                "Coalescing {} drives on {}",
                drives.len(),
                target.dump(&unit)
            );
            let mut drives = drives.into_iter();

            // Get the first drive's value and condition, and remove the drive.
            let first = drives.next().unwrap();
            unit.insert_before(first);
            let mut cond = drive_cond(unit, first);
            let mut value = unit[first].args()[1];
            unit.delete_inst(first);

            // Accumulate subsequent drive conditions and values, and remove.
            for drive in drives {
                unit.insert_before(drive);
                let c = drive_cond(unit, drive);
                let v = unit[drive].args()[1];
                if cond != c {
                    cond = unit.ins().or(cond, c);
                }
                if value != v {
                    let vs = unit.ins().array(vec![value, v]);
                    value = unit.ins().mux(vs, c);
                }
                unit.delete_inst(drive);
            }

            // Build the final drive.
            unit.ins().drv_cond(target, value, delay, cond);
            modified = true;
        }
    }

    // TODO: Collapse drives with the same value and delay.
    // TODO: Build discriminator table for all drives, then build corresponding
    // mux to select driven value, and use or of all drive conditions as new
    // drive condition.

    modified
}

fn drive_cond(unit: &mut UnitBuilder, inst: Inst) -> Value {
    if unit[inst].opcode() == Opcode::DrvCond {
        unit[inst].args()[3]
    } else {
        unit.ins().const_int(IntValue::all_ones(1))
    }
}
