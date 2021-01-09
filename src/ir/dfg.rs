// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of the data flow in a `Function`, `Process`, or `Entity`.
//!
//! Each unit in LLHD has an associated `DataFlowGraph` which contains all the
//! values, instructions, arguments, and links between them.

use crate::{
    impl_table_indexing,
    ir::{Arg, Block, ExtUnit, ExtUnitData, Inst, InstData, Value, ValueData},
    table::{PrimaryTable2, SecondaryTable},
};
use std::collections::{HashMap, HashSet};

/// A data flow graph.
///
/// This is the main container for instructions, values, and the relationship
/// between them. Every `Function`, `Process`, and `Entity` has an associated
/// data flow graph.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct DataFlowGraph {
    /// The instructions in the graph.
    pub insts: PrimaryTable2<Inst, InstData>,
    /// The result values produced by instructions.
    pub results: SecondaryTable<Inst, Value>,
    /// The values in the graph.
    pub values: PrimaryTable2<Value, ValueData>,
    /// The argument values.
    pub args: SecondaryTable<Arg, Value>,
    /// The external units in the graph.
    pub ext_units: PrimaryTable2<ExtUnit, ExtUnitData>,
    /// The names assigned to values.
    pub names: HashMap<Value, String>,
    /// The anonymous name hints assigned to values.
    pub anonymous_hints: HashMap<Value, u32>,
    /// The location hints assigned to instructions.
    pub location_hints: HashMap<Inst, usize>,
    /// The value use lookup table.
    pub value_uses: HashMap<Value, HashSet<Inst>>,
    /// The block use lookup table.
    pub block_uses: HashMap<Block, HashSet<Inst>>,
}

impl_table_indexing!(DataFlowGraph, insts, Inst, InstData);
impl_table_indexing!(DataFlowGraph, values, Value, ValueData);
impl_table_indexing!(DataFlowGraph, ext_units, ExtUnit, ExtUnitData);
