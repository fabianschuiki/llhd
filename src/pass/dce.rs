// Copyright (c) 2017-2021 Fabian Schuiki

//! Dead Code Elimination

use crate::{ir::prelude::*, opt::prelude::*};
use std::collections::{HashMap, HashSet};

/// Dead Code Elimination
///
/// This pass implements dead code elimination. It removes instructions whose
/// value is never used, trivial blocks, and blocks which cannot be reached.
pub struct DeadCodeElim;

impl Pass for DeadCodeElim {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("DCE [{}]", unit.name());
        let mut modified = false;

        // Gather a list of instructions and investigate which branches and
        // blocks are trivial.
        let mut insts = vec![];
        let mut trivial_branches = HashMap::new();
        let mut trivial_blocks = HashMap::new();
        let entry = unit.entry();
        for bb in unit.blocks() {
            let term = unit.terminator(bb);
            check_branch_trivial(unit, bb, term, &mut trivial_blocks, &mut trivial_branches);
            for inst in unit.insts(bb) {
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
            if unit[inst].opcode() == Opcode::Br && unit[inst].blocks() == [target] {
                continue;
            }
            debug!(
                "Replacing {} with br {}",
                inst.dump(&unit),
                target.dump(&unit)
            );
            unit.insert_before(inst);
            unit.ins().br(target);
            unit.delete_inst(inst);
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
                from.dump(&unit),
                to.dump(&unit)
            );
            unit.replace_block_use(from, to);
            // If this is the entry block, hoist the target up as the first block.
            if from == entry {
                unit.swap_blocks(from, to);
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
        let pt = unit.temporal_predtbl();
        let mut merge_blocks = Vec::new();
        let mut already_merged = HashMap::new();
        for bb in unit.blocks().filter(|&bb| bb != entry) {
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
            debug!("Merge {} into {}", block.dump(&unit), into.dump(&unit));
            let term = unit.terminator(into);
            while let Some(inst) = unit.first_inst(block) {
                unit.remove_inst(inst);
                // Do not migrate phi nodes, which at this point have only the
                // `into` block as predecessor and can be trivially replaced.
                if unit[inst].opcode() == Opcode::Phi {
                    assert_eq!(
                        unit[inst].blocks(),
                        &[into],
                        "Phi node must be trivially removable"
                    );
                    let phi = unit.inst_result(inst);
                    let repl = unit[inst].args()[0];
                    unit.replace_use(phi, repl);
                } else {
                    unit.insert_inst_before(inst, term);
                }
            }
            unit.remove_inst(term);
            unit.replace_block_use(block, into);
            unit.delete_block(block);
        }

        modified
    }
}

/// Check if a branch that terminates a block is trivial.
fn check_branch_trivial(
    unit: &UnitBuilder,
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
    trace!("Checking if trivial {}", inst.dump(&unit));

    // Now we know the block is empty. Check for a few common cases of trivial
    // branches.
    let data = &unit[inst];
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
            if let Some(imm) = unit.get_const_int(arg) {
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
    unit: &UnitBuilder,
    block: Block,
    triv_bb: &mut HashMap<Block, Option<Block>>,
    triv_br: &mut HashMap<Inst, Option<Block>>,
) -> Option<Block> {
    trace!("Checking if trivial {}", block.dump(&unit));

    // Check that there are no phi nodes on the target block.
    if unit.insts(block).any(|inst| unit[inst].opcode().is_phi()) {
        triv_bb.insert(block, None);
        return None;
    }

    // If the block is not trivially empty, it is retargetable but cannot be
    // "jumped through".
    if unit.first_inst(block) != unit.last_inst(block) {
        triv_bb.insert(block, Some(block));
        return Some(block);
    }

    // Dig up the terminator instruction and potentially resolve the target to
    // its trivial successor.
    let inst = unit.terminator(block);
    let target = Some(check_branch_trivial(unit, block, inst, triv_bb, triv_br).unwrap_or(block));
    triv_bb.insert(block, target);
    target
}

/// Eliminate unreachable and trivial blocks in a function layout.
fn prune_blocks(unit: &mut UnitBuilder) -> bool {
    let mut modified = false;

    // Find all blocks reachable from the entry point.
    let first_bb = unit.first_block().unwrap();
    let mut unreachable: HashSet<Block> = unit.blocks().collect();
    let mut todo: Vec<Block> = Default::default();
    todo.push(first_bb);
    unreachable.remove(&first_bb);
    while let Some(block) = todo.pop() {
        let term_inst = unit.terminator(block);
        for &bb in unit[term_inst].blocks() {
            if unreachable.remove(&bb) {
                todo.push(bb);
            }
        }
    }

    // Remove all unreachable blocks.
    for bb in unreachable {
        debug!("Prune unreachable block {}", bb.dump(&unit));
        modified |= true;
        unit.delete_block(bb);
    }

    modified
}
