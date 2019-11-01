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
    fn run_on_process(_ctx: &PassContext, prok: &mut ProcessBuilder) -> bool {
        // Build the predecessor table.
        let mut pred = HashMap::<Block, HashSet<Block>>::new();
        for bb in prok.func_layout().blocks() {
            pred.insert(bb, Default::default());
        }
        for bb in prok.func_layout().blocks() {
            let term = prok.func_layout().last_inst(bb).unwrap();
            for to_bb in prok.dfg()[term].blocks() {
                pred.get_mut(&to_bb).unwrap().insert(bb);
            }
        }
        trace!("Predecessor table: {:?}", pred);

        // Build the dominator tree.
        trace!("Building dominator tree");

        let all_blocks: HashSet<Block> = prok.func_layout().blocks().collect();
        let mut dom = HashMap::<Block, HashSet<Block>>::new();

        // Dominator of the entry block is the block itself.
        let entry_bb = prok.func_layout().first_block().unwrap();
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
            trace!("  {} dominated by:", bb.dump(prok.cfg()),);
            for d in dom_by {
                trace!("    {}", d.dump(prok.cfg()));
            }
        }
        trace!("Dominator Tree:");
        for (bb, dom_to) in &dom_inv {
            trace!("  {} dominates:", bb.dump(prok.cfg()),);
            for d in dom_to {
                trace!("    {}", d.dump(prok.cfg()));
            }
        }

        // Check if a dominates b.
        let block_dominates = |a, b| dom[&b].contains(&a);
        let inst_dominates = |prok: &ProcessBuilder, a, b| {
            let bb_a = prok.func_layout().inst_block(a).unwrap();
            let bb_b = prok.func_layout().inst_block(b).unwrap();
            block_dominates(bb_a, bb_b)
        };
        let _value_dominates = |prok: &ProcessBuilder, a, b| {
            inst_dominates(prok, prok.dfg().value_inst(a), prok.dfg().value_inst(b))
        };

        // Collect instructions.
        let mut insts = vec![];
        for bb in prok.func_layout().blocks() {
            for inst in prok.func_layout().insts(bb) {
                insts.push(inst);
            }
        }

        // Perform GCSE.
        let mut modified = false;
        let mut values = HashMap::<InstData, HashSet<Value>>::new();
        'outer: for inst in insts {
            // Don't mess with instructions that produce no result or have side
            // effects.
            let opcode = prok.dfg()[inst].opcode();
            if !prok.dfg().has_result(inst) || opcode == Opcode::Prb || opcode == Opcode::Ld {
                continue;
            }
            let value = prok.dfg().inst_result(inst);
            trace!("Examining {}", inst.dump(prok.dfg(), prok.try_cfg()));

            // Try the candidates.
            if let Some(aliases) = values.get_mut(&prok.dfg()[inst]) {
                'inner: for &cv in aliases.iter() {
                    trace!("  Trying {}", cv.dump(prok.dfg()));
                    let cv_inst = prok.dfg().value_inst(cv);

                    // Replace the current inst with the recorded value if the
                    // latter dominates the former.
                    if inst_dominates(prok, cv_inst, inst) {
                        debug!(
                            "Replace {} with {}",
                            inst.dump(prok.dfg(), prok.try_cfg()),
                            cv.dump(prok.dfg()),
                        );
                        prok.dfg_mut().replace_use(value, cv);
                        prok.prune_if_unused(inst);
                        modified = true;
                        continue 'outer;
                    }

                    // Replace the recorded value with the current inst if the
                    // latter dominates the former.
                    if inst_dominates(prok, inst, cv_inst) {
                        debug!(
                            "Replace {} with {}",
                            cv.dump(prok.dfg()),
                            value.dump(prok.dfg()),
                        );
                        prok.dfg_mut().replace_use(cv, value);
                        prok.prune_if_unused(cv_inst);
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

                    let inst_bb = prok.func_layout().inst_block(inst).unwrap();
                    let cv_bb = prok.func_layout().inst_block(cv_inst).unwrap();
                    // TODO(fschuiki): For `prb` insts check if they are in the
                    // same instant (same block between wait/halt), and then
                    // also allow this.
                    trace!(
                        "    Intersect(Dom({}), Dom({})):",
                        inst_bb.dump(prok.cfg()),
                        cv_bb.dump(prok.cfg())
                    );
                    for bb in dom[&inst_bb].intersection(&dom[&cv_bb]) {
                        trace!("      {}", bb.dump(prok.cfg()));
                    }
                    let target_bb =
                        dom[&inst_bb]
                            .intersection(&dom[&cv_bb])
                            .max_by(|&&bb_a, &&bb_b| {
                                if block_dominates(bb_a, bb_b) {
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
                        target_bb.dump(prok.cfg())
                    );

                    // Hoist the instruction up into the target block.
                    debug!(
                        "Hoist {} up into {}",
                        inst.dump(prok.dfg(), prok.try_cfg()),
                        target_bb.dump(prok.cfg())
                    );
                    let fl = prok.func_layout_mut();
                    let term = fl.last_inst(target_bb).unwrap();
                    fl.remove_inst(inst);
                    fl.insert_inst_before(inst, term);

                    // Replace all uses of the recorded value with the inst.
                    debug!(
                        "Replace {} with {}",
                        cv.dump(prok.dfg()),
                        value.dump(prok.dfg()),
                    );
                    prok.dfg_mut().replace_use(cv, value);
                    prok.prune_if_unused(cv_inst);
                    aliases.remove(&cv); // crazy that this works; NLL <3
                    modified = true;
                    break 'inner;
                }
            }

            // Insert the instruction into the table.
            // trace!("Recording {}", inst.dump(prok.dfg(), prok.try_cfg()));
            values
                .entry(prok.dfg()[inst].clone())
                .or_insert_with(Default::default)
                .insert(value);
        }
        modified
    }
}
