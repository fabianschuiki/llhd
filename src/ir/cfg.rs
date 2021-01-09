// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of the control flow in a `Function` or `Process`.
//!
//! Each `Function` or `Process` in LLHD has an associated `ControlFlowGraph`
//! which contains the basic blocks, dominator tree, and related information.

use crate::{
    impl_table_indexing,
    ir::{Block, BlockData},
    table::PrimaryTable2,
};
use std::collections::HashMap;

/// A control flow graph.
///
/// This is the main container for BBs and control flow related information.
/// Every `Function` and `Process` has an associated control flow graph.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct ControlFlowGraph {
    /// The basic blocks in the graph.
    pub blocks: PrimaryTable2<Block, BlockData>,
    /// The anonymous name hints assigned to basic blocks.
    pub anonymous_hints: HashMap<Block, u32>,
}

impl_table_indexing!(ControlFlowGraph, blocks, Block, BlockData);
