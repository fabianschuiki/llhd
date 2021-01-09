// Copyright (c) 2017-2021 Fabian Schuiki

//! Instruction and BB ordering.

use crate::{
    ir::{Block, Inst},
    table::SecondaryTable,
};
use std::collections::HashMap;

/// Determines the order of instructions and BBs in a `Function` or `Process`.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct FunctionLayout {
    /// A linked list of BBs in layout order.
    pub(super) bbs: SecondaryTable<Block, BlockNode>,
    /// The first BB in the layout.
    pub(super) first_bb: Option<Block>,
    /// The last BB in the layout.
    pub(super) last_bb: Option<Block>,
    /// Lookup table to find the BB that contains an instruction.
    pub(super) inst_map: HashMap<Inst, Block>,
}

/// A node in the layout's double-linked list of BBs.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct BlockNode {
    pub(super) prev: Option<Block>,
    pub(super) next: Option<Block>,
    pub(super) layout: InstLayout,
}

/// Determines the order of instructions.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct InstLayout {
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

impl FunctionLayout {
    /// Add a mapping from an instruction to the block that contains it.
    pub(super) fn map_inst(&mut self, inst: Inst, bb: Block) {
        match self.inst_map.insert(inst, bb) {
            Some(old_bb) => panic!(
                "inst {} already inserted in {}, now being inserted into {}",
                inst, old_bb, bb
            ),
            None => (),
        }
    }

    /// Remove a mapping from an instruction to the block that contains it.
    pub(super) fn unmap_inst(&mut self, inst: Inst) {
        match self.inst_map.remove(&inst) {
            Some(_) => (),
            None => panic!("inst {} was not inserted"),
        }
    }
}

impl InstLayout {
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
