// Copyright (c) 2017-2021 Fabian Schuiki

use crate::ir::prelude::*;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Index,
};

/// A data structure that temporally groups blocks and instructions.
#[derive(Debug)]
pub struct TemporalRegionGraph {
    /// Map that assigns blocks into a region.
    blocks: HashMap<Block, TemporalRegion>,
    /// Actual region information.
    regions: Vec<TemporalRegionData>,
}

impl TemporalRegionGraph {
    /// Compute the TRG of a process.
    #[deprecated(since = "0.13.0", note = "use unit.trg() instead")]
    pub fn new(unit: &Unit) -> Self {
        // trace!("[TRG] Constructing TRG:");

        // Populate the worklist with the entry block, as well as any blocks
        // that are targeted by `wait` instructions.
        let mut todo = VecDeque::new();
        let mut seen = HashSet::new();
        todo.push_back(unit.entry());
        seen.insert(unit.entry());
        // trace!("[TRG]   Root {:?} (entry)", unit.entry());
        for bb in unit.blocks() {
            let term = unit.terminator(bb);
            if unit[term].opcode().is_temporal() {
                for &target in unit[term].blocks() {
                    if seen.insert(target) {
                        // trace!("[TRG]   Root {:?} (wait target)", target);
                        todo.push_back(target);
                    }
                }
            }
        }

        // Assign the root temporal regions.
        let mut next_id = 0;
        let mut blocks = HashMap::<Block, TemporalRegion>::new();
        let mut head_blocks = HashSet::new();
        let mut tail_blocks = HashSet::new();
        let mut breaks = vec![];
        for &bb in &todo {
            blocks.insert(bb, TemporalRegion(next_id));
            head_blocks.insert(bb);
            next_id += 1;
        }

        // Assign temporal regions to the blocks.
        while let Some(bb) = todo.pop_front() {
            let tr = blocks[&bb];
            // trace!("[TRG]   Pushing {:?} ({})", bb, tr);
            let term = unit.terminator(bb);
            if unit[term].opcode().is_temporal() {
                breaks.push(term);
                tail_blocks.insert(bb);
                continue;
            }
            for &target in unit[term].blocks() {
                if seen.insert(target) {
                    todo.push_back(target);
                    // trace!("[TRG]     Assigning {:?} <- {:?}", target, tr);
                    if blocks.insert(target, tr).is_some() {
                        let tr = TemporalRegion(next_id);
                        blocks.insert(target, tr);
                        head_blocks.insert(target);
                        tail_blocks.insert(bb);
                        // trace!("[TRG]     Assigning {:?} <- {:?} (override)", target, tr);
                        next_id += 1;
                    }
                }
            }
        }
        // trace!("[TRG]   Blocks: {:?}", blocks);

        // Create a data struct for each region.
        let mut regions: Vec<_> = (0..next_id)
            .map(|id| TemporalRegionData {
                id: TemporalRegion(id),
                blocks: Default::default(),
                entry: false,
                head_insts: Default::default(),
                head_blocks: Default::default(),
                head_tight: true,
                tail_insts: Default::default(),
                tail_blocks: Default::default(),
                tail_tight: true,
            })
            .collect();

        // Mark the entry block.
        regions[blocks[&unit.entry()].0].entry = true;

        // Build the predecessor table.
        let pt = unit.predtbl();

        // Note the blocks in each region and build the head/tail information.
        for (&bb, &id) in &blocks {
            let mut reg = &mut regions[id.0];
            reg.blocks.insert(bb);

            // Determine whether this is a head block.
            let mut is_head = head_blocks.contains(&bb);
            let mut is_tight = true;
            for pred in pt.pred(bb) {
                let diff_trs = blocks[&pred] != id;
                is_head |= diff_trs;
                is_tight &= diff_trs;
            }
            if is_head {
                reg.head_blocks.insert(bb);
                reg.head_tight &= is_tight;
            }

            // Determine whether this is a tail block.
            let mut is_tail = tail_blocks.contains(&bb);
            let mut is_tight = true;
            for succ in pt.succ(bb) {
                let diff_trs = blocks[&succ] != id;
                is_tail |= diff_trs;
                is_tight &= diff_trs;
            }
            if is_tail {
                reg.tail_blocks.insert(bb);
                reg.tail_tight &= is_tight;
            }

            // Note the head instructions.
            for pred in pt.pred(bb) {
                if blocks[&pred] != id {
                    reg.head_insts.insert(unit.terminator(pred));
                }
            }

            // Note the tail instructions.
            let term = unit.terminator(bb);
            if unit[term].blocks().iter().any(|bb| blocks[bb] != id) {
                reg.tail_insts.insert(term);
            }
        }

        Self { blocks, regions }
    }

    /// Check if a block is a temporal head block.
    pub fn is_head(&self, bb: Block) -> bool {
        self[self[bb]].is_head(bb)
    }

    /// Check if a block is a temporal tail block.
    pub fn is_tail(&self, bb: Block) -> bool {
        self[self[bb]].is_tail(bb)
    }

    /// Get the temporal regions in the graph.
    pub fn regions(&self) -> impl Iterator<Item = &TemporalRegionData> {
        self.regions.iter()
    }
}

impl Index<TemporalRegion> for TemporalRegionGraph {
    type Output = TemporalRegionData;
    fn index(&self, idx: TemporalRegion) -> &Self::Output {
        &self.regions[idx.0]
    }
}

impl Index<Block> for TemporalRegionGraph {
    type Output = TemporalRegion;
    fn index(&self, idx: Block) -> &Self::Output {
        &self.blocks[&idx]
    }
}

/// A unique identifier for a temporal region.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemporalRegion(usize);

impl std::fmt::Display for TemporalRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl std::fmt::Debug for TemporalRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Data associated with a temporal region.
#[derive(Debug, Clone)]
pub struct TemporalRegionData {
    /// The unique identifier for this region.
    pub id: TemporalRegion,

    /// The blocks in this region.
    pub blocks: HashSet<Block>,

    /// Whether this is the initial temporal region upon entering the process.
    pub entry: bool,

    /// The temporal instructions that introduce this region.
    ///
    /// Note that these reside in blocks *outside* this region, namely in the
    /// predecessors of the `head_blocks`.
    pub head_insts: HashSet<Inst>,

    /// The entry blocks into this region.
    ///
    /// These are the first blocks that are jumped into upon entering this
    /// region.
    pub head_blocks: HashSet<Block>,

    /// The head blocks are only reachable via branches from *other* regions.
    pub head_tight: bool,

    /// The temporal instructions that terminate this region.
    ///
    /// Note that these reside in blocks *inside* this region, namely in the
    /// `tail_blocks`.
    pub tail_insts: HashSet<Inst>,

    /// The exit blocks out of this region.
    ///
    /// These are the last blocks in this region, where execution either ends
    /// in a `wait` or `halt` instruction.
    pub tail_blocks: HashSet<Block>,

    /// The tail blocks only branch to *other* regions.
    pub tail_tight: bool,
}

impl TemporalRegionData {
    /// An iterator over the blocks in this region.
    pub fn blocks(&self) -> impl Iterator<Item = Block> + Clone + '_ {
        self.blocks.iter().cloned()
    }

    /// An iterator over the head instructions in this region.
    pub fn head_insts(&self) -> impl Iterator<Item = Inst> + Clone + '_ {
        self.head_insts.iter().cloned()
    }

    /// An iterator over the head blocks in this region.
    pub fn head_blocks(&self) -> impl Iterator<Item = Block> + Clone + '_ {
        self.head_blocks.iter().cloned()
    }

    /// An iterator over the tail instructions in this region.
    pub fn tail_insts(&self) -> impl Iterator<Item = Inst> + Clone + '_ {
        self.tail_insts.iter().cloned()
    }

    /// An iterator over the tail blocks in this region.
    pub fn tail_blocks(&self) -> impl Iterator<Item = Block> + Clone + '_ {
        self.tail_blocks.iter().cloned()
    }

    /// Check if a block is a temporal head block.
    pub fn is_head(&self, bb: Block) -> bool {
        self.head_blocks.contains(&bb)
    }

    /// Check if a block is a temporal tail block.
    pub fn is_tail(&self, bb: Block) -> bool {
        self.tail_blocks.contains(&bb)
    }
}
