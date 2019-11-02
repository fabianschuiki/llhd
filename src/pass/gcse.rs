// Copyright (c) 2017-2019 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::ir::prelude::*;
use crate::ir::{DataFlowGraph, FunctionLayout, InstData};
use crate::opt::prelude::*;
use crate::pass::tcm::TemporalRegionGraph;
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
        info!("GCSE [{}]", unit.unit().name());

        // Build the predecessor table and dominator tree.
        let pred = PredecessorTable::new(unit.dfg(), unit.func_layout());
        let dt = DominatorTree::new(unit.func_layout(), &pred);

        // Build the temporal predecessor table and dominator tree.
        let temp_pt = PredecessorTable::new_temporal(unit.dfg(), unit.func_layout());
        let temp_dt = DominatorTree::new(unit.func_layout(), &temp_pt);

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

                    // Decide which dominator tree to use.
                    let which_dt = if opcode == Opcode::Prb { &temp_dt } else { &dt };

                    // Replace the current inst with the recorded value if the
                    // latter dominates the former.
                    if which_dt.dominates(cv_bb, inst_bb) {
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
                    if which_dt.dominates(inst_bb, cv_bb) {
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
                    for bb in which_dt
                        .dominators(inst_bb)
                        .intersection(&which_dt.dominators(cv_bb))
                    {
                        trace!("      {}", bb.dump(unit.cfg()));
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

    /// Compute the temporal predecessor table for a process.
    ///
    /// This is a special form of predecessor table which ignores edges in the
    /// CFG that cross a temporal instruction. As such all connected blocks in
    /// the table are guaranteed to execute within the same instant of time.
    pub fn new_temporal(dfg: &DataFlowGraph, layout: &FunctionLayout) -> Self {
        let mut pred = HashMap::new();
        let mut succ = HashMap::new();
        for bb in layout.blocks() {
            pred.insert(bb, HashSet::new());
        }
        for bb in layout.blocks() {
            let term = layout.terminator(bb);
            if !dfg[term].opcode().is_temporal() {
                for to_bb in dfg[term].blocks() {
                    pred.get_mut(&to_bb).unwrap().insert(bb);
                }
                succ.insert(bb, dfg[term].blocks().iter().cloned().collect());
            }
        }
        Self { pred, succ }
    }

    /// Get the predecessors of a block.
    pub fn pred_set(&self, bb: Block) -> &HashSet<Block> {
        &self.pred[&bb]
    }

    /// Get the successors of a block.
    pub fn succ_set(&self, bb: Block) -> &HashSet<Block> {
        &self.succ[&bb]
    }

    /// Get the predecessors of a block.
    pub fn pred(&self, bb: Block) -> impl Iterator<Item = Block> + Clone + '_ {
        self.pred[&bb].iter().cloned()
    }

    /// Get the successors of a block.
    pub fn succ(&self, bb: Block) -> impl Iterator<Item = Block> + Clone + '_ {
        self.succ[&bb].iter().cloned()
    }

    /// Check if a block is the sole predecessor of another block.
    pub fn is_sole_pred(&self, bb: Block, pred_of: Block) -> bool {
        self.pred(pred_of).all(|x| x == bb)
    }

    /// Check if a block is the sole successor of another block.
    pub fn is_sole_succ(&self, bb: Block, succ_of: Block) -> bool {
        self.succ(succ_of).all(|x| x == bb)
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
                let mut isect = HashMap::<Block, usize>::new();
                for &p in pred.pred(bb).flat_map(|p| dominated[&p].iter()) {
                    *isect.entry(p).or_insert_with(Default::default) += 1;
                }
                let isect = isect
                    .into_iter()
                    .filter(|&(_, c)| c == pred.pred_set(bb).len())
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
