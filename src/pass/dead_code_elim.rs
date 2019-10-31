// Copyright (c) 2017-2019 Fabian Schuiki

//! Dead Code Elimination

use crate::ir::prelude::*;
use crate::ir::InstData;
use crate::opt::prelude::*;
use std::collections::HashSet;

/// Dead Code Elimination
///
/// This pass implements dead code elimination. It removes instructions whose
/// value is never used, trivial blocks, and blocks which cannot be reached.
pub struct DeadCodeElim;

impl Pass for DeadCodeElim {
    fn run_on_function(_ctx: &PassContext, builder: &mut FunctionBuilder) -> bool {
        let mut modified = false;
        let mut insts = vec![];
        for bb in builder.func.layout.blocks() {
            for inst in builder.func.layout.insts(bb) {
                insts.push(inst);
            }
        }
        for inst in insts {
            modified |= builder.prune_if_unused(inst);
        }
        modified |= prune_blocks(builder);
        modified
    }

    fn run_on_process(_ctx: &PassContext, builder: &mut ProcessBuilder) -> bool {
        let mut modified = false;
        let mut insts = vec![];
        for bb in builder.prok.layout.blocks() {
            for inst in builder.prok.layout.insts(bb) {
                insts.push(inst);
            }
        }
        for inst in insts {
            modified |= builder.prune_if_unused(inst);
        }
        modified |= prune_blocks(builder);
        modified
    }

    fn run_on_entity(_ctx: &PassContext, builder: &mut EntityBuilder) -> bool {
        let mut modified = false;
        for inst in builder.entity.layout.insts().collect::<Vec<_>>() {
            modified |= builder.prune_if_unused(inst);
        }
        modified
    }
}

/// Eliminate unreachable and trivial blocks in a function layout.
fn prune_blocks(builder: &mut impl UnitBuilder) -> bool {
    let mut modified = false;

    // Find all trivially empty blocks and cause all predecessors to directly
    // jump to the successor.
    let first_bb = builder.func_layout().first_block().unwrap();
    let mut trivial: Vec<(Block, Block)> = vec![];
    for bb in builder.func_layout().blocks() {
        let first_inst = builder.func_layout().first_inst(bb).unwrap();
        let last_inst = builder.func_layout().last_inst(bb).unwrap();
        if first_inst != last_inst {
            continue;
        }
        match builder.dfg()[last_inst] {
            InstData::Jump { bbs, .. } => {
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
        let term_inst = builder.func_layout().last_inst(block).unwrap();
        for &bb in builder.dfg()[term_inst].blocks() {
            if unreachable.remove(&bb) {
                todo.push(bb);
            }
        }
    }

    // Remove all unreachable blocks.
    for bb in unreachable {
        modified |= true;
        builder.remove_block(bb);
    }

    modified
}
