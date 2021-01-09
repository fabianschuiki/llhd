// Copyright (c) 2017-2021 Fabian Schuiki

use crate::ir::prelude::*;
use std::collections::{HashMap, HashSet};

/// A table of basic block predecessors.
#[derive(Debug, Clone)]
pub struct PredecessorTable {
    pred: HashMap<Block, HashSet<Block>>,
    succ: HashMap<Block, HashSet<Block>>,
}

impl PredecessorTable {
    /// Compute the predecessor table for a function or process.
    #[deprecated(since = "0.13.0", note = "use unit.predtbl() instead")]
    pub fn new(unit: &Unit) -> Self {
        let mut pred = HashMap::new();
        let mut succ = HashMap::new();
        for bb in unit.blocks() {
            pred.insert(bb, HashSet::new());
        }
        for bb in unit.blocks() {
            if let Some(term) = unit.last_inst(bb) {
                for to_bb in unit[term].blocks() {
                    pred.get_mut(&to_bb).unwrap().insert(bb);
                }
                succ.insert(bb, unit[term].blocks().iter().cloned().collect());
            } else {
                succ.insert(bb, Default::default());
            }
        }
        Self { pred, succ }
    }

    /// Compute the temporal predecessor table for a process.
    ///
    /// This is a special form of predecessor table which ignores edges in the
    /// CFG that cross a temporal instruction. As such all connected blocks in
    /// the table are guaranteed to execute within the same instant of time.
    #[deprecated(since = "0.13.0", note = "use unit.temporal_predtbl() instead")]
    pub fn new_temporal(unit: &Unit) -> Self {
        let mut pred = HashMap::new();
        let mut succ = HashMap::new();
        for bb in unit.blocks() {
            pred.insert(bb, HashSet::new());
        }
        for bb in unit.blocks() {
            if let Some(term) = unit.last_inst(bb) {
                if !unit[term].opcode().is_temporal() {
                    for to_bb in unit[term].blocks() {
                        pred.get_mut(&to_bb).unwrap().insert(bb);
                    }
                    succ.insert(bb, unit[term].blocks().iter().cloned().collect());
                } else {
                    succ.insert(bb, Default::default());
                }
            } else {
                succ.insert(bb, Default::default());
            }
        }
        Self { pred, succ }
    }

    /// Get a map of blocks to predecessor sets in this table.
    pub fn all_pred_sets(&self) -> &HashMap<Block, HashSet<Block>> {
        &self.pred
    }

    /// Get a map of blocks to successor sets in this table.
    pub fn all_succs_sets(&self) -> &HashMap<Block, HashSet<Block>> {
        &self.succ
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
