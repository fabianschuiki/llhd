// Copyright (c) 2017-2019 Fabian Schuiki

//! Dead Code Elimination

use crate::ir::prelude::*;
use crate::ir::InstData;
use crate::opt::prelude::*;
use std::collections::{HashMap, HashSet};

/// Dead Code Elimination
///
/// This pass implements dead code elimination. It removes instructions whose
/// value is never used, trivial blocks, and blocks which cannot be reached.
pub struct DeadCodeElim;

impl Pass for DeadCodeElim {
    fn run_on_cfg(_ctx: &PassContext, builder: &mut impl UnitBuilder) -> bool {
        info!("DCE [{}]", builder.unit().name());
        let mut modified = false;
        let mut insts = vec![];
        for bb in builder.func_layout().blocks() {
            for inst in builder.func_layout().insts(bb) {
                insts.push(inst);
            }
        }
        for inst in insts {
            modified |= fold_inst(inst, builder);
            modified |= builder.prune_if_unused(inst);
        }
        modified |= prune_blocks(builder);
        modified
    }

    fn run_on_entity(_ctx: &PassContext, builder: &mut EntityBuilder) -> bool {
        info!("DCE [{}]", builder.unit().name());
        let mut modified = false;
        for inst in builder.entity.layout.insts().collect::<Vec<_>>() {
            modified |= builder.prune_if_unused(inst);
        }
        modified
    }
}

fn fold_inst(inst: Inst, builder: &mut impl UnitBuilder) -> bool {
    // Fold branches.
    if let InstData::Branch {
        opcode: Opcode::BrCond,
        args,
        bbs,
    } = builder.dfg()[inst]
    {
        return fold_branch(builder, inst, args[0], bbs).unwrap_or(false);
    }
    false
}

/// Fold a branch instruction.
///
/// If the branch's condition is a constant, replaces the branch with a jump to
/// the corresponding target.
fn fold_branch(
    builder: &mut impl UnitBuilder,
    inst: Inst,
    arg: Value,
    bbs: [Block; 2],
) -> Option<bool> {
    let imm = builder.dfg().get_const_int(arg)?;
    let bb = bbs[!imm.is_zero() as usize];
    debug!(
        "Replacing {} with br {}",
        inst.dump(builder.dfg(), builder.try_cfg()),
        bb.dump(builder.cfg())
    );
    builder.insert_before(inst);
    builder.ins().br(bb);
    builder.remove_inst(inst);
    if let Some(arg_inst) = builder.dfg().get_value_inst(arg) {
        builder.prune_if_unused(arg_inst);
    }
    Some(true)
}

/// Eliminate unreachable and trivial blocks in a function layout.
fn prune_blocks(builder: &mut impl UnitBuilder) -> bool {
    let mut modified = false;

    // Find all trivially empty blocks and cause all predecessors to directly
    // jump to the successor.
    let first_bb = builder.func_layout().first_block().unwrap();
    let mut incident_phi_edges = HashMap::<Block, HashSet<Block>>::new();
    let mut trivial: Vec<(Block, Block)> = vec![];
    for bb in builder.func_layout().blocks() {
        let first_inst = builder.func_layout().first_inst(bb).unwrap();
        let last_inst = builder.func_layout().last_inst(bb).unwrap();
        if first_inst != last_inst {
            continue;
        }
        match builder.dfg()[last_inst] {
            InstData::Jump { bbs, .. } => {
                // Make sure that this branch is not part of a phi edge.
                let edges = incident_phi_edges.entry(bbs[0]).or_insert_with(|| {
                    let mut edges = HashSet::new();
                    for inst in builder.func_layout().insts(bbs[0]) {
                        if builder.dfg()[inst].opcode().is_phi() {
                            edges.extend(builder.dfg()[inst].blocks());
                        }
                    }
                    edges
                });
                if edges.contains(&bb) {
                    continue;
                }

                for (_, to) in &mut trivial {
                    if *to == bb {
                        *to = bbs[0];
                    }
                }
                trivial.push((bb, bbs[0]));
            }
            _ => continue,
        }
    }
    for (from, to) in trivial {
        debug!("Prune trivial block {}", from.dump(builder.cfg()));
        modified |= true;
        builder.dfg_mut().replace_block_use(from, to);
        // If this is the entry block, hoist the target up as the first block.
        if from == first_bb {
            builder.func_layout_mut().swap_blocks(from, to);
        }
    }

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
