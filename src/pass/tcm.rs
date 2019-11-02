// Copyright (c) 2017-2019 Fabian Schuiki

//! Temporal Code Motion

use crate::ir::prelude::*;
use crate::ir::{DataFlowGraph, FunctionLayout, InstData};
use crate::opt::prelude::*;
use crate::pass::gcse::{DominatorTree, PredecessorTable};
use std::{
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
        let mut modified = false;

        // Build the temporal region graph.
        let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());
        trace!("Breaks: {:#?}", trg.breaks);
        trace!("Blocks: {:#?}", trg.blocks);
        trace!("Regions: {:#?}", trg.regions);

        // Hoist `prb` instructions which directly operate on input signals to
        // the head block of their region.
        let temp_pt = PredecessorTable::new_temporal(unit.dfg(), unit.func_layout());
        let temp_dt = DominatorTree::new(unit.func_layout(), &temp_pt);
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

        // Push `drv` instructions towards the tail of the regions.
        for i in 0..100 {
            let mut changes = false;
            debug!("Moving `drv` iteration {}", i);

            // Recompute the TRG.
            let inner_trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());
            assert_eq!(
                inner_trg.regions.len(),
                trg.regions.len(),
                "{:#?}",
                inner_trg
            );

            // Push `drv` instructions down into the successors their sole
            // successors.
            let pred = PredecessorTable::new(unit.dfg(), unit.func_layout());
            changes |= diverge_drives(unit, &inner_trg, &pred); // needs new pred afterwards

            // Push `drv` instructions towards the end of their region as far as
            // possible, merging drives to the same signal in different branches.
            let pred = PredecessorTable::new(unit.dfg(), unit.func_layout());
            let dt = DominatorTree::new(unit.func_layout(), &pred);
            changes |= reconverge_drives(unit, &inner_trg, &dt, &pred); // needs new pred afterwards

            // Continue if we were able to make some changes.
            modified |= changes;
            if !changes {
                break;
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

        modified
    }
}

// Push `drv` instructions downwards across diverging control flow. Stops at
// reconvergent control flow where special treatment is necessary.
fn diverge_drives(
    unit: &mut impl UnitBuilder,
    trg: &TemporalRegionGraph,
    pt: &PredecessorTable,
) -> bool {
    let mut relocs = HashSet::new();
    let mut worklist = vec![];

    // Initialize the relocation list with all drives and take note of the
    // driven signals.
    let mut driven_sigs = HashSet::new(); // (bb, sig)
    let mut drive_order = HashMap::new();
    for bb in unit.func_layout().blocks() {
        let mut order_id = 0;
        for inst in unit.func_layout().insts(bb) {
            if unit.dfg()[inst].opcode() == Opcode::Drv {
                let reloc = (bb, inst);
                relocs.insert(reloc);
                worklist.push(reloc);
                driven_sigs.insert((bb, unit.dfg()[inst].args()[0]));
                drive_order.insert(inst, order_id);
                order_id += 1;
            }
        }
    }

    // Push the instructions down as far as possible.
    trace!("Considering drv diverges:");
    let mut helper_blocks = HashSet::new(); // (from_bb, to_bb, inst)
    while let Some((bb, inst)) = worklist.pop() {
        let sig = unit.dfg()[inst].args()[0];

        // Stop moving instructions when we hit the end of a temporal region.
        if trg.is_tail(bb) {
            trace!(
                "  Skipping {} (in temporal tail)",
                inst.dump(unit.dfg(), unit.try_cfg())
            );
            continue;
        }

        // First handle the case of non-diverging control flow. In this case we
        // can simply push the drive down into our successor if we are it's sole
        // predecessor. Otherwise we're out of luck.
        let diverging = pt.succ_set(bb).len() > 1;
        if !diverging {
            let succ = pt.succ(bb).next().unwrap();
            if pt.is_sole_pred(bb, succ) && !driven_sigs.contains(&(succ, sig)) {
                let reloc = (succ, inst);
                relocs.remove(&(bb, inst));
                relocs.insert(reloc);
                worklist.push(reloc);
                trace!(
                    "  Pushing {} into non-diverging succ {}",
                    inst.dump(unit.dfg(), unit.try_cfg()),
                    succ.dump(unit.cfg())
                );
            }
        }
        // Then handle the case of diverging control flow. In this case we push
        // the drive into successors where we are the sole predecessor, or
        // create a helper block otherwise.
        else {
            relocs.remove(&(bb, inst));
            for succ in pt.succ(bb) {
                if driven_sigs.contains(&(succ, sig)) {
                    continue;
                }

                // Directly push into blocks where we are the sole predecessor.
                if pt.is_sole_pred(bb, succ) {
                    let reloc = (succ, inst);
                    relocs.insert(reloc);
                    worklist.push(reloc);
                    trace!(
                        "  Pushing {} into diverging succ {}",
                        inst.dump(unit.dfg(), unit.try_cfg()),
                        succ.dump(unit.cfg())
                    );
                }
                // Otherwise create a helper block.
                else {
                    trace!(
                        "  Pushing {} into helper block from {} to {}",
                        inst.dump(unit.dfg(), unit.try_cfg()),
                        bb.dump(unit.cfg()),
                        succ.dump(unit.cfg())
                    );
                    helper_blocks.insert((bb, succ, inst));
                }
            }
        }
    }

    // Enforce order on drives to the same signal, and retain only last drive.
    let mut relocated = HashSet::new();
    let mut skip = HashSet::new();
    let mut lookup = HashMap::<(Block, Value, Value), Inst>::new();
    let dfg = unit.dfg();
    for &(into_bb, inst) in &relocs {
        let sig = dfg[inst].args()[0];
        let delay = dfg[inst].args()[2];
        let key = (into_bb, sig, delay);
        if let Some(&other) = lookup.get(&key) {
            let inst_bb = unit.func_layout().inst_block(inst).unwrap();
            let other_bb = unit.func_layout().inst_block(other).unwrap();
            trace!(
                "Double drive {} in {}",
                inst.dump(dfg, unit.try_cfg()),
                into_bb.dump(unit.cfg())
            );

            // If both originated in the same basic block we use their relative
            // ordering to determine which takes precedence.
            if inst_bb == other_bb {
                if drive_order[&inst] > drive_order[&other] {
                    debug!(
                        "Removing overdriven {} in {}",
                        other.dump(dfg, unit.try_cfg()),
                        other_bb.dump(unit.cfg())
                    );
                    skip.insert((into_bb, other));
                    relocated.insert(other);
                    lookup.insert(key, inst);
                } else {
                    debug!(
                        "Removing overdriven {} in {}",
                        inst.dump(dfg, unit.try_cfg()),
                        inst_bb.dump(unit.cfg())
                    );
                    skip.insert((into_bb, inst));
                    relocated.insert(inst);
                }
            }
            // Try to resolve the conflict by checking whether one occurs before
            // the other before the move.
            else {
                panic!("Cannot resolve double drive originating in different bbs");
            }
        } else {
            lookup.insert(key, inst);
        }
    }

    // Apply the relocations.
    let mut modified = false;
    for (into_bb, inst) in relocs {
        let bb = unit.func_layout().inst_block(inst).unwrap();

        // Skip instructions that don't move or have already been discarded.
        if into_bb == bb || skip.contains(&(into_bb, inst)) {
            continue;
        }

        debug!(
            "Moving {} into {}",
            inst.dump(unit.dfg(), unit.try_cfg()),
            into_bb.dump(unit.cfg())
        );
        modified |= true;
        relocated.insert(inst);

        let dfg = unit.dfg();
        let sig = dfg[inst].args()[0];
        let value = dfg[inst].args()[1];
        let delay = dfg[inst].args()[2];
        unit.prepend_to(into_bb);
        unit.ins().drv(sig, value, delay);
    }

    // Create the helper blocks.
    let mut helper_cache = HashMap::new();
    for (from_bb, to_bb, inst) in helper_blocks {
        debug!(
            "Moving {} into helper from {} to {}",
            inst.dump(unit.dfg(), unit.try_cfg()),
            from_bb.dump(unit.cfg()),
            to_bb.dump(unit.cfg())
        );
        modified |= true;
        relocated.insert(inst);

        // Create block.
        let helper = *helper_cache.entry((from_bb, to_bb)).or_insert_with(|| {
            let helper = unit.block();
            unit.append_to(helper);
            unit.ins().br(to_bb);
            let term = unit.func_layout().terminator(from_bb);
            unit.dfg_mut()[term].replace_block(to_bb, helper);
            helper
        });
        let dfg = unit.dfg();
        let sig = dfg[inst].args()[0];
        let value = dfg[inst].args()[1];
        let delay = dfg[inst].args()[2];
        unit.insert_before(unit.func_layout().terminator(helper));
        unit.ins().drv(sig, value, delay);
    }

    // Remove the relocated drives.
    for inst in relocated {
        unit.remove_inst(inst);
        modified |= true;
    }

    modified
}

// Merge `drv` instructions across reconvergent control flow boundaries.
fn reconverge_drives(
    unit: &mut impl UnitBuilder,
    trg: &TemporalRegionGraph,
    _dt: &DominatorTree,
    pt: &PredecessorTable,
) -> bool {
    let mut modified = false;
    for tr in &trg.regions {
        let dfg = unit.dfg();
        let layout = unit.func_layout();

        // Group the drives in this region by driven signal.
        let mut drvs = HashMap::<(Value, Value), HashSet<Inst>>::new();
        for bb in tr.blocks() {
            for inst in layout.insts(bb) {
                if dfg[inst].opcode() == Opcode::Drv {
                    drvs.entry((dfg[inst].args()[0], dfg[inst].args()[2]))
                        .or_default()
                        .insert(inst);
                }
            }
        }

        // Group the drives by the successor into which they can be pushed.
        trace!("Considering drv reconverges:");
        let mut candidates = HashMap::<Block, Vec<(Value, Value, Vec<Inst>)>>::new();
        for ((sig, del), drvs) in drvs {
            let mut into = HashMap::<Block, (Vec<Inst>, HashSet<Block>)>::new();
            for drv in drvs {
                let drv_bb = layout.inst_block(drv).unwrap();

                // Be careful not to merge across temporal region boundaries.
                if trg.is_tail(drv_bb) {
                    trace!(
                        "  Skipping {} (in temporal tail)",
                        drv.dump(unit.dfg(), unit.try_cfg())
                    );
                    continue;
                }

                // Consider merging of the drive across non-divergent edges
                // only.
                let succ = pt.succ_set(drv_bb);
                if succ.len() == 1 {
                    let e = into.entry(*succ.iter().next().unwrap()).or_default();
                    (e.0).push(drv);
                    (e.1).insert(drv_bb);
                    trace!("  Considering {} ", drv.dump(unit.dfg(), unit.try_cfg()));
                } else {
                    trace!(
                        "  Skipping {} (divergent control flow)",
                        drv.dump(unit.dfg(), unit.try_cfg())
                    );
                }
            }

            // Add all the movements into the set of candidates where drives
            // can be extracted from *all* predecessors of a block.
            for (into_bb, (insts, from_bbs)) in into {
                let pred_set = pt.pred_set(into_bb);
                if from_bbs == *pred_set {
                    candidates
                        .entry(into_bb)
                        .or_default()
                        .push((sig, del, insts));
                } else {
                    trace!(
                        "  Skipping merge of {:?} into {} (predecessors {:?} not fully covered by {:?})",
                        insts,
                        into_bb.dump(unit.cfg()),
                        pred_set, from_bbs
                    );
                    // TODO(fschuiki): Introduce helper basic block in this case
                    // which carries a unified drive.
                }
            }
        }

        // Consider the drives
        for (into_bb, candidates) in candidates {
            unit.prepend_to(into_bb);
            for (sig, delay, insts) in candidates {
                let dfg = unit.dfg();
                let layout = unit.func_layout();
                debug!(
                    "Grouping {} drives {:?} into {}",
                    sig.dump(dfg),
                    insts,
                    into_bb.dump(unit.cfg()),
                );

                // Determine the phi node arms and omit the phi node if the arms
                // are homogenous.
                let mut phi_args = vec![];
                let mut phi_blocks = vec![];
                for &inst in &insts {
                    phi_args.push(dfg[inst].args()[1]);
                    phi_blocks.push(layout.inst_block(inst).unwrap());
                }

                // Ensure that the blocks we merge from are unique, i.e. that no
                // block contained two drives. These cases should be handled earlier
                // when diverging drives.
                let phi_blocks_unique: HashSet<_> = phi_blocks.iter().cloned().collect();
                assert_eq!(
                    phi_blocks_unique.len(),
                    phi_blocks.len(),
                    "merging multiple drives to {} from the same origin bb should never happen: {:?}",
                    sig.dump(dfg),
                    phi_blocks
                );

                // Create phi node or bypass.
                let homogenous = phi_args.iter().all(|&x| x == phi_args[0]);
                let phi = if homogenous {
                    trace!("Using single value {}", phi_args[0].dump(unit.dfg()));
                    phi_args[0]
                } else {
                    trace!("Add phi node in {} with arms:", into_bb.dump(unit.cfg()));
                    for (v, bb) in phi_args.iter().zip(phi_blocks.iter()) {
                        trace!("  [{}, {}]", v.dump(dfg), bb.dump(unit.cfg()));
                    }
                    unit.ins().phi(phi_args, phi_blocks)
                };

                // Insert the drive.
                // unit.insert_before(unit.func_layout().terminator(into_bb));
                unit.ins().drv(sig, phi, delay);

                // Remove the old drive instructions.
                for inst in insts {
                    unit.remove_inst(inst);
                }
                modified = true;
            }
        }
    }

    modified
}

/// A data structure that temporally groups blocks and instructions.
#[derive(Debug)]
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
        trace!("Constructing TRG:");

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
                trace!("  Assigned {} to {}", k, bb);
                TemporalRegion(k)
            });
            if dfg[term].opcode().is_temporal() {
                breaks.push(term);
            } else if dfg[term].opcode().is_terminator() {
                for &to_bb in dfg[term].blocks() {
                    trace!("  Forcing {} onto {}", id.0, to_bb);
                    if let Some(old_id) = blocks.insert(to_bb, id) {
                        if old_id != id {
                            trace!("    Replace {} with {}", old_id.0, id.0);
                            replace.insert(old_id, id);
                        }
                    }
                }
            }
        }

        trace!("  Breaks: {:#?}", breaks);
        trace!("  Replace: {:#?}", replace);
        trace!("  Blocks: {:#?}", blocks);

        // In a second pass apply all replacements noted above, which assigns
        // the lowest ids possible to each region.
        let mut max_id = 0;
        let mut final_ids = HashMap::new();
        for (_, id) in &mut blocks {
            let first = *id;
            while let Some(&new_id) = replace.get(id) {
                if final_ids.contains_key(&*id) {
                    break; // accept existing ids
                }
                *id = new_id;
                if first == *id {
                    break; // cycle breaker
                }
            }
            *id = *final_ids.entry(*id).or_insert_with(|| {
                let k = max_id;
                max_id += 1;
                TemporalRegion(k)
            });
        }

        // Create a data struct for each region.
        let mut regions: Vec<_> = (0..max_id)
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
