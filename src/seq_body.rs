// Copyright (c) 2017 Fabian Schuiki

use crate::block::*;
use crate::inst::*;
use crate::util::IndirectMapIter;
use crate::value::{BlockRef, InstRef};
use std;
use std::collections::HashMap;

/// A sequential body of blocks and instructions. Represents a control flow
/// graph as describe by a process or function, i.e. a sequential arrangement of
/// instructions. This in contrast to the dataflow body of an entity.
pub struct SeqBody {
    blocks: HashMap<BlockRef, Block>,
    block_seq: Vec<BlockRef>,
    block_of_inst: HashMap<InstRef, BlockRef>,
    insts: HashMap<InstRef, Inst>,
}

impl SeqBody {
    /// Create a new sequential body.
    pub fn new() -> SeqBody {
        SeqBody {
            blocks: HashMap::new(),
            block_seq: Vec::new(),
            block_of_inst: HashMap::new(),
            insts: HashMap::new(),
        }
    }

    /// Add a block to the body.
    pub fn add_block(&mut self, block: Block, pos: BlockPosition) -> BlockRef {
        let br = block.as_ref();
        self.blocks.insert(br, block);
        self.insert_block(br, pos, false);
        br
    }

    /// Move a block around within the body.
    pub fn move_block(&mut self, block: BlockRef, pos: BlockPosition) {
        self.detach_block(block, true);
        self.insert_block(block, pos, true);
    }

    /// Remove a block from the body.
    pub fn remove_block(&mut self, block: BlockRef) {
        self.detach_block(block, false);
        self.blocks.remove(&block);
    }

    /// Insert a block into the body at the requested position. The block must
    /// be detached, i.e. either it is a fresh block or `detach_block` has been
    /// called before. `just_move` indicates whether the block has been attached
    /// to the body before, and prevents the lookup tables to be changed as an
    /// optimization.
    fn insert_block(&mut self, block: BlockRef, pos: BlockPosition, just_move: bool) {
        let index = match pos {
            BlockPosition::Begin => 0,
            BlockPosition::End => self.block_seq.len(),
            BlockPosition::Before(b) => self.block_pos(b),
            BlockPosition::After(b) => self.block_pos(b) + 1,
        };
        self.block_seq.insert(index, block);

        // Update the inst-to-block mapping if this was not just a move.
        if !just_move {
            let insts: Vec<InstRef> = self.blocks[&block].inst_refs().map(|r| *r).collect();
            self.block_of_inst.extend(insts.iter().map(|r| (*r, block)));
        }
    }

    /// Detach a block from the body. `just_move` indicates whether the block
    /// will be attached again immediately after this function, and prevents the
    /// lookup tables to be changed as an optimization.
    fn detach_block(&mut self, block: BlockRef, just_move: bool) {
        let pos = self.block_pos(block);
        self.block_seq.remove(pos);

        // Update the inst-to-block mapping if this was not just a move.
        if !just_move {
            let insts: Vec<InstRef> = self.blocks[&block].inst_refs().map(|r| *r).collect();
            for inst in insts {
                self.block_of_inst.remove(&inst);
            }
        }
    }

    /// Determine the index at which a certain block is located. Panics if the
    /// block is not part of the body.
    fn block_pos(&self, block: BlockRef) -> usize {
        self.block_seq
            .iter()
            .position(|&b| b == block)
            .expect("body does not contain basic block")
    }

    /// Add an instruction to the body.
    pub fn add_inst(&mut self, inst: Inst, pos: InstPosition) -> InstRef {
        let ir = inst.as_ref();
        self.insts.insert(ir, inst);
        self.insert_inst(ir, pos);
        ir
    }

    /// Move an instruction around within the body.
    pub fn move_inst(&mut self, inst: InstRef, pos: InstPosition) {
        self.detach_inst(inst);
        self.insert_inst(inst, pos);
    }

    /// Remove an instruction from the body.
    pub fn remove_inst(&mut self, inst: InstRef) {
        self.detach_inst(inst);
        self.insts.remove(&inst);
    }

    /// Insert an instruction into the basic block dictated by the requested
    /// position. The instruction must be detached, i.e. either it is a fresh
    /// instruction or `detach_inst` has been called before.
    fn insert_inst(&mut self, inst: InstRef, pos: InstPosition) {
        let block = match pos {
            InstPosition::Begin => self
                .blocks
                .get_mut(self.block_seq.first().unwrap())
                .unwrap(),
            InstPosition::End => self.blocks.get_mut(self.block_seq.last().unwrap()).unwrap(),
            InstPosition::BlockBegin(b) | InstPosition::BlockEnd(b) => {
                self.blocks.get_mut(&b).unwrap()
            }
            InstPosition::Before(i) | InstPosition::After(i) => {
                self.blocks.get_mut(&self.block_of_inst[&i]).unwrap()
            }
        };
        block.insert_inst(inst, pos);
        self.block_of_inst.insert(inst, block.as_ref());
    }

    /// Detach an instruction from its current basic block.
    fn detach_inst(&mut self, inst: InstRef) {
        self.block_of_inst.remove(&inst);
        self.blocks
            .get_mut(&self.block_of_inst[&inst])
            .unwrap()
            .detach_inst(inst)
    }

    /// Obtain an iterator over the blocks in this body.
    pub fn blocks(&self) -> IndirectMapIter<std::slice::Iter<BlockRef>, Block> {
        IndirectMapIter::new(self.block_seq.iter(), &self.blocks)
    }

    /// Get a reference to a block in the body. Panics if the block does not
    /// exist.
    pub fn block(&self, block: BlockRef) -> &Block {
        self.blocks.get(&block).unwrap()
    }

    /// Get a mutable reference to a block in the body. Panics if the block does
    /// not exist.
    pub fn block_mut(&mut self, block: BlockRef) -> &mut Block {
        self.blocks.get_mut(&block).unwrap()
    }

    /// Get a reference to an instruction in the body. Panics if the instruction
    /// does not exist.
    pub fn inst(&self, inst: InstRef) -> &Inst {
        self.insts.get(&inst).unwrap()
    }

    /// Get a mutable reference to an instruction in the body. Panics if the
    /// instruction does not exist.
    pub fn inst_mut(&mut self, inst: InstRef) -> &mut Inst {
        self.insts.get_mut(&inst).unwrap()
    }
}
