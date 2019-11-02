// Copyright (c) 2017-2019 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::ir::prelude::*;
use crate::ir::{DataFlowGraph, FunctionLayout, InstData};
use crate::opt::prelude::*;
use crate::pass::tcm::TemporalRegionGraph;
use std::{
    collections::{HashMap, HashSet},
    iter::once,
    ops::Index,
};

/// Global Common Subexpression Elimination
///
/// This pass implements global common subexpression elimination. It tries to
/// eliminate redundant instructions.
pub struct GlobalCommonSubexprElim;

impl Pass for GlobalCommonSubexprElim {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        info!("GCSE [{}]", unit.unit().name());

        // Build the predecessor table and dominator tree.
        let pred = PredecessorTable::new(unit.dfg(), unit.func_layout());
        let dt = DominatorTree::new(unit.func_layout(), &pred);

        // Compute the TRG to allow for `prb` instructions to be eliminated.
        let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());

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
            if !unit.dfg().has_result(inst) || opcode == Opcode::Ld {
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

                    // Make sure that we don't merge `prb` instructions in
                    // different temporal regions.
                    if trg[inst_bb] != trg[cv_bb] {
                        trace!("    Skipping because in other temporal region");
                        continue;
                    }

                    // Replace the current inst with the recorded value if the
                    // latter dominates the former.
                    if dt.dominates(cv_bb, inst_bb) {
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
                    if dt.dominates(inst_bb, cv_bb) {
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

                    trace!(
                        "    Intersect(Dom({}), Dom({})):",
                        inst_bb.dump(unit.cfg()),
                        cv_bb.dump(unit.cfg())
                    );
                    for bb in dt.dominators(inst_bb).intersection(&dt.dominators(cv_bb)) {
                        trace!("      {}", bb.dump(unit.cfg()));
                    }
                    let target_bb = dt
                        .dominators(inst_bb)
                        .intersection(dt.dominators(cv_bb))
                        .max_by(|&&bb_a, &&bb_b| {
                            if dt.dominates(bb_a, bb_b) {
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

/// A table of basic block predecessors.
#[derive(Debug, Clone)]
pub struct PredecessorTable {
    pred: HashMap<Block, HashSet<Block>>,
    succ: HashMap<Block, HashSet<Block>>,
}

impl PredecessorTable {
    /// Compute the predecessor table for a function or process.
    pub fn new(dfg: &DataFlowGraph, layout: &FunctionLayout) -> Self {
        let mut pred = HashMap::new();
        let mut succ = HashMap::new();
        for bb in layout.blocks() {
            pred.insert(bb, HashSet::new());
        }
        for bb in layout.blocks() {
            let term = layout.terminator(bb);
            for to_bb in dfg[term].blocks() {
                pred.get_mut(&to_bb).unwrap().insert(bb);
            }
            succ.insert(bb, dfg[term].blocks().iter().cloned().collect());
        }
        Self { pred, succ }
    }
}

impl Index<Block> for PredecessorTable {
    type Output = HashSet<Block>;
    fn index(&self, idx: Block) -> &Self::Output {
        &self.entry[&idx]
    }
}

/// A block dominator tree.
///
/// Records for every block which other blocks in the CFG *have* to be traversed
/// to reach it. And vice versa, which blocks a block precedeces in all cases.
#[derive(Debug, Clone)]
pub struct DominatorTree {
    /// Map from a block to the blocks it dominates.
    dominates: HashMap<Block, HashSet<Block>>,
    /// Map from a block to the blocks that dominate it.
    dominated: HashMap<Block, HashSet<Block>>,
}

impl DominatorTree {
    /// Compute the dominator tree of a function or process.
    pub fn new(layout: &FunctionLayout, pred: &PredecessorTable) -> Self {
        // This is a pretty inefficient implementation, but it gets the job done
        // for now.
        let all_blocks: HashSet<Block> = layout.blocks().collect();
        let mut dominated = HashMap::<Block, HashSet<Block>>::new();

        // Dominator of the entry block is the block itself.
        let entry_bb = layout.entry();
        dominated.insert(entry_bb, Some(entry_bb).into_iter().collect());

        // For all other blocks, set all blocks as the dominators.
        for &bb in all_blocks.iter().filter(|&&bb| bb != entry_bb) {
            dominated.insert(bb, all_blocks.clone());
        }

        // Iteratively eliminate nodes that are not dominators.
        loop {
            let mut changes = false;
            for &bb in all_blocks.iter().filter(|&&bb| bb != entry_bb) {
                // Intersect all Dom(p), where p in pred(bb).
                let preds = &pred[bb];
                let mut isect = HashMap::<Block, usize>::new();
                for &p in preds.iter().flat_map(|p| dominated[p].iter()) {
                    *isect.entry(p).or_insert_with(Default::default) += 1;
                }
                let isect = isect
                    .into_iter()
                    .filter(|&(_, c)| c == preds.len())
                    .map(|(bb, _)| bb);

                // Add the block back in an update the entry Dom(bb).
                let new_dom: HashSet<Block> = isect.chain(once(bb)).collect();
                if dominated[&bb] != new_dom {
                    changes |= true;
                    dominated.insert(bb, new_dom);
                }
            }
            if !changes {
                break;
            }
        }

        // Invert the tree.
        let mut dominates: HashMap<Block, HashSet<Block>> =
            all_blocks.iter().map(|&bb| (bb, HashSet::new())).collect();
        for (&bb, dom) in &dominated {
            for d in dom {
                dominates.get_mut(d).unwrap().insert(bb);
            }
        }

        Self {
            dominates,
            dominated,
        }
    }

    /// Check if a block dominates another.
    pub fn dominates(&self, dominator: Block, follower: Block) -> bool {
        self.dominates
            .get(&dominator)
            .map(|d| d.contains(&follower))
            .unwrap_or(false)
    }

    /// Get the dominators of a block.
    pub fn dominators(&self, follower: Block) -> &HashSet<Block> {
        &self.dominated[&follower]
    }

    /// Get the followers of a block, i.e. the blocks it dominates.
    pub fn dominated_by(&self, dominator: Block) -> &HashSet<Block> {
        &self.dominates[&dominator]
    }
}
