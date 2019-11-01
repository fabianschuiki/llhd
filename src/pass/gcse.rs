// Copyright (c) 2017-2019 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::ir::prelude::*;
use crate::ir::InstData;
use crate::opt::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    iter::once,
};

/// Global Common Subexpression Elimination
///
/// This pass implements global common subexpression elimination. It tries to
/// eliminate redundant instructions.
pub struct GlobalCommonSubexprElim;

impl Pass for GlobalCommonSubexprElim {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        // Build the predecessor table.
        let mut pred = HashMap::<Block, HashSet<Block>>::new();
        for bb in unit.func_layout().blocks() {
            pred.insert(bb, Default::default());
        }
        for bb in unit.func_layout().blocks() {
            let term = unit.func_layout().terminator(bb);
            for to_bb in unit.dfg()[term].blocks() {
                pred.get_mut(&to_bb).unwrap().insert(bb);
            }
        }
        trace!("Predecessor table: {:?}", pred);

        // Build the dominator tree.
        trace!("Building dominator tree");

        let all_blocks: HashSet<Block> = unit.func_layout().blocks().collect();
        let mut dom = HashMap::<Block, HashSet<Block>>::new();

        // Dominator of the entry block is the block itself.
        let entry_bb = unit.func_layout().first_block().unwrap();
        dom.insert(entry_bb, Some(entry_bb).into_iter().collect());

        // For all other blocks, set all blocks as the dominators.
        for &bb in all_blocks.iter().filter(|&&bb| bb != entry_bb) {
            dom.insert(bb, all_blocks.clone());
        }

        // Iteratively eliminate nodes that are not dominators.
        loop {
            let mut changes = false;
            for &bb in all_blocks.iter().filter(|&&bb| bb != entry_bb) {
                // Intersect all Dom(p), where p in pred(bb).
                let preds = &pred[&bb];
                let mut isect = HashMap::<Block, usize>::new();
                for &p in preds.iter().flat_map(|p| dom[p].iter()) {
                    *isect.entry(p).or_insert_with(Default::default) += 1;
                }
                let isect = isect
                    .into_iter()
                    .filter(|&(_, c)| c == preds.len())
                    .map(|(bb, _)| bb);

                // Add the block back in an update the entry Dom(bb).
                let new_dom: HashSet<Block> = isect.chain(once(bb)).collect();
                if dom[&bb] != new_dom {
                    changes |= true;
                    dom.insert(bb, new_dom);
                    trace!("  Updated {}", bb);
                }
            }
            if !changes {
                break;
            }
        }

        // Invert the tree.
        let mut dom_inv: HashMap<Block, HashSet<Block>> =
            all_blocks.iter().map(|&bb| (bb, HashSet::new())).collect();
        for (&bb, dom_by) in &dom {
            for d in dom_by {
                dom_inv.get_mut(d).unwrap().insert(bb);
            }
        }

        // Dump the tree.
        trace!("Domination Tree:");
        for (bb, dom_by) in &dom {
            trace!("  {} dominated by:", bb.dump(unit.cfg()),);
            for d in dom_by {
                trace!("    {}", d.dump(unit.cfg()));
            }
        }
        trace!("Dominator Tree:");
        for (bb, dom_to) in &dom_inv {
            trace!("  {} dominates:", bb.dump(unit.cfg()),);
            for d in dom_to {
                trace!("    {}", d.dump(unit.cfg()));
            }
        }

        // Check if a dominates b.
        let dominates = |a, b| dom[&b].contains(&a);

        // Collect instructions.
        let mut insts = vec![];
        for bb in unit.func_layout().blocks() {
            for inst in unit.func_layout().insts(bb) {
                insts.push(inst);
            }
        }

        // Perform GCSE.
        let mut modified = false;
        let mut values = HashMap::<InstData, HashSet<Value>>::new();
        'outer: for inst in insts {
            // Don't mess with instructions that produce no result or have side
            // effects.
            let opcode = unit.dfg()[inst].opcode();
            if !unit.dfg().has_result(inst) || opcode == Opcode::Prb || opcode == Opcode::Ld {
                continue;
            }
            let value = unit.dfg().inst_result(inst);
            trace!("Examining {}", inst.dump(unit.dfg(), unit.try_cfg()));

            // Try the candidates.
            if let Some(aliases) = values.get_mut(&unit.dfg()[inst]) {
                'inner: for &cv in aliases.iter() {
                    trace!("  Trying {}", cv.dump(unit.dfg()));
                    let cv_inst = unit.dfg().value_inst(cv);
                    let inst_bb = unit.func_layout().inst_block(inst).unwrap();
                    let cv_bb = unit.func_layout().inst_block(cv_inst).unwrap();

                    // Replace the current inst with the recorded value if the
                    // latter dominates the former.
                    if dominates(cv_bb, inst_bb) {
                        debug!(
                            "Replace {} with {}",
                            inst.dump(unit.dfg(), unit.try_cfg()),
                            cv.dump(unit.dfg()),
                        );
                        unit.dfg_mut().replace_use(value, cv);
                        unit.prune_if_unused(inst);
                        modified = true;
                        continue 'outer;
                    }

                    // Replace the recorded value with the current inst if the
                    // latter dominates the former.
                    if dominates(inst_bb, cv_bb) {
                        debug!(
                            "Replace {} with {}",
                            cv.dump(unit.dfg()),
                            value.dump(unit.dfg()),
                        );
                        unit.dfg_mut().replace_use(cv, value);
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

                    // TODO(fschuiki): For `prb` insts check if they are in the
                    // same instant (same block between wait/halt), and then
                    // also allow this.
                    trace!(
                        "    Intersect(Dom({}), Dom({})):",
                        inst_bb.dump(unit.cfg()),
                        cv_bb.dump(unit.cfg())
                    );
                    for bb in dom[&inst_bb].intersection(&dom[&cv_bb]) {
                        trace!("      {}", bb.dump(unit.cfg()));
                    }
                    let target_bb =
                        dom[&inst_bb]
                            .intersection(&dom[&cv_bb])
                            .max_by(|&&bb_a, &&bb_b| {
                                if dominates(bb_a, bb_b) {
                                    std::cmp::Ordering::Less
                                } else {
                                    std::cmp::Ordering::Greater
                                }
                            });
                    let target_bb = match target_bb {
                        Some(&bb) => bb,
                        None => panic!("no target bb"),
                    };
                    trace!(
                        "    Latest common dominator: {}",
                        target_bb.dump(unit.cfg())
                    );

                    // Hoist the instruction up into the target block.
                    debug!(
                        "Hoist {} up into {}",
                        inst.dump(unit.dfg(), unit.try_cfg()),
                        target_bb.dump(unit.cfg())
                    );
                    let fl = unit.func_layout_mut();
                    let term = fl.terminator(target_bb);
                    fl.remove_inst(inst);
                    fl.insert_inst_before(inst, term);

                    // Replace all uses of the recorded value with the inst.
                    debug!(
                        "Replace {} with {}",
                        cv.dump(unit.dfg()),
                        value.dump(unit.dfg()),
                    );
                    unit.dfg_mut().replace_use(cv, value);
                    unit.prune_if_unused(cv_inst);
                    aliases.remove(&cv); // crazy that this works; NLL <3
                    modified = true;
                    break 'inner;
                }
            }

            // Insert the instruction into the table.
            // trace!("Recording {}", inst.dump(unit.dfg(), unit.try_cfg()));
            values
                .entry(unit.dfg()[inst].clone())
                .or_insert_with(Default::default)
                .insert(value);
        }
        modified
    }
}
