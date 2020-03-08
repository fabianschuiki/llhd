// Copyright (c) 2017-2019 Fabian Schuiki

//! Instruction and BB ordering.

use crate::{
    ir::{Block, Inst},
    table::SecondaryTable,
};
use std::collections::HashMap;

/// Common functionality between CFG and DFG unit layouts.
pub trait Layout {
    /// Check if an instruction is inserted.
    fn is_inst_inserted(&self, inst: Inst) -> bool;

    /// Check if a block is inserted.
    fn is_block_inserted(&self, block: Block) -> bool;
}

/// Determines the order of instructions and BBs in a `Function` or `Process`.
#[derive(Default, Serialize, Deserialize)]
pub struct FunctionLayout {
    /// A linked list of BBs in layout order.
    pub(crate) bbs: SecondaryTable<Block, BlockNode>,
    /// The first BB in the layout.
    first_bb: Option<Block>,
    /// The last BB in the layout.
    last_bb: Option<Block>,
    /// Lookup table to find the BB that contains an instruction.
    inst_map: HashMap<Inst, Block>,
}

/// A node in the layout's double-linked list of BBs.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct BlockNode {
    prev: Option<Block>,
    next: Option<Block>,
    pub(crate) layout: InstLayout,
}

impl FunctionLayout {
    /// Create a new function layout.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Layout for FunctionLayout {
    fn is_inst_inserted(&self, inst: Inst) -> bool {
        self.inst_map.contains_key(&inst)
    }

    fn is_block_inserted(&self, bb: Block) -> bool {
        self.bbs.contains(bb)
    }
}

/// Basic block arrangement.
///
/// The following functions are used for laying out the basic blocks within a
/// `Function` or `Process`.
impl FunctionLayout {
    /// Append a BB to the end of the function.
    pub fn append_block(&mut self, bb: Block) {
        self.bbs.add(
            bb,
            BlockNode {
                prev: self.last_bb,
                next: None,
                layout: Default::default(),
            },
        );
        if let Some(prev) = self.last_bb {
            self.bbs[prev].next = Some(bb);
        }
        if self.first_bb.is_none() {
            self.first_bb = Some(bb);
        }
        self.last_bb = Some(bb);
    }

    /// Prepend a BB to the beginning of a function.
    ///
    /// This effectively makes `bb` the new entry block.
    pub fn prepend_block(&mut self, bb: Block) {
        self.bbs.add(
            bb,
            BlockNode {
                prev: None,
                next: self.first_bb,
                layout: Default::default(),
            },
        );
        if let Some(next) = self.first_bb {
            self.bbs[next].prev = Some(bb);
        }
        if self.last_bb.is_none() {
            self.last_bb = Some(bb);
        }
        self.first_bb = Some(bb);
    }

    /// Insert a BB after another BB.
    pub fn insert_block_after(&mut self, bb: Block, after: Block) {
        self.bbs.add(
            bb,
            BlockNode {
                prev: Some(after),
                next: self.bbs[after].next,
                layout: Default::default(),
            },
        );
        if let Some(next) = self.bbs[after].next {
            self.bbs[next].prev = Some(bb);
        }
        self.bbs[after].next = Some(bb);
        if self.last_bb == Some(after) {
            self.last_bb = Some(bb);
        }
    }

    /// Insert a BB before another BB.
    pub fn insert_block_before(&mut self, bb: Block, before: Block) {
        self.bbs.add(
            bb,
            BlockNode {
                prev: self.bbs[before].prev,
                next: Some(before),
                layout: Default::default(),
            },
        );
        if let Some(prev) = self.bbs[before].prev {
            self.bbs[prev].next = Some(bb);
        }
        self.bbs[before].prev = Some(bb);
        if self.first_bb == Some(before) {
            self.first_bb = Some(bb);
        }
    }

    /// Remove a BB from the function.
    pub fn remove_block(&mut self, bb: Block) {
        let node = self.bbs.remove(bb).unwrap();
        if let Some(next) = node.next {
            self.bbs[next].prev = node.prev;
        }
        if let Some(prev) = node.prev {
            self.bbs[prev].next = node.next;
        }
        if self.first_bb == Some(bb) {
            self.first_bb = node.next;
        }
        if self.last_bb == Some(bb) {
            self.last_bb = node.prev;
        }
    }

    /// Swap the position of two BBs.
    pub fn swap_blocks(&mut self, bb0: Block, bb1: Block) {
        if bb0 == bb1 {
            return;
        }

        let mut bb0_next = self.bbs[bb0].next;
        let mut bb0_prev = self.bbs[bb0].prev;
        let mut bb1_next = self.bbs[bb1].next;
        let mut bb1_prev = self.bbs[bb1].prev;
        if bb0_next == Some(bb1) {
            bb0_next = Some(bb0);
        }
        if bb0_prev == Some(bb1) {
            bb0_prev = Some(bb0);
        }
        if bb1_next == Some(bb0) {
            bb1_next = Some(bb1);
        }
        if bb1_prev == Some(bb0) {
            bb1_prev = Some(bb1);
        }
        self.bbs[bb0].next = bb1_next;
        self.bbs[bb0].prev = bb1_prev;
        self.bbs[bb1].next = bb0_next;
        self.bbs[bb1].prev = bb0_prev;

        if let Some(next) = bb0_next {
            self.bbs[next].prev = Some(bb1);
        }
        if let Some(prev) = bb0_prev {
            self.bbs[prev].next = Some(bb1);
        }
        if let Some(next) = bb1_next {
            self.bbs[next].prev = Some(bb0);
        }
        if let Some(prev) = bb1_prev {
            self.bbs[prev].next = Some(bb0);
        }

        if self.first_bb == Some(bb0) {
            self.first_bb = Some(bb1);
        } else if self.first_bb == Some(bb1) {
            self.first_bb = Some(bb0);
        }
        if self.last_bb == Some(bb0) {
            self.last_bb = Some(bb1);
        } else if self.last_bb == Some(bb1) {
            self.last_bb = Some(bb0);
        }
    }

    /// Return an iterator over all BBs in layout order.
    pub fn blocks<'a>(&'a self) -> impl Iterator<Item = Block> + 'a {
        std::iter::successors(self.first_bb, move |&bb| self.next_block(bb))
    }

    /// Get the first BB in the layout. This is the entry block.
    pub fn first_block(&self) -> Option<Block> {
        self.first_bb
    }

    /// Get the last BB in the layout.
    pub fn last_block(&self) -> Option<Block> {
        self.last_bb
    }

    /// Get the BB preceding `bb` in the layout.
    pub fn prev_block(&self, bb: Block) -> Option<Block> {
        self.bbs[bb].prev
    }

    /// Get the BB following `bb` in the layout.
    pub fn next_block(&self, bb: Block) -> Option<Block> {
        self.bbs[bb].next
    }

    /// Get the entry block in the layout.
    ///
    /// The fallible alternative is `first_block(bb)`.
    pub fn entry(&self) -> Block {
        self.first_block().expect("entry block is required")
    }
}

/// Determines the order of instructions.
#[derive(Default, Serialize, Deserialize)]
pub struct InstLayout {
    /// A linked list of instructions in layout order.
    insts: SecondaryTable<Inst, InstNode>,
    /// The first instruction in the layout.
    first_inst: Option<Inst>,
    /// The last instruction in the layout.
    last_inst: Option<Inst>,
}

/// A node in the layout's double-linked list of BBs.
#[derive(Default, Serialize, Deserialize)]
struct InstNode {
    prev: Option<Inst>,
    next: Option<Inst>,
}

impl Layout for InstLayout {
    fn is_inst_inserted(&self, inst: Inst) -> bool {
        self.insts.contains(inst)
    }

    fn is_block_inserted(&self, _: Block) -> bool {
        false
    }
}

impl InstLayout {
    /// Create a new instruction layout.
    pub fn new() -> Self {
        Default::default()
    }

    /// Append an instruction to the end of the function.
    pub fn append_inst(&mut self, inst: Inst) {
        self.insts.add(
            inst,
            InstNode {
                prev: self.last_inst,
                next: None,
            },
        );
        if let Some(prev) = self.last_inst {
            self.insts[prev].next = Some(inst);
        }
        if self.first_inst.is_none() {
            self.first_inst = Some(inst);
        }
        self.last_inst = Some(inst);
    }

    /// Prepend an instruction to the beginning of the function.
    pub fn prepend_inst(&mut self, inst: Inst) {
        self.insts.add(
            inst,
            InstNode {
                prev: None,
                next: self.first_inst,
            },
        );
        if let Some(next) = self.first_inst {
            self.insts[next].prev = Some(inst);
        }
        if self.last_inst.is_none() {
            self.last_inst = Some(inst);
        }
        self.first_inst = Some(inst);
    }

    /// Insert an instruction after another instruction.
    pub fn insert_inst_after(&mut self, inst: Inst, after: Inst) {
        self.insts.add(
            inst,
            InstNode {
                prev: Some(after),
                next: self.insts[after].next,
            },
        );
        if let Some(next) = self.insts[after].next {
            self.insts[next].prev = Some(inst);
        }
        self.insts[after].next = Some(inst);
        if self.last_inst == Some(after) {
            self.last_inst = Some(inst);
        }
    }

    /// Insert an instruction before another instruction.
    pub fn insert_inst_before(&mut self, inst: Inst, before: Inst) {
        self.insts.add(
            inst,
            InstNode {
                prev: self.insts[before].prev,
                next: Some(before),
            },
        );
        if let Some(prev) = self.insts[before].prev {
            self.insts[prev].next = Some(inst);
        }
        self.insts[before].prev = Some(inst);
        if self.first_inst == Some(before) {
            self.first_inst = Some(inst);
        }
    }

    /// Remove an instruction from the function.
    pub fn remove_inst(&mut self, inst: Inst) {
        let node = self.insts.remove(inst).unwrap();
        if let Some(next) = node.next {
            self.insts[next].prev = node.prev;
        }
        if let Some(prev) = node.prev {
            self.insts[prev].next = node.next;
        }
        if self.first_inst == Some(inst) {
            self.first_inst = node.next;
        }
        if self.last_inst == Some(inst) {
            self.last_inst = node.prev;
        }
    }

    /// Return an iterator over all instructions in layout order.
    pub fn insts<'a>(&'a self) -> impl Iterator<Item = Inst> + 'a {
        std::iter::successors(self.first_inst, move |&inst| self.next_inst(inst))
    }

    /// Get the first instruction in the layout.
    pub fn first_inst(&self) -> Option<Inst> {
        self.first_inst
    }

    /// Get the last instruction in the layout.
    pub fn last_inst(&self) -> Option<Inst> {
        self.last_inst
    }

    /// Get the instruction preceding `inst` in the layout.
    pub fn prev_inst(&self, inst: Inst) -> Option<Inst> {
        self.insts[inst].prev
    }

    /// Get the instruction following `inst` in the layout.
    pub fn next_inst(&self, inst: Inst) -> Option<Inst> {
        self.insts[inst].next
    }
}

/// Instruction arrangement.
///
/// The following functions are used for laying out the instructions within a
/// `Function` or `Process`.
impl FunctionLayout {
    /// Get the BB which contains `inst`, or `None` if `inst` is not inserted.
    pub fn inst_block(&self, inst: Inst) -> Option<Block> {
        self.inst_map.get(&inst).cloned()
    }

    /// Append an instruction to the end of a BB.
    pub fn append_inst(&mut self, inst: Inst, bb: Block) {
        self.bbs[bb].layout.append_inst(inst);
        self.inst_map.insert(inst, bb);
    }

    /// Prepend an instruction to the beginning of a BB.
    pub fn prepend_inst(&mut self, inst: Inst, bb: Block) {
        self.bbs[bb].layout.prepend_inst(inst);
        self.inst_map.insert(inst, bb);
    }

    /// Insert an instruction after another instruction.
    pub fn insert_inst_after(&mut self, inst: Inst, after: Inst) {
        let bb = self.inst_block(after).expect("`after` not inserted");
        self.bbs[bb].layout.insert_inst_after(inst, after);
        self.inst_map.insert(inst, bb);
    }

    /// Insert an instruction before another instruction.
    pub fn insert_inst_before(&mut self, inst: Inst, before: Inst) {
        let bb = self.inst_block(before).expect("`before` not inserted");
        self.bbs[bb].layout.insert_inst_before(inst, before);
        self.inst_map.insert(inst, bb);
    }

    /// Remove an instruction from the function.
    pub fn remove_inst(&mut self, inst: Inst) {
        let bb = self.inst_block(inst).expect("`inst` not inserted");
        self.bbs[bb].layout.remove_inst(inst);
        self.inst_map.remove(&inst);
    }

    /// Return an iterator over all instructions in a block in layout order.
    pub fn insts<'a>(&'a self, bb: Block) -> impl Iterator<Item = Inst> + 'a {
        self.bbs[bb].layout.insts()
    }

    /// Get the first instruction in the layout.
    pub fn first_inst(&self, bb: Block) -> Option<Inst> {
        self.bbs[bb].layout.first_inst()
    }

    /// Get the last instruction in the layout.
    pub fn last_inst(&self, bb: Block) -> Option<Inst> {
        self.bbs[bb].layout.last_inst()
    }

    /// Get the instruction preceding `inst` in the layout.
    pub fn prev_inst(&self, inst: Inst) -> Option<Inst> {
        let bb = self.inst_block(inst).unwrap();
        self.bbs[bb].layout.prev_inst(inst)
    }

    /// Get the instruction following `inst` in the layout.
    pub fn next_inst(&self, inst: Inst) -> Option<Inst> {
        let bb = self.inst_block(inst).unwrap();
        self.bbs[bb].layout.next_inst(inst)
    }

    /// Get the terminator instruction in the layout.
    ///
    /// The fallible alternative is `last_inst(bb)`.
    pub fn terminator(&self, bb: Block) -> Inst {
        self.last_inst(bb).expect("block must have terminator")
    }
}
