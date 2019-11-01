// Copyright (c) 2017-2019 Fabian Schuiki

//! Temporal Code Motion

use crate::ir::prelude::*;
use crate::ir::{DataFlowGraph, FunctionLayout};
use crate::opt::prelude::*;
use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    ops::Index,
};

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
    fn run_on_cfg(_ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        info!("TCM [{}]", unit.unit().name());

        // Build the temporal region graph.
        let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());
        trace!("Breaks: {:#?}", trg.breaks);
        trace!("Blocks: {:#?}", trg.blocks);
        trace!("Regions: {:#?}", trg.regions);

        // Hoist `prb` instructions which directly operate on input signals to
        // the head block of their region.
        for tr in &trg.regions {
            let dfg = unit.dfg();
            let layout = unit.func_layout();
            if tr.head_blocks.len() != 1 {
                trace!("Skipping {} (multiple head blocks)", tr.id);
                continue;
            }
            let mut hoist = vec![];
            for &bb in &tr.blocks {
                for inst in layout.insts(bb) {
                    if dfg[inst].opcode() == Opcode::Prb
                        && dfg.get_value_inst(dfg[inst].args()[0]).is_none()
                    {
                        hoist.push(inst);
                    }
                }
            }
            hoist.sort();
            let head_bb = *tr.head_blocks.iter().next().unwrap();
            for inst in hoist {
                debug!(
                    "Hoisting {} into {}",
                    inst.dump(unit.dfg(), unit.try_cfg()),
                    head_bb.dump(unit.cfg())
                );
                let layout = unit.func_layout_mut();
                layout.remove_inst(inst);
                layout.prepend_inst(inst, head_bb);
            }
        }

        false
    }
}

/// A data structure that temporally groups blocks and instructions.
pub struct TemporalRegionGraph {
    /// All temporal instructions.
    breaks: Vec<Inst>,
    /// Map that assigns blocks into a region.
    blocks: HashMap<Block, TemporalRegion>,
    /// Actual region information.
    regions: Vec<TemporalRegionData>,
}

impl TemporalRegionGraph {
    /// Compute the TRG of a process.
    pub fn new(dfg: &DataFlowGraph, layout: &FunctionLayout) -> Self {
        // In a first pass assign ids to each block, and mark the ids of two
        // blocks equivalent if they are connected by a branch instruction.
        let mut replace = HashMap::<TemporalRegion, TemporalRegion>::new();
        let mut blocks = HashMap::<Block, TemporalRegion>::new();
        let mut breaks = vec![];
        let mut next_id = 0;
        for bb in layout.blocks() {
            let term = layout.terminator(bb);
            let id = *blocks.entry(bb).or_insert_with(|| {
                let k = next_id;
                next_id += 1;
                TemporalRegion(k)
            });
            if dfg[term].opcode().is_temporal() {
                breaks.push(term);
            } else {
                for &to_bb in dfg[term].blocks() {
                    if let Some(old_id) = blocks.insert(to_bb, id) {
                        if old_id != id {
                            replace.insert(max(old_id, id), min(old_id, id));
                        }
                    }
                }
            }
        }

        // In a second pass apply all replacements noted above, which assigns
        // the lowest ids possible to each region.
        let mut max_id = 0;
        for (_, id) in &mut blocks {
            while let Some(&new_id) = replace.get(id) {
                *id = new_id;
            }
            max_id = max(max_id, id.0);
        }

        // Create a data struct for each region.
        let mut regions: Vec<_> = (0..max_id + 1)
            .map(|id| TemporalRegionData {
                id: TemporalRegion(id),
                blocks: Default::default(),
                head_insts: Default::default(),
                head_blocks: Default::default(),
                tail_insts: Default::default(),
                tail_blocks: Default::default(),
            })
            .collect();

        // Note the blocks in each region.
        for (&bb, &id) in &blocks {
            regions[id.0].blocks.insert(bb);
        }

        // Note the temporal instructions that introduce and terminate each
        // region.
        for &inst in &breaks {
            let bb = layout.inst_block(inst).unwrap();
            for &to_bb in dfg[inst].blocks() {
                let data = &mut regions[blocks[&to_bb].0];
                data.head_insts.insert(inst);
                data.head_blocks.insert(to_bb);
            }
            let data = &mut regions[blocks[&bb].0];
            data.tail_insts.insert(inst);
            data.tail_blocks.insert(bb);
        }

        Self {
            breaks,
            blocks,
            regions,
        }
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
}
