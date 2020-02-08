// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of the data flow in a `Function`, `Process`, or `Entity`.
//!
//! Each unit in LLHD has an associated `DataFlowGraph` which contains all the
//! values, instructions, arguments, and links between them.

use crate::{
    impl_table_indexing,
    ir::{Arg, Block, ExtUnit, ExtUnitData, Inst, InstData, Signature, Value, ValueData},
    table::{PrimaryTable, SecondaryTable, TableKey},
    ty::{void_ty, Type},
};
use std::collections::HashMap;

/// A data flow graph.
///
/// This is the main container for instructions, values, and the relationship
/// between them. Every `Function`, `Process`, and `Entity` has an associated
/// data flow graph.
#[derive(Default, Serialize, Deserialize)]
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
    /// The anonymous name hints assigned to values.
    pub(crate) anonymous_hints: HashMap<Value, u32>,
    /// The location hints assigned to instructions.
    pub(crate) location_hints: HashMap<Inst, usize>,
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

    /// Check if a value is a placeholder.
    pub fn is_placeholder(&self, value: Value) -> bool {
        self[value].is_placeholder()
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
        if self.has_result(inst) {
            let value = self.inst_result(inst);
            assert!(!self.has_uses(value));
            self.values.remove(value);
        }
        self.insts.remove(inst);
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

    /// Returns the type of an instruction.
    pub fn inst_type(&self, inst: Inst) -> Type {
        if self.has_result(inst) {
            self.value_type(self.inst_result(inst))
        } else {
            void_ty()
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

    /// Replace all uses of a block with another.
    ///
    /// Returns how many blocks were replaced.
    pub fn replace_block_use(&mut self, from: Block, to: Block) -> usize {
        let mut count = 0;
        for inst in self.insts.storage.values_mut() {
            count += inst.replace_block(from, to);
        }
        count
    }

    /// Remove all uses of a block.
    ///
    /// Replaces all uses of the block with an invalid block placeholder, and
    /// removes phi node entries for the block.
    ///
    /// Returns how many blocks were replaced.
    pub fn remove_block_use(&mut self, block: Block) -> usize {
        let mut count = 0;
        for inst in self.insts.storage.values_mut() {
            count += inst.remove_block(block);
        }
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
