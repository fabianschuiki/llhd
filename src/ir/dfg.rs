// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of the data flow in a `Function`, `Process`, or `Entity`.
//!
//! Each unit in LLHD has an associated `DataFlowGraph` which contains all the
//! values, instructions, arguments, and links between them.

use crate::{
    impl_table_indexing,
    ir::{Arg, ExtUnit, ExtUnitData, Inst, InstData, Signature, Value, ValueData},
    table::{PrimaryTable, SecondaryTable, TableKey},
    ty::Type,
};
use std::collections::HashMap;

/// A data flow graph.
///
/// This is the main container for instructions, values, and the relationship
/// between them. Every `Function`, `Process`, and `Entity` has an associated
/// data flow graph.
#[derive(Default)]
pub struct DataFlowGraph {
    /// The instructions in the graph.
    pub(crate) insts: PrimaryTable<Inst, InstData>,
    /// The result values produced by instructions.
    pub(crate) results: SecondaryTable<Inst, Value>,
    /// The values in the graph.
    pub(crate) values: PrimaryTable<Value, ValueData>,
    /// The argument values.
    pub(crate) args: SecondaryTable<Arg, Value>,
    /// The external units in the graph.
    pub(crate) ext_units: PrimaryTable<ExtUnit, ExtUnitData>,
    /// The names assigned to values.
    pub(crate) names: HashMap<Value, String>,
}

impl_table_indexing!(DataFlowGraph, insts, Inst, InstData);
impl_table_indexing!(DataFlowGraph, values, Value, ValueData);
impl_table_indexing!(DataFlowGraph, ext_units, ExtUnit, ExtUnitData);

impl DataFlowGraph {
    /// Create a new data flow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a placeholder value.
    ///
    /// This function is intended to be used when constructing PHI nodes.
    pub fn add_placeholder(&mut self, ty: Type) -> Value {
        self.values.add(ValueData::Placeholder { ty })
    }

    /// Remove a placeholder value.
    pub fn remove_placeholder(&mut self, value: Value) {
        assert!(!self.has_uses(value));
        assert!(self[value].is_placeholder());
        self.values.remove(value);
    }

    /// Add an instruction.
    pub fn add_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.insts.add(data);
        if !ty.is_void() {
            let result = self.values.add(ValueData::Inst { ty, inst });
            self.results.add(inst, result);
        }
        inst
    }

    /// Remove an instruction.
    pub fn remove_inst(&mut self, inst: Inst) {
        let value = self.inst_result(inst);
        assert!(!self.has_uses(value));
        self.insts.remove(inst);
        self.results.remove(inst);
        self.values.remove(value);
    }

    /// Returns whether an instruction produces a result.
    pub fn has_result(&self, inst: Inst) -> bool {
        self.results.storage.contains_key(&inst.index())
    }

    /// Returns the result of an instruction.
    pub fn inst_result(&self, inst: Inst) -> Value {
        self.results[inst]
    }

    /// Returns the value of an argument.
    pub fn arg_value(&self, arg: Arg) -> Value {
        self.args[arg]
    }

    /// Create values for the arguments in a signature.
    pub(crate) fn make_args_for_signature(&mut self, sig: &Signature) {
        for arg in sig.args() {
            let value = self.values.add(ValueData::Arg {
                ty: sig.arg_type(arg),
                arg: arg,
            });
            self.args.add(arg, value);
        }
    }

    /// Returns the type of a value.
    pub fn value_type(&self, value: Value) -> Type {
        match &self[value] {
            ValueData::Inst { ty, .. } => ty.clone(),
            ValueData::Arg { ty, .. } => ty.clone(),
            ValueData::Placeholder { ty, .. } => ty.clone(),
        }
    }

    /// Return the instruction that produces `value`.
    pub fn get_value_inst(&self, value: Value) -> Option<Inst> {
        match self[value] {
            ValueData::Inst { inst, .. } => Some(inst),
            _ => None,
        }
    }

    /// Return the instruction that produces `value`, or panic.
    pub fn value_inst(&self, value: Value) -> Inst {
        match self.get_value_inst(value) {
            Some(inst) => inst,
            None => panic!("value {} not the result of an instruction", value),
        }
    }

    /// Return the name of a value.
    pub fn get_name(&self, value: Value) -> Option<&String> {
        self.names.get(&value)
    }

    /// Set the name of a value.
    pub fn set_name(&mut self, value: Value, name: String) {
        self.names.insert(value, name);
    }

    /// Clear the name of a value.
    pub fn clear_name(&mut self, value: Value) -> Option<String> {
        self.names.remove(&value)
    }

    /// Replace all uses of a value with another.
    ///
    /// Returns how many uses were replaced.
    pub fn replace_use(&mut self, from: Value, to: Value) -> usize {
        let mut count = 0;
        for inst in self.insts.storage.values_mut() {
            count += inst.replace_value(from, to);
        }
        count
    }

    /// Iterate over all uses of a value.
    pub fn uses(&self, value: Value) -> impl Iterator<Item = (Inst, usize)> {
        let mut uses = vec![];
        for inst in self.insts.keys() {
            for (i, arg) in self[inst].args().iter().cloned().enumerate() {
                if arg == value {
                    uses.push((inst, i));
                }
            }
        }
        uses.into_iter()
    }

    /// Check if a value is used.
    pub fn has_uses(&self, value: Value) -> bool {
        self.uses(value).count() > 0
    }

    /// Check if a value has exactly one use.
    pub fn has_one_use(&self, value: Value) -> bool {
        self.uses(value).count() == 1
    }
}
