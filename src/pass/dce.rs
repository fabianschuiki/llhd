// Copyright (c) 2017-2019 Fabian Schuiki

//! Dead Code Elimination

use crate::ir::prelude::*;
use crate::opt::prelude::*;
use crate::pass::gcse::PredecessorTable;
use std::collections::{HashMap, HashSet};

/// Dead Code Elimination
///
/// This pass implements dead code elimination. It removes instructions whose
/// value is never used, trivial blocks, and blocks which cannot be reached.
pub struct DeadCodeElim;

impl Pass for DeadCodeElim {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        info!("DCE [{}]", unit.unit().name());
        let mut modified = false;

        // Gather a list of instructions and investigate which branches and
        // blocks are trivial.
        let mut insts = vec![];
        let mut trivial_branches = HashMap::new();
        let mut trivial_blocks = HashMap::new();
        let entry = unit.func_layout().entry();
        for bb in unit.func_layout().blocks() {
            let term = unit.func_layout().terminator(bb);
            check_branch_trivial(unit, bb, term, &mut trivial_blocks, &mut trivial_branches);
            for inst in unit.func_layout().insts(bb) {
                if inst != term {
                    insts.push(inst);
                }
            }
        }
        check_block_retargetable(unit, entry, &mut trivial_blocks, &mut trivial_branches);
        trace!("Trivial Blocks: {:?}", trivial_blocks);
        trace!("Trivial Branches: {:?}", trivial_branches);

        // Simplify trivial branches.
        for (inst, target) in trivial_branches
            .into_iter()
            .flat_map(|(i, t)| t.map(|t| (i, t)))
        {
            debug!(
                "Replacing {} with br {}",
                inst.dump(unit.dfg(), unit.try_cfg()),
                target.dump(unit.cfg())
            );
            unit.insert_before(inst);
            unit.ins().br(target);
            unit.remove_inst(inst);
            modified |= true;
        }

        // Replace trivial blocks.
        for (from, to) in trivial_blocks
            .into_iter()
            .flat_map(|(b, w)| w.map(|w| (b, w)))
            .filter(|(from, to)| from != to)
        {
            debug!(
                "Replacing trivial block {} with {}",
                from.dump(unit.cfg()),
                to.dump(unit.cfg())
            );
            unit.dfg_mut().replace_block_use(from, to);
            // If this is the entry block, hoist the target up as the first block.
            if from == entry {
                unit.func_layout_mut().swap_blocks(from, to);
            }
            modified |= true;
        }

        // Prune instructions and unreachable blocks.
        for inst in insts {
            modified |= unit.prune_if_unused(inst);
        }
        modified |= prune_blocks(unit);

        // Detect trivially sequential blocks. We use a temporal predecessor
        // table here to avoid treating wait instructions as branches.
        let pt = PredecessorTable::new_temporal(unit.dfg(), unit.func_layout());
        let mut merge_blocks = Vec::new();
        let mut already_merged = HashMap::new();
        for bb in unit.func_layout().blocks().filter(|&bb| bb != entry) {
            let preds = pt.pred_set(bb);
            if preds.len() == 1 {
                let pred = preds.iter().cloned().next().unwrap();
                if pt.is_sole_succ(bb, pred) {
                    let into = already_merged.get(&pred).cloned().unwrap_or(pred);
                    merge_blocks.push((bb, into));
                    already_merged.insert(bb, into);
                }
            }
        }

        // Concatenate trivially sequential blocks.
        for (block, into) in merge_blocks {
            debug!(
                "Merge {} into {}",
                block.dump(unit.cfg()),
                into.dump(unit.cfg())
            );
            let layout = unit.func_layout_mut();
            let term = layout.terminator(into);
            while let Some(inst) = layout.first_inst(block) {
                layout.remove_inst(inst);
                layout.insert_inst_before(inst, term);
            }
            layout.remove_inst(term);
            unit.dfg_mut().replace_block_use(block, into);
            unit.remove_block(block);
        }

        modified
    }

    fn run_on_entity(_ctx: &PassContext, unit: &mut EntityBuilder) -> bool {
        info!("DCE [{}]", unit.unit().name());
        let mut modified = false;
        for inst in unit.entity.layout.insts().collect::<Vec<_>>() {
            modified |= unit.prune_if_unused(inst);
        }
        modified
    }
}

/// Check if a branch that terminates a block is trivial.
fn check_branch_trivial(
    unit: &impl UnitBuilder,
    _block: Block,
    inst: Inst,
    triv_bb: &mut HashMap<Block, Option<Block>>,
    triv_br: &mut HashMap<Inst, Option<Block>>,
) -> Option<Block> {
    // Insert a sentinel value to avoid recursion.
    if let Some(&entry) = triv_br.get(&inst) {
        return entry;
    }
    triv_br.insert(inst, None);
    trace!(
        "Checking if trivial {}",
        inst.dump(unit.dfg(), unit.try_cfg())
    );

    // Now we know the block is empty. Check for a few common cases of trivial
    // branches.
    let data = &unit.dfg()[inst];
    let target = match data.opcode() {
        Opcode::Br => {
            let bb = data.blocks()[0];
            check_block_retargetable(unit, bb, triv_bb, triv_br)
        }
        Opcode::BrCond => {
            let arg = data.args()[0];
            let bbs = data.blocks();
            let bbs: Vec<_> = bbs
                .iter()
                .map(|&bb| check_block_retargetable(unit, bb, triv_bb, triv_br))
                .collect();
            if let Some(imm) = unit.dfg().get_const_int(arg) {
                bbs[!imm.is_zero() as usize]
            } else if bbs[0] == bbs[1] {
                bbs[0]
            } else {
                None
            }
        }
        _ => None,
    };
    triv_br.insert(inst, target);
    target
}

/// Check if a block can be trivially addressed from a different block, and if
/// so, return a potential immediate forward through the block if trivial.
fn check_block_retargetable(
    unit: &impl UnitBuilder,
    block: Block,
    triv_bb: &mut HashMap<Block, Option<Block>>,
    triv_br: &mut HashMap<Inst, Option<Block>>,
) -> Option<Block> {
    trace!("Checking if trivial {}", block.dump(unit.cfg()));

    // Check that there are no phi nodes on the target block.
    if unit
        .func_layout()
        .insts(block)
        .any(|inst| unit.dfg()[inst].opcode().is_phi())
    {
        triv_bb.insert(block, None);
        return None;
    }

    // If the block is not trivially empty, it is retargetable but cannot be
    // "jumped through".
    let layout = unit.func_layout();
    if layout.first_inst(block) != layout.last_inst(block) {
        triv_bb.insert(block, Some(block));
        return Some(block);
    }

    // Dig up the terminator instruction and potentially resolve the target to
    // its trivial successor.
    let inst = unit.func_layout().terminator(block);
    let target = Some(check_branch_trivial(unit, block, inst, triv_bb, triv_br).unwrap_or(block));
    triv_bb.insert(block, target);
    target
}

/// Eliminate unreachable and trivial blocks in a function layout.
fn prune_blocks(builder: &mut impl UnitBuilder) -> bool {
    let mut modified = false;

    // Find all blocks reachable from the entry point.
    let first_bb = builder.func_layout().first_block().unwrap();
    let mut unreachable: HashSet<Block> = builder.func_layout().blocks().collect();
    let mut todo: Vec<Block> = Default::default();
    todo.push(first_bb);
    unreachable.remove(&first_bb);
    while let Some(block) = todo.pop() {
        let term_inst = builder.func_layout().terminator(block);
        for &bb in builder.dfg()[term_inst].blocks() {
            if unreachable.remove(&bb) {
                todo.push(bb);
            }
        }
    }

    // Remove all unreachable blocks.
    for bb in unreachable {
        debug!("Prune unreachable block {}", bb.dump(builder.cfg()));
        modified |= true;
        builder.remove_block(bb);
    }

    modified
}
