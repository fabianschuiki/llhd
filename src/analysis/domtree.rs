// Copyright (c) 2017-2020 Fabian Schuiki

use crate::{
    analysis::PredecessorTable,
    ir::{prelude::*, ValueData},
    table::TableKey,
};
use hibitset::BitSet;
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
};

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
    #[deprecated(since = "0.13.0", note = "use unit.domtree() instead")]
    pub fn new(unit: &Unit, pred: &PredecessorTable) -> Self {
        let t0 = time::precise_time_ns();
        let post_order = Self::compute_blocks_post_order(unit, pred);
        let length = post_order.len();
        // trace!("[DomTree] post-order {:?}", post_order);

        let undef = std::u32::MAX;
        let mut doms = vec![undef; length];
        let mut inv_post_order = vec![undef; unit.block_id_bound()];
        for (i, &bb) in post_order.iter().enumerate() {
            inv_post_order[bb.index()] = i as u32;
        }
        // trace!("[DomTree] inv-post-order {:?}", inv_post_order);

        for root in Some(unit.entry())
            .into_iter()
            .chain(unit.blocks().filter(|&id| pred.pred_set(id).is_empty()))
        {
            let poidx = inv_post_order[root.index()];
            doms[poidx as usize] = poidx; // root nodes
        }
        // trace!("[DomTree] initial {:?}", doms);

        let mut changed = true;
        while changed {
            changed = false;
            // trace!("[DomTree] iteration {:?}", doms);

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
                    // trace!("[DomTree] doms[{}] = {}", idx, new_idom);
                    doms[idx] = new_idom;
                    changed = true;
                }
            }
        }
        // trace!("[DomTree] converged {:?}", doms);

        let mut doms_final = vec![Block::invalid(); unit.block_id_bound()];
        for bb in &post_order {
            doms_final[bb.index()] = post_order[doms[inv_post_order[bb.index()] as usize] as usize];
        }
        // trace!("[DomTree] final {:?}", doms_final);

        // Compatibility with old dominator tree.
        let mut dominated = HashMap::new();
        for block in unit.blocks() {
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
            unit.blocks().map(|bb| (bb, HashSet::new())).collect();
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

    fn compute_blocks_post_order(unit: &Unit, pred: &PredecessorTable) -> Vec<Block> {
        let mut order = Vec::with_capacity(pred.all_pred_sets().len());

        let mut stack = Vec::with_capacity(8);
        let mut discovered = BitSet::with_capacity(pred.all_pred_sets().len() as u32);
        let mut finished = BitSet::with_capacity(pred.all_pred_sets().len() as u32);

        stack.push(unit.entry());
        stack.extend(unit.blocks().filter(|&id| pred.pred_set(id).is_empty()));

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
    pub fn inst_dominates_block(&self, unit: &Unit, inst: Inst, block: Block) -> bool {
        match unit.inst_block(inst) {
            Some(bb) => self.block_dominates_block(bb, block),
            None => false,
        }
    }

    /// Check if a value dominates a block.
    pub fn value_dominates_block(&self, unit: &Unit, value: Value, block: Block) -> bool {
        match unit[value] {
            ValueData::Inst { inst, .. } => self.inst_dominates_block(unit, inst, block),
            ValueData::Arg { .. } => true,
            _ => false,
        }
    }
}

/// Total time spent constructing dominator trees.
pub static DOMINATOR_TREE_TIME: AtomicU64 = AtomicU64::new(0);
