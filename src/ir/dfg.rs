// Copyright (c) 2017-2020 Fabian Schuiki

//! Representation of the data flow in a `Function`, `Process`, or `Entity`.
//!
//! Each unit in LLHD has an associated `DataFlowGraph` which contains all the
//! values, instructions, arguments, and links between them.

use crate::{
    impl_table_indexing,
    ir::{Arg, Block, ExtUnit, ExtUnitData, Inst, InstData, Signature, Value, ValueData},
    table::{PrimaryTable2, SecondaryTable, TableKey},
    ty::{void_ty, Type},
};
use std::collections::{HashMap, HashSet};

/// A data flow graph.
///
/// This is the main container for instructions, values, and the relationship
/// between them. Every `Function`, `Process`, and `Entity` has an associated
/// data flow graph.
#[derive(Default, Serialize, Deserialize)]
pub struct DataFlowGraph {
    /// The instructions in the graph.
    pub(crate) insts: PrimaryTable2<Inst, InstData>,
    /// The result values produced by instructions.
    pub(crate) results: SecondaryTable<Inst, Value>,
    /// The values in the graph.
    pub(crate) values: PrimaryTable2<Value, ValueData>,
    /// The argument values.
    pub(crate) args: SecondaryTable<Arg, Value>,
    /// The external units in the graph.
    pub(crate) ext_units: PrimaryTable2<ExtUnit, ExtUnitData>,
    /// The names assigned to values.
    pub(crate) names: HashMap<Value, String>,
    /// The anonymous name hints assigned to values.
    pub(crate) anonymous_hints: HashMap<Value, u32>,
    /// The location hints assigned to instructions.
    pub(crate) location_hints: HashMap<Inst, usize>,
    /// The value use lookup table.
    pub(crate) value_uses: HashMap<Value, HashSet<Inst>>,
    /// The block use lookup table.
    pub(crate) block_uses: HashMap<Block, HashSet<Inst>>,
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
        self.add_value(ValueData::Placeholder { ty })
    }

    /// Remove a placeholder value.
    pub fn remove_placeholder(&mut self, value: Value) {
        assert!(!self.has_uses(value));
        assert!(self[value].is_placeholder());
        self.remove_value(value);
    }

    /// Check if a value is a placeholder.
    pub fn is_placeholder(&self, value: Value) -> bool {
        self[value].is_placeholder()
    }

    /// Add a value.
    fn add_value(&mut self, data: ValueData) -> Value {
        let v = self.values.add(data);
        self.value_uses.insert(v, Default::default());
        v
    }

    /// Remove a value.
    fn remove_value(&mut self, value: Value) -> ValueData {
        let data = self.values.remove(value);
        self.value_uses.remove(&value);
        data
    }

    /// Register a value use.
    fn update_uses(&mut self, inst: Inst) {
        for value in self[inst].args().to_vec() {
            self.value_uses.entry(value).or_default().insert(inst);
        }
        for block in self[inst].blocks().to_vec() {
            self.block_uses.entry(block).or_default().insert(inst);
        }
    }

    /// Remove a value use.
    fn remove_uses(&mut self, inst: Inst, data: InstData) {
        for value in data.args() {
            self.value_uses.get_mut(value).unwrap().remove(&inst);
        }
        for block in data.blocks() {
            self.block_uses.get_mut(block).unwrap().remove(&inst);
        }
    }

    /// Add an instruction.
    pub fn add_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.insts.add(data);
        if !ty.is_void() {
            let result = self.add_value(ValueData::Inst { ty, inst });
            self.results.add(inst, result);
        }
        self.update_uses(inst);
        inst
    }

    /// Remove an instruction.
    pub fn remove_inst(&mut self, inst: Inst) {
        if self.has_result(inst) {
            let value = self.inst_result(inst);
            assert!(!self.has_uses(value));
            self.remove_value(value);
        }
        let data = self.insts.remove(inst);
        self.remove_uses(inst, data);
        self.results.remove(inst);
    }

    /// Returns whether an instruction produces a result.
    pub fn has_result(&self, inst: Inst) -> bool {
        self.results.storage.contains_key(&inst.index())
    }

    /// Returns the result of an instruction.
    pub fn inst_result(&self, inst: Inst) -> Value {
        self.results[inst]
    }

    /// Returns the result of an instruction.
    pub fn get_inst_result(&self, inst: Inst) -> Option<Value> {
        self.results.get(inst).cloned()
    }

    /// Returns the value of an argument.
    pub fn arg_value(&self, arg: Arg) -> Value {
        self.args[arg]
    }

    /// Create values for the arguments in a signature.
    pub(crate) fn make_args_for_signature(&mut self, sig: &Signature) {
        for arg in sig.args() {
            let value = self.add_value(ValueData::Arg {
                ty: sig.arg_type(arg),
                arg: arg,
            });
            self.args.add(arg, value);
        }
    }

    /// Returns the type of a value.
    pub fn value_type(&self, value: Value) -> Type {
        match &self[value] {
            ValueData::Invalid => panic!("invalid value"),
            ValueData::Inst { ty, .. } => ty.clone(),
            ValueData::Arg { ty, .. } => ty.clone(),
            ValueData::Placeholder { ty, .. } => ty.clone(),
        }
    }

    /// Returns the type of an instruction.
    pub fn inst_type(&self, inst: Inst) -> Type {
        if self.has_result(inst) {
            self.value_type(self.inst_result(inst))
        } else {
            void_ty()
        }
    }

    /// Return the argument that produces `value`.
    pub fn get_value_arg(&self, value: Value) -> Option<Arg> {
        match self[value] {
            ValueData::Arg { arg, .. } => Some(arg),
            _ => None,
        }
    }

    /// Return the argument that produces `value`, or panic.
    pub fn value_arg(&self, value: Value) -> Arg {
        match self.get_value_arg(value) {
            Some(arg) => arg,
            None => panic!("value {} not an argument", value),
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
    pub fn get_name(&self, value: Value) -> Option<&str> {
        self.names.get(&value).map(AsRef::as_ref)
    }

    /// Set the name of a value.
    pub fn set_name(&mut self, value: Value, name: String) {
        self.names.insert(value, name);
    }

    /// Clear the name of a value.
    pub fn clear_name(&mut self, value: Value) -> Option<String> {
        self.names.remove(&value)
    }

    /// Return the anonymous name hint of a value.
    pub fn get_anonymous_hint(&self, value: Value) -> Option<u32> {
        self.anonymous_hints.get(&value).cloned()
    }

    /// Set the anonymous name hint of a value.
    pub fn set_anonymous_hint(&mut self, value: Value, hint: u32) {
        self.anonymous_hints.insert(value, hint);
    }

    /// Clear the anonymous name hint of a value.
    pub fn clear_anonymous_hint(&mut self, value: Value) -> Option<u32> {
        self.anonymous_hints.remove(&value)
    }

    /// Replace all uses of a value with another.
    ///
    /// Returns how many uses were replaced.
    pub fn replace_use(&mut self, from: Value, to: Value) -> usize {
        let mut count = 0;
        for inst in self
            .value_uses
            .get(&from)
            .cloned()
            .unwrap_or_else(Default::default)
        {
            count += self.replace_value_within_inst(from, to, inst);
        }
        count
    }

    /// Replace the uses of a value with another, in a single instruction.
    ///
    /// Returns how many uses were replaced.
    pub fn replace_value_within_inst(&mut self, from: Value, to: Value, inst: Inst) -> usize {
        #[allow(deprecated)]
        let count = self[inst].replace_value(from, to);
        self.value_uses.entry(from).or_default().remove(&inst);
        self.update_uses(inst);
        count
    }

    /// Iterate over all uses of a value.
    pub fn uses(&self, value: Value) -> impl Iterator<Item = Inst> + '_ {
        self.value_uses[&value].iter().cloned()
    }

    /// Check if a value is used.
    pub fn has_uses(&self, value: Value) -> bool {
        self.uses(value).count() > 0
    }

    /// Check if a value has exactly one use.
    pub fn has_one_use(&self, value: Value) -> bool {
        self.uses(value).count() == 1
    }

    /// Replace all uses of a block with another.
    ///
    /// Returns how many blocks were replaced.
    pub fn replace_block_use(&mut self, from: Block, to: Block) -> usize {
        let mut count = 0;
        for inst in self
            .block_uses
            .get(&from)
            .cloned()
            .unwrap_or_else(Default::default)
        {
            count += self.replace_block_within_inst(from, to, inst);
        }
        count
    }

    /// Replace all uses of a block with another, in a single instruction.
    ///
    /// Returns how many blocks were replaced.
    pub fn replace_block_within_inst(&mut self, from: Block, to: Block, inst: Inst) -> usize {
        #[allow(deprecated)]
        let count = self[inst].replace_block(from, to);
        self.block_uses.entry(from).or_default().remove(&inst);
        self.update_uses(inst);
        count
    }

    /// Remove all uses of a block.
    ///
    /// Replaces all uses of the block with an invalid block placeholder, and
    /// removes phi node entries for the block.
    ///
    /// Returns how many blocks were removed.
    pub fn remove_block_use(&mut self, block: Block) -> usize {
        let mut count = 0;
        for inst in self
            .block_uses
            .get(&block)
            .cloned()
            .unwrap_or_else(Default::default)
        {
            count += self.remove_block_from_inst(block, inst);
        }
        count
    }

    /// Remove all uses of a block, from a single instruction.
    ///
    /// Replaces all uses of the block with an invalid block placeholder, and
    /// removes phi node entries for the block.
    ///
    /// Returns how many blocks were removed.
    pub fn remove_block_from_inst(&mut self, block: Block, inst: Inst) -> usize {
        #[allow(deprecated)]
        let count = self[inst].remove_block(block);
        self.block_uses.entry(block).or_default().remove(&inst);
        self.update_uses(inst);
        count
    }

    /// Resolve a constant value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const(&self, value: Value) -> Option<crate::Value> {
        use super::Opcode;
        let inst = self.get_value_inst(value)?;
        match self[inst].opcode() {
            Opcode::ConstInt => self.get_const_int(value).cloned().map(Into::into),
            Opcode::ConstTime => self.get_const_time(value).cloned().map(Into::into),
            Opcode::Array | Opcode::ArrayUniform => self.get_const_array(value).map(Into::into),
            Opcode::Struct => self.get_const_struct(value).map(Into::into),
            _ => None,
        }
    }

    /// Resolve a constant time value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_time(&self, value: Value) -> Option<&crate::TimeValue> {
        let inst = self.get_value_inst(value)?;
        self[inst].get_const_time()
    }

    /// Resolve a constant integer value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_int(&self, value: Value) -> Option<&crate::IntValue> {
        let inst = self.get_value_inst(value)?;
        self[inst].get_const_int()
    }

    /// Resolve a constant array value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_array(&self, value: Value) -> Option<crate::ArrayValue> {
        use super::Opcode;
        let inst = self.get_value_inst(value)?;
        match self[inst].opcode() {
            Opcode::Array => {
                let args: Option<Vec<_>> = self[inst]
                    .args()
                    .iter()
                    .map(|&a| self.get_const(a))
                    .collect();
                Some(crate::ArrayValue::new(args?))
            }
            Opcode::ArrayUniform => Some(crate::ArrayValue::new_uniform(
                self[inst].imms()[0],
                self.get_const(self[inst].args()[0])?,
            )),
            _ => None,
        }
    }

    /// Resolve a constant struct value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_struct(&self, value: Value) -> Option<crate::StructValue> {
        use super::Opcode;
        let inst = self.get_value_inst(value)?;
        match self[inst].opcode() {
            Opcode::Struct => {
                let args: Option<Vec<_>> = self[inst]
                    .args()
                    .iter()
                    .map(|&a| self.get_const(a))
                    .collect();
                Some(crate::StructValue::new(args?))
            }
            _ => None,
        }
    }

    /// Add a location hint to an instruction.
    ///
    /// Annotates the byte offset of an instruction in the input file.
    pub fn set_location_hint(&mut self, inst: Inst, loc: usize) {
        self.location_hints.insert(inst, loc);
    }

    /// Get the location hint associated with an instruction.
    ///
    /// Returns the byte offset of the instruction in the input file, or None if there
    /// is no hint for the instruction.
    pub fn location_hint(&self, inst: Inst) -> Option<usize> {
        self.location_hints.get(&inst).cloned()
    }
}
