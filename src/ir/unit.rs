// Copyright (c) 2017-2020 Fabian Schuiki

//! Common functionality of `Function`, `Process`, and `Entity`.

use crate::{
    ir::{
        Arg, Block, BlockData, ControlFlowGraph, DataFlowGraph, Entity, ExtUnit, ExtUnitData,
        Function, FunctionLayout, Inst, InstBuilder, InstData, Process, Signature, Value,
        ValueData,
    },
    ty::Type,
};
use std::{
    collections::HashSet,
    ops::{Index, IndexMut},
};

/// A name of a function, process, or entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitName {
    /// An anonymous name, like `%42`.
    Anonymous(u32),
    /// A local name, like `%foo`.
    Local(String),
    /// A global name, like `@foo`.
    Global(String),
}

impl UnitName {
    // Create a new anonymous unit name.
    pub fn anonymous(id: u32) -> Self {
        UnitName::Anonymous(id)
    }

    // Create a new local unit name.
    pub fn local(name: impl Into<String>) -> Self {
        UnitName::Local(name.into())
    }

    // Create a new global unit name.
    pub fn global(name: impl Into<String>) -> Self {
        UnitName::Global(name.into())
    }

    /// Check whether this is a local name.
    ///
    /// Local names can only be linked within the same module.
    pub fn is_local(&self) -> bool {
        match self {
            UnitName::Anonymous(..) | UnitName::Local(..) => true,
            _ => false,
        }
    }

    /// Check whether this is a global name.
    ///
    /// Global names may be referenced by other modules and are considered by
    /// the global linker.
    pub fn is_global(&self) -> bool {
        match self {
            UnitName::Global(..) => true,
            _ => false,
        }
    }

    /// Get the underlying name.
    pub fn get_name(&self) -> Option<&str> {
        match self {
            UnitName::Global(n) | UnitName::Local(n) => Some(n.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for UnitName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnitName::Anonymous(id) => write!(f, "%{}", id),
            UnitName::Local(n) => write!(f, "%{}", n),
            UnitName::Global(n) => write!(f, "@{}", n),
        }
    }
}

/// The three different units that may appear in LLHD IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    /// A `Function`.
    Function,
    /// A `Process`.
    Process,
    /// An `Entity`.
    Entity,
}

/// A `Function`, `Process`, or `Entity`.
pub trait Unit:
    Index<Value, Output = ValueData>
    + Index<Inst, Output = InstData>
    + Index<ExtUnit, Output = ExtUnitData>
    + Index<Block, Output = BlockData>
    + IndexMut<Value>
    + IndexMut<Inst>
    + IndexMut<ExtUnit>
    + IndexMut<Block>
{
    /// Get the unit's DFG.
    fn dfg(&self) -> &DataFlowGraph;

    /// Get the unit's mutable DFG.
    fn dfg_mut(&mut self) -> &mut DataFlowGraph;

    /// Get the unit's CFG.
    fn try_cfg(&self) -> Option<&ControlFlowGraph>;

    /// Get the unit's mutable CFG.
    fn try_cfg_mut(&mut self) -> Option<&mut ControlFlowGraph>;

    /// Get the unit's CFG.
    #[inline]
    fn cfg(&self) -> &ControlFlowGraph {
        match self.try_cfg() {
            Some(cfg) => cfg,
            None => panic!("cfg() called on entity"),
        }
    }

    /// Get the unit's mutable CFG.
    #[inline]
    fn cfg_mut(&mut self) -> &mut ControlFlowGraph {
        match self.try_cfg_mut() {
            Some(cfg) => cfg,
            None => panic!("cfg_mut() called on entity"),
        }
    }

    /// Get the unit's signature.
    fn sig(&self) -> &Signature;

    /// Get the unit's mutable signature.
    fn sig_mut(&mut self) -> &mut Signature;

    /// Get the unit's name.
    fn name(&self) -> &UnitName;

    /// Get the unit's mutable name.
    fn name_mut(&mut self) -> &mut UnitName;

    /// Get the unit's function/process layout.
    ///
    /// Panics if the unit is an `Entity`.
    fn func_layout(&self) -> &FunctionLayout;

    /// Get the unit's function/process layout.
    ///
    /// Panics if the unit is an `Entity`.
    fn func_layout_mut(&mut self) -> &mut FunctionLayout;

    /// Dump the unit in human-readable form.
    fn dump(&self) -> UnitDumper
    where
        Self: Sized,
    {
        UnitDumper(self)
    }

    /// Actual implementation of `dump()`.
    fn dump_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    /// Panic if the unit is not well-formed.
    fn verify(&self);

    /// Return the kind of this unit.
    fn kind(&self) -> UnitKind;

    /// Check if this unit is a `Function`.
    fn is_function(&self) -> bool {
        self.kind() == UnitKind::Function
    }

    /// Check if this unit is a `Process`.
    fn is_process(&self) -> bool {
        self.kind() == UnitKind::Process
    }

    /// Check if this unit is an `Entity`.
    fn is_entity(&self) -> bool {
        self.kind() == UnitKind::Entity
    }

    /// Access this unit as a `Function`, if it is one.
    fn get_function(&self) -> Option<&Function> {
        None
    }

    /// Access this unit as a mutable `Function`, if it is one.
    fn get_function_mut(&mut self) -> Option<&mut Function> {
        None
    }

    /// Access this unit as a `Process`, if it is one.
    fn get_process(&self) -> Option<&Process> {
        None
    }

    /// Access this unit as a mutable `Process`, if it is one.
    fn get_process_mut(&mut self) -> Option<&mut Process> {
        None
    }

    /// Access this unit as an `Entity`, if it is one.
    fn get_entity(&self) -> Option<&Entity> {
        None
    }

    /// Access this unit as a mutablen `Entity`, if it is one.
    fn get_entity_mut(&mut self) -> Option<&mut Entity> {
        None
    }

    /// Return an iterator over the unit's input arguments.
    fn input_args<'a>(&'a self) -> Box<dyn Iterator<Item = Value> + 'a> {
        Box::new(self.sig().inputs().map(move |arg| self.arg_value(arg)))
    }

    /// Return an iterator over the unit's output arguments.
    fn output_args<'a>(&'a self) -> Box<dyn Iterator<Item = Value> + 'a> {
        Box::new(self.sig().outputs().map(move |arg| self.arg_value(arg)))
    }

    /// Return an iterator over the unit's arguments.
    fn args<'a>(&'a self) -> Box<dyn Iterator<Item = Value> + 'a> {
        Box::new(self.sig().args().map(move |arg| self.arg_value(arg)))
    }

    /// Get the input argument at position `pos`.
    fn input_arg(&self, pos: usize) -> Value {
        self.arg_value(
            self.sig()
                .inputs()
                .nth(pos)
                .expect("input argument position out of bounds"),
        )
    }

    /// Get the output argument at position `pos`.
    fn output_arg(&self, pos: usize) -> Value {
        self.arg_value(
            self.sig()
                .outputs()
                .nth(pos)
                .expect("output argument position out of bounds"),
        )
    }

    /// Return the name of an external unit.
    fn extern_name(&self, ext: ExtUnit) -> &UnitName {
        &self.dfg()[ext].name
    }

    /// Return the signature of an external unit.
    fn extern_sig(&self, ext: ExtUnit) -> &Signature {
        &self.dfg()[ext].sig
    }

    // ----- Control Flow Graph ------------------------------------------------

    /// Return the name of a BB.
    fn get_block_name(&self, bb: Block) -> Option<&str> {
        self.cfg().get_name(bb)
    }

    /// Return the anonymous name hint of a BB.
    fn get_anonymous_block_hint(&self, bb: Block) -> Option<u32> {
        self.cfg().get_anonymous_hint(bb)
    }

    // ----- Data Flow Graph ---------------------------------------------------

    /// Check if a value is a placeholder.
    fn is_placeholder(&self, value: Value) -> bool {
        self.dfg().is_placeholder(value)
    }

    /// Returns whether an instruction produces a result.
    fn has_result(&self, inst: Inst) -> bool {
        self.dfg().has_result(inst)
    }

    /// Returns the result of an instruction.
    fn inst_result(&self, inst: Inst) -> Value {
        self.dfg().inst_result(inst)
    }

    /// Returns the result of an instruction.
    fn get_inst_result(&self, inst: Inst) -> Option<Value> {
        self.dfg().get_inst_result(inst)
    }

    /// Returns the value of an argument.
    fn arg_value(&self, arg: Arg) -> Value {
        self.dfg().arg_value(arg)
    }

    /// Returns the type of a value.
    fn value_type(&self, value: Value) -> Type {
        self.dfg().value_type(value)
    }

    /// Returns the type of an instruction.
    fn inst_type(&self, inst: Inst) -> Type {
        self.dfg().inst_type(inst)
    }

    /// Return the argument that produces `value`.
    fn get_value_arg(&self, value: Value) -> Option<Arg> {
        self.dfg().get_value_arg(value)
    }

    /// Return the argument that produces `value`, or panic.
    fn value_arg(&self, value: Value) -> Arg {
        self.dfg().value_arg(value)
    }

    /// Return the instruction that produces `value`.
    fn get_value_inst(&self, value: Value) -> Option<Inst> {
        self.dfg().get_value_inst(value)
    }

    /// Return the instruction that produces `value`, or panic.
    fn value_inst(&self, value: Value) -> Inst {
        self.dfg().value_inst(value)
    }

    /// Return the name of a value.
    fn get_name(&self, value: Value) -> Option<&str> {
        self.dfg().get_name(value)
    }

    /// Return the anonymous name hint of a value.
    fn get_anonymous_hint(&self, value: Value) -> Option<u32> {
        self.dfg().get_anonymous_hint(value)
    }

    /// Iterate over all uses of a value.
    fn uses(&self, value: Value) -> &HashSet<Inst> {
        self.dfg().uses(value)
    }

    /// Check if a value is used.
    fn has_uses(&self, value: Value) -> bool {
        self.dfg().has_uses(value)
    }

    /// Check if a value has exactly one use.
    fn has_one_use(&self, value: Value) -> bool {
        self.dfg().has_one_use(value)
    }

    /// Resolve a constant value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    fn get_const(&self, value: Value) -> Option<crate::Value> {
        self.dfg().get_const(value)
    }

    /// Resolve a constant time value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    fn get_const_time(&self, value: Value) -> Option<&crate::TimeValue> {
        self.dfg().get_const_time(value)
    }

    /// Resolve a constant integer value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    fn get_const_int(&self, value: Value) -> Option<&crate::IntValue> {
        self.dfg().get_const_int(value)
    }

    /// Resolve a constant array value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    fn get_const_array(&self, value: Value) -> Option<crate::ArrayValue> {
        self.dfg().get_const_array(value)
    }

    /// Resolve a constant struct value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    fn get_const_struct(&self, value: Value) -> Option<crate::StructValue> {
        self.dfg().get_const_struct(value)
    }

    /// Get the location hint associated with an instruction.
    ///
    /// Returns the byte offset of the instruction in the input file, or None if there
    /// is no hint for the instruction.
    fn location_hint(&self, inst: Inst) -> Option<usize> {
        self.dfg().location_hint(inst)
    }
}

/// Temporary object to dump an `Entity` in human-readable form for debugging.
pub struct UnitDumper<'a>(&'a dyn Unit);

impl std::fmt::Display for UnitDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.dump_fmt(f)
    }
}

/// A temporary object used to populate a `Function`, `Process` or `Entity`.
pub trait UnitBuilder:
    Index<Value, Output = ValueData>
    + Index<Inst, Output = InstData>
    + Index<ExtUnit, Output = ExtUnitData>
    + Index<Block, Output = BlockData>
    + IndexMut<Value>
    + IndexMut<Inst>
    + IndexMut<ExtUnit>
    + IndexMut<Block>
{
    /// The type returned by `unit()` and `unit_mut()`.
    type Unit: Unit;

    /// Return the unit being built.
    fn unit(&self) -> &Self::Unit;

    /// Return the mutable unit being built.
    fn unit_mut(&mut self) -> &mut Self::Unit;

    /// Add a new instruction using an `InstBuilder`.
    fn ins(&mut self) -> InstBuilder<&mut Self> {
        InstBuilder::new(self)
    }

    /// Add a new instruction.
    fn build_inst(&mut self, data: InstData, ty: Type) -> Inst;

    /// Remove an instruction.
    fn remove_inst(&mut self, inst: Inst);

    /// Create a new BB.
    ///
    /// Panics if the unit is an `Entity`.
    fn block(&mut self) -> Block;

    /// Create a new named BB.
    ///
    /// Panics if the unit is an `Entity`. This is a convenience wrapper around
    /// `block()` followed by `unit_mut().cfg_mut().set_name(..)`.
    fn named_block(&mut self, name: impl Into<String>) -> Block {
        let bb = self.block();
        self.unit_mut().cfg_mut().set_name(bb, name.into());
        bb
    }

    /// Remove a BB.
    ///
    /// Panics if the unit is an `Entity`.
    fn remove_block(&mut self, bb: Block);

    /// Append all following instructions at the end of the unit.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    fn insert_at_end(&mut self);

    /// Prepend all following instructions at the beginning of the unit.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    fn insert_at_beginning(&mut self);

    /// Append all following instructions to the end of `bb`.
    ///
    /// Panics if the unit is an `Entity`.
    fn append_to(&mut self, bb: Block);

    /// Prepend all following instructions to the beginning of `bb`.
    ///
    /// Panics if the unit is an `Entity`.
    fn prepend_to(&mut self, bb: Block);

    /// Insert all following instructions after `inst`.
    fn insert_after(&mut self, inst: Inst);

    /// Insert all following instructions before `inst`.
    fn insert_before(&mut self, inst: Inst);

    /// Get the DFG of the unit being built.
    fn dfg(&self) -> &DataFlowGraph {
        self.unit().dfg()
    }

    /// Get the mutable DFG of the unit being built.
    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        self.unit_mut().dfg_mut()
    }

    /// Get the CFG of the unit being built.
    fn cfg(&self) -> &ControlFlowGraph {
        self.unit().cfg()
    }

    /// Get the mutable CFG of the unit being built.
    fn cfg_mut(&mut self) -> &mut ControlFlowGraph {
        self.unit_mut().cfg_mut()
    }

    /// Get the CFG of the unit being built.
    fn try_cfg(&self) -> Option<&ControlFlowGraph> {
        self.unit().try_cfg()
    }

    /// Get the mutable CFG of the unit being built.
    fn try_cfg_mut(&mut self) -> Option<&mut ControlFlowGraph> {
        self.unit_mut().try_cfg_mut()
    }

    /// Get the function/process layout of the unit being built.
    ///
    /// Panics if the unit is an `Entity`.
    fn func_layout(&self) -> &FunctionLayout {
        self.unit().func_layout()
    }

    /// Get the function/process layout of the unit being built.
    ///
    /// Panics if the unit is an `Entity`.
    fn func_layout_mut(&mut self) -> &mut FunctionLayout {
        self.unit_mut().func_layout_mut()
    }

    /// Import an external unit for use within this unit.
    fn add_extern(&mut self, name: UnitName, sig: Signature) -> ExtUnit {
        self.dfg_mut().ext_units.add(ExtUnitData { sig, name })
    }

    /// Remove an instruction if its value is not being read.
    ///
    /// Returns true if the instruction was removed.
    fn prune_if_unused(&mut self, inst: Inst) -> bool {
        if self.dfg().has_result(inst) && !self.dfg().has_uses(self.dfg().inst_result(inst)) {
            #[allow(unreachable_patterns)]
            let inst_args: Vec<_> = self.dfg()[inst]
                .args()
                .iter()
                .cloned()
                .flat_map(|arg| self.dfg().get_value_inst(arg))
                .collect();
            self.remove_inst(inst);
            for inst in inst_args {
                self.prune_if_unused(inst);
            }
            true
        } else {
            false
        }
    }

    // ----- Control Flow Graph ------------------------------------------------

    /// Set the name of a BB.
    fn set_block_name(&mut self, bb: Block, name: String) {
        self.cfg_mut().set_name(bb, name)
    }

    /// Clear the name of a BB.
    fn clear_block_name(&mut self, bb: Block) -> Option<String> {
        self.cfg_mut().clear_name(bb)
    }

    /// Set the anonymous name hint of a BB.
    fn set_anonymous_block_hint(&mut self, bb: Block, hint: u32) {
        self.cfg_mut().set_anonymous_hint(bb, hint)
    }

    /// Clear the anonymous name hint of a BB.
    fn clear_anonymous_block_hint(&mut self, bb: Block) -> Option<u32> {
        self.cfg_mut().clear_anonymous_hint(bb)
    }

    // ----- Data Flow Graph ---------------------------------------------------

    /// Add a placeholder value.
    ///
    /// This function is intended to be used when constructing PHI nodes.
    fn add_placeholder(&mut self, ty: Type) -> Value {
        self.dfg_mut().add_placeholder(ty)
    }

    /// Remove a placeholder value.
    fn remove_placeholder(&mut self, value: Value) {
        self.dfg_mut().remove_placeholder(value)
    }

    /// Add an instruction.
    fn add_inst(&mut self, data: InstData, ty: Type) -> Inst {
        self.dfg_mut().add_inst(data, ty)
    }

    /// Set the name of a value.
    fn set_name(&mut self, value: Value, name: String) {
        self.dfg_mut().set_name(value, name)
    }

    /// Clear the name of a value.
    fn clear_name(&mut self, value: Value) -> Option<String> {
        self.dfg_mut().clear_name(value)
    }

    /// Set the anonymous name hint of a value.
    fn set_anonymous_hint(&mut self, value: Value, hint: u32) {
        self.dfg_mut().set_anonymous_hint(value, hint)
    }

    /// Clear the anonymous name hint of a value.
    fn clear_anonymous_hint(&mut self, value: Value) -> Option<u32> {
        self.dfg_mut().clear_anonymous_hint(value)
    }

    /// Replace all uses of a value with another.
    ///
    /// Returns how many uses were replaced.
    fn replace_use(&mut self, from: Value, to: Value) -> usize {
        self.dfg_mut().replace_use(from, to)
    }

    /// Replace the uses of a value with another, in a single instruction.
    ///
    /// Returns how many uses were replaced.
    fn replace_value_within_inst(&mut self, from: Value, to: Value, inst: Inst) -> usize {
        self.dfg_mut().replace_value_within_inst(from, to, inst)
    }

    /// Replace all uses of a block with another.
    ///
    /// Returns how many blocks were replaced.
    fn replace_block_use(&mut self, from: Block, to: Block) -> usize {
        self.dfg_mut().replace_block_use(from, to)
    }

    /// Replace all uses of a block with another, in a single instruction.
    ///
    /// Returns how many blocks were replaced.
    fn replace_block_within_inst(&mut self, from: Block, to: Block, inst: Inst) -> usize {
        self.dfg_mut().replace_block_within_inst(from, to, inst)
    }

    /// Remove all uses of a block.
    ///
    /// Replaces all uses of the block with an invalid block placeholder, and
    /// removes phi node entries for the block.
    ///
    /// Returns how many blocks were removed.
    fn remove_block_use(&mut self, block: Block) -> usize {
        self.dfg_mut().remove_block_use(block)
    }

    /// Remove all uses of a block, from a single instruction.
    ///
    /// Replaces all uses of the block with an invalid block placeholder, and
    /// removes phi node entries for the block.
    ///
    /// Returns how many blocks were removed.
    fn remove_block_from_inst(&mut self, block: Block, inst: Inst) -> usize {
        self.dfg_mut().remove_block_from_inst(block, inst)
    }

    /// Add a location hint to an instruction.
    ///
    /// Annotates the byte offset of an instruction in the input file.
    fn set_location_hint(&mut self, inst: Inst, loc: usize) {
        self.dfg_mut().set_location_hint(inst, loc)
    }
}

// Check that `Unit` is object safe. Will abort with a compiler error otherwise.
#[allow(dead_code, unused_variables)]
fn is_object_safe() {
    let unit_ref: &dyn Unit;
}
