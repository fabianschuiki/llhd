// Copyright (c) 2017-2019 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::ir::prelude::*;
use crate::ir::{ControlFlowGraph, DataFlowGraph, FunctionLayout, InstData, ValueData};
use crate::opt::prelude::*;
use crate::pass::tcm::TemporalRegionGraph;
use crate::table::TableKey;
use hibitset::BitSet;
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
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
        let dt = DominatorTree::new(unit.cfg(), unit.func_layout(), &pred);

        // Build the temporal predecessor table and dominator tree.
        let temp_pt = PredecessorTable::new_temporal(unit.dfg(), unit.func_layout());
        let temp_dt = DominatorTree::new(unit.cfg(), unit.func_layout(), &temp_pt);

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
            if !unit.dfg().has_result(inst)
                || opcode == Opcode::Ld
                || opcode == Opcode::Var
                || opcode == Opcode::Sig
            {
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

    fn run_on_entity(_ctx: &PassContext, unit: &mut EntityBuilder) -> bool {
        info!("GCSE [{}]", unit.unit().name());

        // Collect instructions.
        let mut insts = vec![];
        for inst in unit.inst_layout().insts() {
            insts.push(inst);
        }

        // Perform GCSE.
        let mut modified = false;
        let mut values = HashMap::<InstData, Value>::new();
        for inst in insts {
            // Don't mess with instructions that produce no result or have side
            // effects.
            let opcode = unit.dfg()[inst].opcode();
            if !unit.dfg().has_result(inst) || opcode == Opcode::Var || opcode == Opcode::Sig {
                continue;
            }
            let value = unit.dfg().inst_result(inst);
            trace!("Examining {}", inst.dump(unit.dfg(), unit.try_cfg()));

            // Replace the current inst with the recorded value, if there is
            // one.
            if let Some(&cv) = values.get(&unit.dfg()[inst]) {
                debug!(
                    "Replace {} with {}",
                    inst.dump(unit.dfg(), unit.try_cfg()),
                    cv.dump(unit.dfg()),
                );
                unit.dfg_mut().replace_use(value, cv);
                unit.prune_if_unused(inst);
                modified = true;
            }
            // Otherwise insert the instruction into the table.
            else {
                values.insert(unit.dfg()[inst].clone(), value);
            }
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
            } else {
                succ.insert(bb, Default::default());
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
    /// Vector of immediate dominators.
    doms: Vec<Block>,
    /// Blocks in post-order.
    post_order: Vec<Block>,
    /// Post-order index for each block.
    inv_post_order: Vec<u32>,
}

impl DominatorTree {
    /// Compute the dominator tree of a function or process.
    ///
    /// This implementation is based on [1].
    ///
    /// [1]: https://www.cs.rice.edu/~keith/Embed/dom.pdf "Cooper, Keith D., Timothy J. Harvey, and Ken Kennedy. 'A simple, fast dominance algorithm.' Software Practice & Experience 4.1-10 (2001): 1-8."
    pub fn new(cfg: &ControlFlowGraph, layout: &FunctionLayout, pred: &PredecessorTable) -> Self {
        let t0 = time::precise_time_ns();
        let post_order = Self::compute_blocks_post_order(layout, pred);
        let length = post_order.len();
        trace!("[DomTree] post-order {:?}", post_order);

        let undef = std::u32::MAX;
        let mut doms = vec![undef; length];
        let mut inv_post_order = vec![undef; cfg.blocks.capacity()];
        for (i, &bb) in post_order.iter().enumerate() {
            inv_post_order[bb.index()] = i as u32;
        }
        trace!("[DomTree] inv-post-order {:?}", inv_post_order);

        for root in Some(layout.entry())
            .into_iter()
            .chain(layout.blocks().filter(|&id| pred.pred_set(id).is_empty()))
        {
            let poidx = inv_post_order[root.index()];
            doms[poidx as usize] = poidx; // root nodes
        }
        trace!("[DomTree] initial {:?}", doms);
        // trace!("[DomTree] preds:");
        // for (bb, p) in &pred.pred {
        //     trace!("  {}:", inv_post_order[bb.index()]);
        //     for b in p {
        //         trace!("    - {}", inv_post_order[b.index()]);
        //     }
        // }

        let mut changed = true;
        while changed {
            changed = false;
            trace!("[DomTree] iteration {:?}", doms);

            for idx in (0..length).rev() {
                if doms[idx] == idx as u32 {
                    continue; // skip root nodes
                }
                let bb = post_order[idx];

                let mut preds = pred
                    .pred_set(bb)
                    .iter()
                    .map(|id| inv_post_order[id.index()])
                    .filter(|&p| doms[p as usize] != undef);
                let new_idom = preds.next().unwrap();
                let new_idom = preds.fold(new_idom, |mut i1, mut i2| {
                    let i1_init = i1;
                    while i1 != i2 {
                        if i1 < i2 {
                            if i1 == doms[i1 as usize] {
                                return i1;
                            }
                            i1 = doms[i1 as usize];
                        } else if i2 < i1 {
                            if i2 == doms[i2 as usize] {
                                return i1_init;
                            }
                            i2 = doms[i2 as usize];
                        }
                    }
                    i1
                });
                debug_assert!(new_idom < length as u32);
                if doms[idx] != new_idom {
                    trace!("[DomTree] doms[{}] = {}", idx, new_idom);
                    doms[idx] = new_idom;
                    changed = true;
                }
            }
        }
        trace!("[DomTree] converged {:?}", doms);

        let mut doms_final = vec![Block::invalid(); cfg.blocks.capacity()];
        for bb in &post_order {
            doms_final[bb.index()] = post_order[doms[inv_post_order[bb.index()] as usize] as usize];
        }
        trace!("[DomTree] final {:?}", doms_final);

        // Compatibility with old dominator tree.
        let mut dominated = HashMap::new();
        for block in layout.blocks() {
            let mut s = HashSet::new();
            let mut bb = block;
            loop {
                s.insert(bb);
                let next = doms_final[bb.index()];
                // trace!("Dominated[{}]: {}", block, bb);
                if next == bb {
                    break;
                }
                bb = next;
            }
            dominated.insert(block, s);
        }

        // Invert the tree.
        let mut dominates: HashMap<Block, HashSet<Block>> =
            layout.blocks().map(|bb| (bb, HashSet::new())).collect();
        for (&bb, dom) in &dominated {
            for d in dom {
                dominates.get_mut(d).unwrap().insert(bb);
            }
        }

        let t1 = time::precise_time_ns();
        DOMINATOR_TREE_TIME.fetch_add(t1 - t0, Ordering::Relaxed);
        // trace!(
        //     "Dominator Tree constructed in {} ms",
        //     (t1 - t0) as f64 * 1.0e-6
        // );

        Self {
            dominates,
            dominated,
            doms: doms_final,
            post_order,
            inv_post_order,
        }
    }

    fn compute_blocks_post_order(layout: &FunctionLayout, pred: &PredecessorTable) -> Vec<Block> {
        let mut order = Vec::with_capacity(pred.pred.len());

        let mut stack = Vec::with_capacity(8);
        let mut discovered = BitSet::with_capacity(pred.pred.len() as u32);
        let mut finished = BitSet::with_capacity(pred.pred.len() as u32);

        stack.push(layout.entry());
        stack.extend(layout.blocks().filter(|&id| pred.pred_set(id).is_empty()));

        while let Some(&next) = stack.last() {
            if !discovered.add(next.index() as u32) {
                for &succ in pred.succ_set(next) {
                    if !discovered.contains(succ.index() as u32) {
                        stack.push(succ);
                    }
                }
            } else {
                stack.pop();
                if !finished.add(next.index() as u32) {
                    order.push(next);
                }
            }
        }

        order
    }

    /// Get the blocks in the original CFG in post-order.
    pub fn blocks_post_order(&self) -> &[Block] {
        &self.post_order
    }

    /// Get the post-order index of a block.
    pub fn block_order(&self, block: Block) -> usize {
        self.inv_post_order[block.index()] as usize
    }

    /// Check if a block dominates another.
    pub fn dominates(&self, dominator: Block, follower: Block) -> bool {
        self.dominates
            .get(&dominator)
            .map(|d| d.contains(&follower))
            .unwrap_or(false)
    }

    /// Get the immediate dominator of a block.
    pub fn dominator(&self, block: Block) -> Block {
        self.doms[block.index()]
    }

    /// Get the dominators of a block.
    pub fn dominators(&self, follower: Block) -> &HashSet<Block> {
        &self.dominated[&follower]
    }

    /// Get the followers of a block, i.e. the blocks it dominates.
    pub fn dominated_by(&self, dominator: Block) -> &HashSet<Block> {
        &self.dominates[&dominator]
    }

    /// Check if a block dominates another block.
    pub fn block_dominates_block(&self, parent: Block, mut child: Block) -> bool {
        while parent != child {
            let next = self.dominator(child);
            if next == child {
                // Arrived at the root of the tree. Did not encounter the
                // suspected parent, so no domination.
                return false;
            }
            child = next;
        }
        true
    }

    /// Check if an instruction dominates a block.
    pub fn inst_dominates_block(&self, layout: &FunctionLayout, inst: Inst, block: Block) -> bool {
        match layout.inst_block(inst) {
            Some(bb) => self.block_dominates_block(bb, block),
            None => false,
        }
    }

    /// Check if a value dominates a block.
    pub fn value_dominates_block(
        &self,
        dfg: &DataFlowGraph,
        layout: &FunctionLayout,
        value: Value,
        block: Block,
    ) -> bool {
        match dfg[value] {
            ValueData::Inst { inst, .. } => self.inst_dominates_block(layout, inst, block),
            ValueData::Arg { .. } => true,
            _ => false,
        }
    }
}

/// Total time spent constructing dominator trees.
pub static DOMINATOR_TREE_TIME: AtomicU64 = AtomicU64::new(0);
