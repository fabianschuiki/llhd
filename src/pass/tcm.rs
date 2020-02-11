// Copyright (c) 2017-2019 Fabian Schuiki

//! Temporal Code Motion

use crate::ir::prelude::*;
use crate::opt::prelude::*;
use crate::{
    ir::{DataFlowGraph, FunctionLayout, InstData},
    pass::gcse::{DominatorTree, PredecessorTable},
    value::IntValue,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
    fn run_on_cfg(ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        info!("TCM [{}]", unit.unit().name());
        let mut modified = false;

        // Build the temporal region graph.
        let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());

        // Hoist `prb` instructions which directly operate on input signals to
        // the head block of their region.
        let temp_pt = PredecessorTable::new_temporal(unit.dfg(), unit.func_layout());
        let temp_dt = DominatorTree::new(unit.cfg(), unit.func_layout(), &temp_pt);
        for tr in &trg.regions {
            let dfg = unit.dfg();
            let layout = unit.func_layout();
            if tr.head_blocks.len() != 1 {
                trace!("Skipping {} for prb move (multiple head blocks)", tr.id);
                continue;
            }
            let head_bb = tr.head_blocks().next().unwrap();
            let mut hoist = vec![];
            for bb in tr.blocks() {
                for inst in layout.insts(bb) {
                    if dfg[inst].opcode() == Opcode::Prb
                        && dfg.get_value_inst(dfg[inst].args()[0]).is_none()
                    {
                        // Check if the new prb location would dominate its old
                        // location temporally.
                        let mut dominates = temp_dt.dominates(head_bb, bb);

                        // Only move when the move instruction would still
                        // dominate all its uses.
                        for (user_inst, _) in dfg.uses(dfg.inst_result(inst)) {
                            let user_bb = unit.func_layout().inst_block(user_inst).unwrap();
                            let dom = temp_dt.dominates(head_bb, user_bb);
                            dominates &= dom;
                        }
                        if dominates {
                            hoist.push(inst);
                        } else {
                            trace!(
                                "Skipping {} for prb move (would not dominate uses)",
                                inst.dump(dfg, unit.try_cfg())
                            );
                        }
                    }
                }
            }
            hoist.sort();
            for inst in hoist {
                debug!(
                    "Hoisting {} into {}",
                    inst.dump(unit.dfg(), unit.try_cfg()),
                    head_bb.dump(unit.cfg())
                );
                let layout = unit.func_layout_mut();
                layout.remove_inst(inst);
                layout.prepend_inst(inst, head_bb);
                modified = true;
            }
        }

        // Fuse equivalent wait instructions.
        let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());
        for tr in &trg.regions {
            if tr.tail_insts.len() <= 1 {
                trace!("Skipping {} for wait merge (single wait inst)", tr.id);
                continue;
            }
            let mut merge = HashMap::<&InstData, Vec<Inst>>::new();
            for inst in tr.tail_insts() {
                merge.entry(&unit.dfg()[inst]).or_default().push(inst);
            }
            let merge: Vec<_> = merge.into_iter().map(|(_, is)| is).collect();
            for insts in merge {
                if insts.len() <= 1 {
                    trace!(
                        "Skipping {} (no equivalents)",
                        insts[0].dump(unit.dfg(), unit.try_cfg())
                    );
                    continue;
                }
                trace!("Merging:",);
                for i in &insts {
                    trace!("  {}", i.dump(unit.dfg(), unit.try_cfg()));
                }

                // Create a new basic block for the singleton wait inst.
                let unified_bb = unit.block();

                // Replace all waits with branches into the unified block.
                for &inst in &insts {
                    unit.insert_after(inst);
                    unit.ins().br(unified_bb);
                }

                // Add one of the instructions to the unified block and delete
                // the rest.
                unit.func_layout_mut().remove_inst(insts[0]);
                unit.func_layout_mut().append_inst(insts[0], unified_bb);
                for &inst in &insts[1..] {
                    unit.remove_inst(inst);
                }
                modified = true;
            }
        }

        // Push `drv` instructions towards the tails of their temporal regions.
        modified |= push_drives(ctx, unit);

        // TODO: Coalesce drives to the same signal.

        modified
    }
}

/// Push `drv` instructions downards into the tails of their temporal regions.
fn push_drives(ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
    let mut modified = false;

    // We need the dominator tree of the current CFG.
    let pt = PredecessorTable::new(unit.dfg(), unit.func_layout());
    let dt = DominatorTree::new(unit.cfg(), unit.func_layout(), &pt);

    // Build an alias table of all signals, which indicates which signals are
    // aliases (e.g. extf/exts) of another. As we encounter drives, keep track
    // of their sequential dependency.
    let mut aliases = HashMap::<Value, Value>::new();
    let mut drv_seq = HashMap::<Value, Vec<Inst>>::new();
    let dfg = unit.dfg();
    let cfg = unit.cfg();
    for &bb in dt.blocks_post_order().iter().rev() {
        trace!("Checking {} for aliases", bb.dump(unit.cfg()));
        for inst in unit.func_layout().insts(bb) {
            let data = &dfg[inst];
            if let Opcode::Drv | Opcode::DrvCond = data.opcode() {
                // Gather drive sequences to the same signal.
                let signal = data.args()[0];
                let signal = aliases.get(&signal).cloned().unwrap_or(signal);
                trace!(
                    "  Drive {} ({})",
                    signal.dump(dfg),
                    inst.dump(dfg, Some(cfg))
                );
                drv_seq.entry(signal).or_default().push(inst);
            } else if let Some(value) = dfg.get_inst_result(inst) {
                // Gather signal aliases.
                if !dfg.value_type(value).is_signal() {
                    continue;
                }
                for &arg in data.args() {
                    if !dfg.value_type(arg).is_signal() {
                        continue;
                    }
                    let arg = aliases.get(&arg).cloned().unwrap_or(arg);
                    trace!(
                        "  Alias {} of {} ({})",
                        value.dump(dfg),
                        arg.dump(dfg),
                        inst.dump(dfg, Some(cfg))
                    );
                    aliases.insert(value, arg);
                }
            }
        }
    }

    // Build the temporal region graph.
    let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());

    // Try to migrate drive instructions into the tails of their respective
    // temporal regions.
    for (&signal, drives) in &drv_seq {
        trace!("Moving drives on signal {}", signal.dump(unit.dfg()));
        for &drive in drives.iter().rev() {
            // trace!("  Checking {}", drive.dump(unit.dfg(), unit.try_cfg()));
            let moved = push_drive(ctx, drive, unit, &dt, &trg);
            modified |= moved;
            // If the move was not possible, abort all other drives since we
            // cannot move over them.
            if !moved {
                break;
            }
        }
    }

    trace!("All drives moved");
    modified
}

fn push_drive(
    _ctx: &PassContext,
    drive: Inst,
    unit: &mut impl UnitBuilder,
    dt: &DominatorTree,
    trg: &TemporalRegionGraph,
) -> bool {
    let dfg = unit.dfg();
    let cfg = unit.cfg();
    let layout = unit.func_layout();
    let src_bb = layout.inst_block(drive).unwrap();
    let tr = trg[src_bb];
    let mut moves = Vec::new();

    // For each tail block that this drive moves to, find the branch conditions
    // along the way that lead to the drive being executed, and check if the
    // arguments for the drive are available in the destination block.
    for dst_bb in trg[tr].tail_blocks() {
        // trace!("    Will have to move to {}", dst_bb.dump(cfg));

        // First check if all arguments of the drive instruction dominate the
        // destination block. If not, the move is not possible.
        for &arg in dfg[drive].args() {
            if !dt.value_dominates_block(dfg, layout, arg, dst_bb) {
                trace!(
                    "  Skipping {} ({} does not dominate {})",
                    drive.dump(dfg, Some(cfg)),
                    arg.dump(dfg),
                    dst_bb.dump(cfg)
                );
                return false;
            }
        }

        // Find the branch conditions that lead to src_bb being executed on the
        // way to dst_bb. We do this by stepping up the dominator tree until we
        // find the common dominator. For every step of src_bb, we record the
        // branch condition.
        let mut src_finger = src_bb;
        let mut dst_finger = dst_bb;
        let mut conds = Vec::<(Value, bool)>::new();
        while src_finger != dst_finger {
            let i1 = dt.block_order(src_finger);
            let i2 = dt.block_order(dst_finger);
            if i1 < i2 {
                let parent = dt.dominator(src_finger);
                if src_finger == parent {
                    break;
                }
                let term = layout.terminator(parent);
                if dfg[term].opcode() == Opcode::BrCond {
                    let cond_val = dfg[term].args()[0];
                    if !dt.value_dominates_block(dfg, layout, cond_val, dst_bb) {
                        trace!(
                            "  Skipping {} (branch cond {} does not dominate {})",
                            drive.dump(dfg, Some(cfg)),
                            cond_val.dump(dfg),
                            dst_bb.dump(cfg)
                        );
                        return false;
                    }
                    let cond_pol = dfg[term].blocks().iter().position(|&bb| bb == src_finger);
                    if let Some(cond_pol) = cond_pol {
                        conds.push((cond_val, cond_pol != 0));
                        trace!(
                            "    {} -> {} ({} == {})",
                            parent.dump(cfg),
                            src_finger.dump(cfg),
                            cond_val.dump(dfg),
                            cond_pol
                        );
                    }
                } else {
                    trace!("    {} -> {}", parent.dump(cfg), src_finger.dump(cfg));
                }
                src_finger = parent;
            } else if i2 < i1 {
                let parent = dt.dominator(dst_finger);
                if dst_finger == parent {
                    break;
                }
                dst_finger = parent;
            }
        }
        if src_finger != dst_finger {
            trace!(
                "  Skipping {} (no common dominator)",
                drive.dump(dfg, Some(cfg))
            );
            return false;
        }

        // Keep note of this destination block and the conditions that must
        // hold.
        moves.push((dst_bb, conds));
    }

    // If we arrive here, all moves are possible and can now be executed.
    for (dst_bb, conds) in moves {
        debug!(
            "Moving {} to {}",
            drive.dump(unit.dfg(), unit.try_cfg()),
            dst_bb.dump(unit.cfg())
        );

        // Start by assembling the drive condition in the destination block. The
        // order is key here to allow for easy constant folding and subexpr
        // elimination: the conditions are in reverse CFG order, so and them
        // together in reverse order to reflect the CFG, which allows for most
        // of these conditions to be shared.
        unit.prepend_to(dst_bb);
        let mut cond = unit.ins().const_int(IntValue::all_ones(1));
        for (value, polarity) in conds.into_iter().rev() {
            let value = match polarity {
                true => value,
                false => unit.ins().not(value),
            };
            cond = unit.ins().and(cond, value);
        }

        // Add the drive condition, if any.
        if unit.dfg()[drive].opcode() == Opcode::DrvCond {
            let arg = unit.dfg()[drive].args()[3];
            cond = unit.ins().and(cond, arg);
        }

        // Insert the new drive.
        let args = unit.dfg()[drive].args();
        let signal = args[0];
        let value = args[1];
        let delay = args[2];
        unit.ins().drv_cond(signal, value, delay, cond);
    }

    // Remove the old drive instruction.
    unit.remove_inst(drive);

    true
}

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
    pub fn new(dfg: &DataFlowGraph, layout: &FunctionLayout) -> Self {
        trace!("Constructing TRG:");

        // Populate the worklist with the entry block, as well as any blocks
        // that are targeted by `wait` instructions.
        let mut todo = VecDeque::new();
        let mut seen = HashSet::new();
        todo.push_back(layout.entry());
        seen.insert(layout.entry());
        trace!("  Root {:?} (entry)", layout.entry());
        for bb in layout.blocks() {
            let term = layout.terminator(bb);
            if dfg[term].opcode().is_temporal() {
                for &target in dfg[term].blocks() {
                    if seen.insert(target) {
                        trace!("  Root {:?} (wait target)", target);
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
            trace!("  Pushing {:?} ({})", bb, tr);
            let term = layout.terminator(bb);
            if dfg[term].opcode().is_temporal() {
                breaks.push(term);
                tail_blocks.insert(bb);
                continue;
            }
            for &target in dfg[term].blocks() {
                if seen.insert(target) {
                    todo.push_back(target);
                    trace!("    Assigning {:?} <- {:?}", target, tr);
                    if blocks.insert(target, tr).is_some() {
                        let tr = TemporalRegion(next_id);
                        blocks.insert(target, tr);
                        head_blocks.insert(target);
                        tail_blocks.insert(bb);
                        trace!("    Assigning {:?} <- {:?} (override)", target, tr);
                        next_id += 1;
                    }
                }
            }
        }
        trace!("  Blocks: {:#?}", blocks);

        // Create a data struct for each region.
        let mut regions: Vec<_> = (0..next_id)
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
            for &target in dfg[inst].blocks() {
                let data = &mut regions[blocks[&target].0];
                data.head_insts.insert(inst);
                // data.head_blocks.insert(target);
            }
            let data = &mut regions[blocks[&bb].0];
            data.tail_insts.insert(inst);
            // data.tail_blocks.insert(bb);
        }
        for bb in head_blocks {
            regions[blocks[&bb].0].head_blocks.insert(bb);
        }
        for bb in tail_blocks {
            regions[blocks[&bb].0].tail_blocks.insert(bb);
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
