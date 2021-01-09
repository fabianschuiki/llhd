// Copyright (c) 2017-2021 Fabian Schuiki

//! Early Code Motion

use crate::{analysis::DominatorTree, ir::prelude::*, opt::prelude::*};
use std::collections::{HashMap, HashSet};

/// Early Code Motion
///
/// This moves all instructions as far upwards in the control flow graph as
/// possible given the point of declaration of their arguments.
pub struct EarlyCodeMotion;

impl Pass for EarlyCodeMotion {
    fn run_on_cfg(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("ECM [{}]", unit.name());
        let mut modified = false;

        // Build the predecessor table and dominator tree.
        let pred = unit.predtbl();
        let dt = unit.domtree_with_predtbl(&pred);

        // Create a work queue which allows us to process the blocks in control
        // flow order. Also number the blocks as we go.
        let mut block_numbers = HashMap::<Block, usize>::new();
        let mut work_done = HashSet::<Block>::new();
        let mut work_pending = HashSet::<Block>::new();
        let entry = unit.entry();
        work_pending.insert(entry);
        block_numbers.insert(entry, 0);

        while let Some(&block) = work_pending.iter().next() {
            work_pending.remove(&block);
            work_done.insert(block);
            trace!("Working on {}", block.dump(&unit));

            // Process the instructions in this block.
            for inst in unit.insts(block).collect::<Vec<_>>() {
                modified |= move_instruction(ctx, unit, block, inst, &dt, &block_numbers);
            }

            // Work on the successors of this block.
            let term = unit.terminator(block);
            if unit[term].opcode().is_terminator() {
                work_pending.extend(
                    unit[term]
                        .blocks()
                        .iter()
                        .cloned()
                        .filter(|bb| !work_done.contains(bb)),
                );
                let next_number = block_numbers[&block] + 1;
                for bb in unit[term].blocks().iter().cloned() {
                    block_numbers.entry(bb).or_insert(next_number);
                }
            }
        }

        trace!("Final block numbers:");
        for (bb, num) in block_numbers {
            trace!("  {} = {}", bb.dump(&unit), num);
        }

        modified
    }
}

fn move_instruction(
    _ctx: &PassContext,
    unit: &mut UnitBuilder,
    block: Block,
    inst: Inst,
    dt: &DominatorTree,
    block_numbers: &HashMap<Block, usize>,
) -> bool {
    // Some instructions we don't want to touch.
    let op = unit[inst].opcode();
    if op == Opcode::Ld
        || op == Opcode::St
        || op == Opcode::Prb
        || op == Opcode::Drv
        || op == Opcode::Phi
        || op.is_terminator()
    {
        return false;
    }
    trace!("  Working on {}", inst.dump(&unit));

    // To determine the possible insertion locations, we first need to find for
    // each argument of this instruction, which blocks its definition dominates.
    let doms: Vec<_> = unit[inst]
        .args()
        .iter()
        .flat_map(|&arg| unit.get_value_inst(arg))
        .map(|inst| unit.inst_block(inst).unwrap())
        .map(|block| dt.dominated_by(block))
        .collect();

    // If the instruction depends on nothing (e.g. constants), move them up into
    // the entry block.
    if doms.is_empty() {
        let entry = unit.entry();
        let entry_term = unit.terminator(entry);
        if unit.inst_block(inst) == Some(entry) {
            return false;
        }
        unit.remove_inst(inst);
        unit.insert_inst_before(inst, entry_term);
        debug!("Move {} into {}", inst.dump(&unit), entry.dump(&unit));
        return true;
    }
    // trace!("    Dominated blocks: {:?}", doms);

    // Find the blocks that are present in all the domination sets. These are
    // the blocks where the current instruction can safely be moved and still
    // "see" all its arguments.
    let possible_bbs = doms
        .get(0)
        .into_iter()
        .flat_map(|bbs| bbs.iter())
        .filter(|bb| doms.iter().all(|bbs| bbs.contains(bb)))
        .cloned();

    // Decide on the best block to move the instruction to. This is given as the
    // block with the lowest number, which is the lowest number of control flow
    // edges from the entry block.
    let best_bb = possible_bbs
        .flat_map(|bb| block_numbers.get(&bb).map(|&num| (bb, num)))
        .min_by_key(|&(_, num)| num)
        .map(|(bb, _)| bb);
    let best_bb = match best_bb {
        Some(bb) => bb,
        None => return false,
    };
    trace!("    Best block: {}", best_bb.dump(&unit));

    // Move the instruction up into this block.
    if best_bb == block {
        return false;
    }
    debug!("Move {} into {}", inst.dump(&unit), best_bb.dump(&unit));
    let term = unit.terminator(best_bb);
    unit.remove_inst(inst);
    unit.insert_inst_before(inst, term);
    true
}
