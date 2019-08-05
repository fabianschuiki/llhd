// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of the control flow in a `Function` or `Process`.
//!
//! Each `Function` or `Process` in LLHD has an associated `ControlFlowGraph`
//! which contains the basic blocks, dominator tree, and related information.

use crate::{
    impl_table_indexing,
    ir::{Block, BlockData},
    table::PrimaryTable,
};

/// A control flow graph.
///
/// This is the main container for BBs and control flow related information.
/// Every `Function` and `Process` has an associated control flow graph.
#[derive(Default)]
pub struct ControlFlowGraph {
    /// The basic blocks in the graph.
    pub(crate) blocks: PrimaryTable<Block, BlockData>,
}

impl_table_indexing!(ControlFlowGraph, blocks, Block, BlockData);

impl ControlFlowGraph {
    /// Create a new control flow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a BB to the graph.
    pub fn add_block(&mut self) -> Block {
        self.blocks.add(BlockData { name: None })
    }

    /// Remove a BB from the graph.
    pub fn remove_block(&mut self, bb: Block) {
        self.blocks.remove(bb);
    }

    /// Return the name of a BB.
    pub fn get_name(&self, bb: Block) -> Option<&str> {
        self[bb].name.as_ref().map(AsRef::as_ref)
    }

    /// Set the name of a BB.
    pub fn set_name(&mut self, bb: Block, name: String) {
        self[bb].name = Some(name);
    }

    /// Clear the name of a BB.
    pub fn clear_name(&mut self, bb: Block) -> Option<String> {
        std::mem::replace(&mut self[bb].name, None)
    }
}
