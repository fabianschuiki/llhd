// Copyright (c) 2017-2021 Fabian Schuiki

use crate::{
    analysis::{DominatorTree, PredecessorTable, TemporalRegionGraph},
    ir::{
        layout::BlockNode, prelude::*, BlockData, ControlFlowGraph, DataFlowGraph, ExtUnit,
        ExtUnitData, FunctionLayout, InstBuilder, InstData, UnitId, ValueData,
    },
    table::TableKey,
    verifier::Verifier,
    void_ty, Type,
};
use std::{
    collections::HashSet,
    ops::{Deref, Index, IndexMut},
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
    /// Create a new anonymous unit name.
    pub fn anonymous(id: u32) -> Self {
        UnitName::Anonymous(id)
    }

    /// Create a new local unit name.
    pub fn local(name: impl Into<String>) -> Self {
        UnitName::Local(name.into())
    }

    /// Create a new global unit name.
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

impl std::fmt::Display for UnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnitKind::Function => write!(f, "func"),
            UnitKind::Process => write!(f, "proc"),
            UnitKind::Entity => write!(f, "entity"),
        }
    }
}

/// A function, process, or entity.
#[allow(missing_docs)]
#[derive(Serialize, Deserialize)]
pub struct UnitData {
    pub kind: UnitKind,
    pub name: UnitName,
    pub(super) sig: Signature,
    pub(super) dfg: DataFlowGraph,
    pub(super) cfg: ControlFlowGraph,
    pub(super) layout: FunctionLayout,
}

impl UnitData {
    /// Create a new unit.
    pub fn new(kind: UnitKind, name: UnitName, sig: Signature) -> Self {
        match kind {
            UnitKind::Function => {
                assert!(!sig.has_outputs());
                assert!(sig.has_return_type());
            }
            UnitKind::Process | UnitKind::Entity => {
                assert!(!sig.has_return_type());
            }
        }
        let mut data = Self {
            kind,
            name,
            sig,
            dfg: Default::default(),
            cfg: Default::default(),
            layout: Default::default(),
        };
        let mut unit = UnitBuilder::new_anonymous(&mut data);
        if kind == UnitKind::Entity {
            unit.block();
            unit.insert_at_end();
            unit.ins().halt();
        }
        unit.make_args_for_signature(&unit.sig().clone());
        data
    }
}

/// An immutable function, process, or entity.
#[derive(Clone, Copy)]
pub struct Unit<'a> {
    unit: UnitId,
    data: &'a UnitData,
}

impl<'a> Unit<'a> {
    /// Create a new unit wrapper around raw unit data.
    pub fn new(unit: UnitId, data: &'a UnitData) -> Self {
        Self { unit, data }
    }

    /// Create a new unit wrapper around raw unit data that has not been added
    /// to a module yet.
    pub fn new_anonymous(data: &'a UnitData) -> Self {
        Self::new(UnitId::invalid(), data)
    }

    /// Get the unit's id.
    #[inline(always)]
    pub fn id(self) -> UnitId {
        self.unit
    }

    /// Get the unit's data.
    #[inline(always)]
    pub fn data(self) -> &'a UnitData {
        self.data
    }

    /// Get the kind of this unit.
    pub fn kind(&self) -> UnitKind {
        self.data.kind
    }

    /// Get the unit's signature.
    pub fn sig(self) -> &'a Signature {
        &self.data.sig
    }

    /// Get the unit's name.
    pub fn name(self) -> &'a UnitName {
        &self.data.name
    }

    /// Dump the unit in human-readable form.
    #[deprecated(since = "0.13.0", note = "simply drop the dump()")]
    pub fn dump(self) -> Self {
        self
    }

    /// Panic if the unit is not well-formed.
    pub fn verify(self) {
        let mut verifier = Verifier::new();
        verifier.verify_unit(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified {}:", self.data.kind);
                eprintln!("{}", self);
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }

    /// Check if this unit is a `Function`.
    pub fn is_function(self) -> bool {
        self.kind() == UnitKind::Function
    }

    /// Check if this unit is a `Process`.
    pub fn is_process(self) -> bool {
        self.kind() == UnitKind::Process
    }

    /// Check if this unit is an `Entity`.
    pub fn is_entity(self) -> bool {
        self.kind() == UnitKind::Entity
    }

    /// Return an iterator over the unit's input arguments.
    pub fn input_args(self) -> impl Iterator<Item = Value> + 'a {
        self.sig().inputs().map(move |arg| self.arg_value(arg))
    }

    /// Return an iterator over the unit's output arguments.
    pub fn output_args(self) -> impl Iterator<Item = Value> + 'a {
        self.sig().outputs().map(move |arg| self.arg_value(arg))
    }

    /// Return an iterator over the unit's arguments.
    pub fn args(self) -> impl Iterator<Item = Value> + 'a {
        self.sig().args().map(move |arg| self.arg_value(arg))
    }

    /// Get the input argument at position `pos`.
    pub fn input_arg(self, pos: usize) -> Value {
        self.arg_value(
            self.sig()
                .inputs()
                .nth(pos)
                .expect("input argument position out of bounds"),
        )
    }

    /// Get the output argument at position `pos`.
    pub fn output_arg(self, pos: usize) -> Value {
        self.arg_value(
            self.sig()
                .outputs()
                .nth(pos)
                .expect("output argument position out of bounds"),
        )
    }

    /// Return the name of an external unit.
    pub fn extern_name(self, ext: ExtUnit) -> &'a UnitName {
        &self.data.dfg[ext].name
    }

    /// Return the signature of an external unit.
    pub fn extern_sig(self, ext: ExtUnit) -> &'a Signature {
        &self.data.dfg[ext].sig
    }

    /// Return an iterator over the external units used by this unit.
    pub fn extern_units(self) -> impl Iterator<Item = (ExtUnit, &'a ExtUnitData)> + 'a {
        self.data.dfg.ext_units.iter()
    }
}

/// # Analyses
impl<'a> Unit<'a> {
    /// Compute the unit's temporal region graph.
    pub fn trg(self) -> TemporalRegionGraph {
        #[allow(deprecated)]
        TemporalRegionGraph::new(&self)
    }

    /// Compute the unit's block predecessor table.
    pub fn predtbl(self) -> PredecessorTable {
        #[allow(deprecated)]
        PredecessorTable::new(&self)
    }

    /// Compute the unit's temporal block predecessor table.
    pub fn temporal_predtbl(self) -> PredecessorTable {
        #[allow(deprecated)]
        PredecessorTable::new_temporal(&self)
    }

    /// Compute the unit's dominator tree.
    pub fn domtree(self) -> DominatorTree {
        self.domtree_with_predtbl(&self.predtbl())
    }

    /// Compute the unit's temporal dominator tree.
    pub fn temporal_domtree(self) -> DominatorTree {
        self.domtree_with_predtbl(&self.temporal_predtbl())
    }

    /// Compute the unit's dominator tree, if a predecessor table is already
    /// available.
    pub fn domtree_with_predtbl(self, pt: &PredecessorTable) -> DominatorTree {
        #[allow(deprecated)]
        DominatorTree::new(&self, pt)
    }
}

/// # Control Flow Graph
impl<'a> Unit<'a> {
    /// Return the name of a BB.
    pub fn get_block_name(self, bb: Block) -> Option<&'a str> {
        self.data.cfg[bb].name.as_ref().map(AsRef::as_ref)
    }

    /// Return the anonymous name hint of a BB.
    pub fn get_anonymous_block_hint(self, bb: Block) -> Option<u32> {
        self.data.cfg.anonymous_hints.get(&bb).cloned()
    }
}

/// # Data Flow Graph
impl<'a> Unit<'a> {
    /// Check if a value is a placeholder.
    pub fn is_placeholder(self, value: Value) -> bool {
        self[value].is_placeholder()
    }

    /// Returns whether an instruction produces a result.
    pub fn has_result(self, inst: Inst) -> bool {
        self.data.dfg.results.storage.contains_key(&inst.index())
    }

    /// Returns the result of an instruction.
    pub fn inst_result(self, inst: Inst) -> Value {
        self.data.dfg.results[inst]
    }

    /// Returns the result of an instruction.
    pub fn get_inst_result(self, inst: Inst) -> Option<Value> {
        self.data.dfg.results.get(inst).cloned()
    }

    /// Returns the value of an argument.
    pub fn arg_value(self, arg: Arg) -> Value {
        self.data.dfg.args[arg]
    }

    /// Returns the type of a value.
    pub fn value_type(self, value: Value) -> Type {
        match &self[value] {
            ValueData::Invalid => panic!("invalid value"),
            ValueData::Inst { ty, .. } => ty.clone(),
            ValueData::Arg { ty, .. } => ty.clone(),
            ValueData::Placeholder { ty, .. } => ty.clone(),
        }
    }

    /// Returns the type of an instruction.
    pub fn inst_type(self, inst: Inst) -> Type {
        if self.has_result(inst) {
            self.value_type(self.inst_result(inst))
        } else {
            void_ty()
        }
    }

    /// Return the argument that produces `value`.
    pub fn get_value_arg(self, value: Value) -> Option<Arg> {
        match self[value] {
            ValueData::Arg { arg, .. } => Some(arg),
            _ => None,
        }
    }

    /// Return the argument that produces `value`, or panic.
    pub fn value_arg(self, value: Value) -> Arg {
        match self.get_value_arg(value) {
            Some(arg) => arg,
            None => panic!("value {} not an argument", value),
        }
    }

    /// Return the instruction that produces `value`.
    pub fn get_value_inst(self, value: Value) -> Option<Inst> {
        match self[value] {
            ValueData::Inst { inst, .. } => Some(inst),
            _ => None,
        }
    }

    /// Return the instruction that produces `value`, or panic.
    pub fn value_inst(self, value: Value) -> Inst {
        match self.get_value_inst(value) {
            Some(inst) => inst,
            None => panic!("value {} not the result of an instruction", value),
        }
    }

    /// Return the name of a value.
    pub fn get_name(self, value: Value) -> Option<&'a str> {
        self.data.dfg.names.get(&value).map(AsRef::as_ref)
    }

    /// Return the anonymous name hint of a value.
    pub fn get_anonymous_hint(self, value: Value) -> Option<u32> {
        self.data.dfg.anonymous_hints.get(&value).cloned()
    }

    /// Iterate over all uses of a value.
    pub fn uses(self, value: Value) -> &'a HashSet<Inst> {
        &self.data.dfg.value_uses[&value]
    }

    /// Check if a value is used.
    pub fn has_uses(self, value: Value) -> bool {
        !self.uses(value).is_empty()
    }

    /// Check if a value has exactly one use.
    pub fn has_one_use(self, value: Value) -> bool {
        self.uses(value).len() == 1
    }

    /// Resolve a constant value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const(self, value: Value) -> Option<crate::Value> {
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
    pub fn get_const_time(self, value: Value) -> Option<&'a crate::TimeValue> {
        let inst = self.get_value_inst(value)?;
        self.data.dfg[inst].get_const_time()
    }

    /// Resolve a constant integer value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_int(self, value: Value) -> Option<&'a crate::IntValue> {
        let inst = self.get_value_inst(value)?;
        self.data.dfg[inst].get_const_int()
    }

    /// Resolve a constant array value.
    ///
    /// Returns `None` if the value is not constant. Note that this *does not*
    /// perform constant folding. Rather, the value must resolve to an
    /// instruction which produces a constant value.
    pub fn get_const_array(self, value: Value) -> Option<crate::ArrayValue> {
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
    pub fn get_const_struct(self, value: Value) -> Option<crate::StructValue> {
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

    /// Get the location hint associated with an instruction.
    ///
    /// Returns the byte offset of the instruction in the input file, or None if there
    /// is no hint for the instruction.
    pub fn location_hint(self, inst: Inst) -> Option<usize> {
        self.data.dfg.location_hints.get(&inst).cloned()
    }

    /// Get the block ID bound.
    ///
    /// This function is useful for creating dense vectors to associate data
    /// with blocks.
    pub fn block_id_bound(self) -> usize {
        self.data.cfg.blocks.capacity()
    }
}

/// # Basic Block Layout
///
/// The following functions are used to query the basic block layout.
impl<'a> Unit<'a> {
    /// Return an iterator over all BBs in layout order.
    pub fn blocks(self) -> impl Iterator<Item = Block> + 'a {
        let layout = &self.data.layout;
        std::iter::successors(layout.first_bb, move |&bb| self.next_block(bb))
    }

    /// Check if a block is inserted into the layout.
    pub fn is_block_inserted(self, bb: Block) -> bool {
        self.data.layout.bbs.contains(bb)
    }

    /// Get the first BB in the layout. This is the entry block.
    pub fn first_block(self) -> Option<Block> {
        let layout = &self.data.layout;
        layout.first_bb
    }

    /// Get the last BB in the layout.
    pub fn last_block(self) -> Option<Block> {
        let layout = &self.data.layout;
        layout.last_bb
    }

    /// Get the BB preceding `bb` in the layout.
    pub fn prev_block(self, bb: Block) -> Option<Block> {
        let layout = &self.data.layout;
        layout.bbs[bb].prev
    }

    /// Get the BB following `bb` in the layout.
    pub fn next_block(self, bb: Block) -> Option<Block> {
        let layout = &self.data.layout;
        layout.bbs[bb].next
    }

    /// Get the entry block in the layout.
    ///
    /// The fallible alternative is `first_block(bb)`.
    pub fn entry(self) -> Block {
        self.first_block().expect("entry block is required")
    }
}

/// # Instruction Layout
///
/// The following functions are used to query the instruction layout within a
/// block.
impl<'a> Unit<'a> {
    /// Get the BB which contains `inst`, or `None` if `inst` is not inserted.
    pub fn inst_block(self, inst: Inst) -> Option<Block> {
        self.data.layout.inst_map.get(&inst).cloned()
    }

    /// Return an iterator over all instructions in a block in layout order.
    pub fn insts(self, bb: Block) -> impl Iterator<Item = Inst> + 'a {
        self.data.layout.bbs[bb].layout.insts()
    }

    /// Return an iterator over all instructions in layout order.
    pub fn all_insts(self) -> impl Iterator<Item = Inst> + 'a {
        self.blocks().flat_map(move |bb| self.insts(bb))
    }

    /// Check if an instruction is inserted into the layout.
    pub fn is_inst_inserted(self, inst: Inst) -> bool {
        self.data.layout.inst_map.contains_key(&inst)
    }

    /// Get the first instruction in the layout.
    pub fn first_inst(self, bb: Block) -> Option<Inst> {
        self.data.layout.bbs[bb].layout.first_inst()
    }

    /// Get the last instruction in the layout.
    pub fn last_inst(self, bb: Block) -> Option<Inst> {
        self.data.layout.bbs[bb].layout.last_inst()
    }

    /// Get the instruction preceding `inst` in the layout.
    pub fn prev_inst(self, inst: Inst) -> Option<Inst> {
        let bb = self.inst_block(inst).unwrap();
        self.data.layout.bbs[bb].layout.prev_inst(inst)
    }

    /// Get the instruction following `inst` in the layout.
    pub fn next_inst(self, inst: Inst) -> Option<Inst> {
        let bb = self.inst_block(inst).unwrap();
        self.data.layout.bbs[bb].layout.next_inst(inst)
    }

    /// Get the terminator instruction in the layout.
    ///
    /// The fallible alternative is `last_inst(bb)`.
    pub fn terminator(self, bb: Block) -> Inst {
        match self.last_inst(bb) {
            Some(term) => term,
            None => panic!("block {} must have a terminator", bb.dump(&self)),
        }
    }
}

impl std::fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {{\n",
            self.data.kind,
            self.data.name,
            self.data.sig.dump(self)
        )?;
        for bb in self.blocks() {
            write!(f, "{}:\n", bb.dump(self))?;
            for inst in self.insts(bb) {
                if self[inst].opcode().is_terminator() && self.is_entity() {
                    continue;
                }
                write!(f, "    {}\n", inst.dump(self))?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// A mutable function, process, or entity.
pub struct UnitBuilder<'a> {
    /// The unit being modified.
    unit: Unit<'a>,
    /// The unit data being modified.
    data: &'a mut UnitData,
    /// The position where we are currently inserting instructions.
    pos: InsertPos,
}

// Ensure the UnitBuilder can be used like a Unit.
impl<'a> Deref for UnitBuilder<'a> {
    type Target = Unit<'a>;
    fn deref(&self) -> &Unit<'a> {
        &self.unit
    }
}

impl<'a> UnitBuilder<'a> {
    /// Create a new builder for a unit.
    pub fn new(unit: UnitId, data: &'a mut UnitData) -> Self {
        let pos = {
            let unit = Unit::new(unit, data);
            match data.kind {
                UnitKind::Entity => match unit.first_block() {
                    Some(bb) => InsertPos::Before(unit.terminator(bb)),
                    None => InsertPos::None,
                },
                _ => InsertPos::None,
            }
        };
        Self {
            unit: Unit::new(unit, unsafe { &*(data as *const _) }),
            // Safety of the above is enforced by UnitBuilder by requiring all
            // mutation of the unit to go through a mutable borrow of the
            // builder itself.
            data: data,
            pos,
        }
    }

    /// Create a new builder for a unit that has not yet been added to a module.
    pub fn new_anonymous(data: &'a mut UnitData) -> Self {
        Self::new(UnitId::invalid(), data)
    }

    /// Finish building and make the unit immutable again.
    pub fn finish(self) -> Unit<'a> {
        self.unit
    }

    /// Get the unit's mutable data.
    #[inline(always)]
    pub fn data(&mut self) -> &mut UnitData {
        self.data
    }

    /// Return the unit being built.
    pub fn unit(&'a self) -> Unit<'a> {
        self.unit
    }

    /// Add a new instruction using an `InstBuilder`.
    pub fn ins(&mut self) -> InstBuilder<'a, '_> {
        InstBuilder::new(self)
    }

    /// Add a new instruction.
    pub fn build_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.add_inst_dfg(data, ty);
        match self.pos {
            InsertPos::None => panic!("no block selected to insert instruction"),
            InsertPos::Append(bb) => self.append_inst(inst, bb),
            InsertPos::Prepend(bb) => {
                self.prepend_inst(inst, bb);
                self.pos = InsertPos::After(inst);
            }
            InsertPos::After(other) => {
                self.insert_inst_after(inst, other);
                self.pos = InsertPos::After(inst);
            }
            InsertPos::Before(other) => self.insert_inst_before(inst, other),
        }
        inst
    }

    /// Delete an instruction.
    ///
    /// Removes the instruction from the layout, data flwo graph, and control
    /// flow graph, and deletes it. The `Inst` is no longer valid afterwards.
    pub fn delete_inst(&mut self, inst: Inst) {
        self.remove_inst_dfg(inst);
        match self.pos {
            // If we inserted after i, now insert before i's successor, or if i
            // was the last inst in the block, at the end of the block.
            InsertPos::After(i) if i == inst => {
                self.pos = self
                    .next_inst(i)
                    .map(InsertPos::Before)
                    .unwrap_or(InsertPos::Append(self.inst_block(i).unwrap()))
            }
            // If we inserted before i, now insert after i's predecessor, or if
            // i was the first inst in the block, at the beginning of the block.
            InsertPos::Before(i) if i == inst => {
                self.pos = self
                    .prev_inst(i)
                    .map(InsertPos::After)
                    .unwrap_or(InsertPos::Prepend(self.inst_block(i).unwrap()))
            }
            // Everything else we just keep as is.
            _ => (),
        }
        self.remove_inst(inst);
    }

    /// Create a new BB.
    pub fn block(&mut self) -> Block {
        let bb = self.data.cfg.blocks.add(BlockData { name: None });
        self.append_block(bb);
        bb
    }

    /// Create a new named BB.
    pub fn named_block(&mut self, name: impl Into<String>) -> Block {
        let bb = self.block();
        self.set_block_name(bb, name.into());
        bb
    }

    /// Delete a block.
    ///
    /// Removes the block, and all its instructions, from the layout and control
    /// flow graph, deletes it. The `Block` is no longer valid afterwards.
    pub fn delete_block(&mut self, bb: Block) {
        let insts: Vec<_> = self.insts(bb).collect();
        self.remove_block_use(bb);
        self.remove_block(bb);
        self.data.cfg.blocks.remove(bb);
        for inst in insts {
            if self.has_result(inst) {
                let value = self.inst_result(inst);
                self.replace_use(value, Value::invalid());
            }
            self.remove_inst_dfg(inst);
            self.data.layout.unmap_inst(inst);
        }
    }

    /// Append all following instructions at the end of the unit.
    pub fn insert_at_end(&mut self) {
        self.pos = InsertPos::Append(self.entry());
    }

    /// Prepend all following instructions at the beginning of the unit.
    pub fn insert_at_beginning(&mut self) {
        self.pos = InsertPos::Prepend(self.entry());
    }

    /// Append all following instructions to the end of `bb`.
    pub fn append_to(&mut self, bb: Block) {
        self.pos = InsertPos::Append(bb);
    }

    /// Prepend all following instructions to the beginning of `bb`.
    pub fn prepend_to(&mut self, bb: Block) {
        self.pos = InsertPos::Prepend(bb);
    }

    /// Insert all following instructions after `inst`.
    pub fn insert_after(&mut self, inst: Inst) {
        self.pos = InsertPos::After(inst);
    }

    /// Insert all following instructions before `inst`.
    pub fn insert_before(&mut self, inst: Inst) {
        self.pos = InsertPos::Before(inst);
    }

    /// Import an external unit for use within this unit.
    pub fn add_extern(&mut self, name: UnitName, sig: Signature) -> ExtUnit {
        self.data.dfg.ext_units.add(ExtUnitData { sig, name })
    }

    /// Remove an instruction if its value is not being read.
    ///
    /// Returns true if the instruction was removed.
    pub fn prune_if_unused(&mut self, inst: Inst) -> bool {
        if self.has_result(inst) && !self.has_uses(self.inst_result(inst)) {
            #[allow(unreachable_patterns)]
            let inst_args: Vec<_> = self[inst]
                .args()
                .iter()
                .cloned()
                .flat_map(|arg| self.get_value_inst(arg))
                .collect();
            self.delete_inst(inst);
            for inst in inst_args {
                self.prune_if_unused(inst);
            }
            true
        } else {
            false
        }
    }
}

/// # Control Flow Graph
impl<'a> UnitBuilder<'a> {
    /// Set the name of a BB.
    pub fn set_block_name(&mut self, bb: Block, name: String) {
        self.data.cfg[bb].name = Some(name);
    }

    /// Clear the name of a BB.
    pub fn clear_block_name(&mut self, bb: Block) -> Option<String> {
        std::mem::replace(&mut self.data.cfg[bb].name, None)
    }

    /// Set the anonymous name hint of a BB.
    pub fn set_anonymous_block_hint(&mut self, bb: Block, hint: u32) {
        self.data.cfg.anonymous_hints.insert(bb, hint);
    }

    /// Clear the anonymous name hint of a BB.
    pub fn clear_anonymous_block_hint(&mut self, bb: Block) -> Option<u32> {
        self.data.cfg.anonymous_hints.remove(&bb)
    }
}

/// # Data Flow Graph
impl<'a> UnitBuilder<'a> {
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

    /// Add a value.
    fn add_value(&mut self, data: ValueData) -> Value {
        let v = self.data.dfg.values.add(data);
        self.data.dfg.value_uses.insert(v, Default::default());
        v
    }

    /// Remove a value.
    fn remove_value(&mut self, value: Value) -> ValueData {
        let data = self.data.dfg.values.remove(value);
        self.data.dfg.value_uses.remove(&value);
        data
    }

    /// Register a value use.
    fn update_uses(&mut self, inst: Inst) {
        for value in self[inst].args().to_vec() {
            self.data
                .dfg
                .value_uses
                .entry(value)
                .or_default()
                .insert(inst);
        }
        for block in self[inst].blocks().to_vec() {
            self.data
                .dfg
                .block_uses
                .entry(block)
                .or_default()
                .insert(inst);
        }
    }

    /// Remove a value use.
    fn remove_uses(&mut self, inst: Inst, data: InstData) {
        for value in data.args() {
            self.data
                .dfg
                .value_uses
                .get_mut(value)
                .unwrap()
                .remove(&inst);
        }
        for block in data.blocks() {
            self.data
                .dfg
                .block_uses
                .get_mut(block)
                .unwrap()
                .remove(&inst);
        }
    }

    /// Add an instruction.
    fn add_inst_dfg(&mut self, data: InstData, ty: Type) -> Inst {
        let has_result = data.opcode() == Opcode::Call || !ty.is_void();
        let inst = self.data.dfg.insts.add(data);
        if has_result {
            let result = self.add_value(ValueData::Inst { ty, inst });
            self.data.dfg.results.add(inst, result);
        }
        self.update_uses(inst);
        inst
    }

    /// Remove an instruction.
    fn remove_inst_dfg(&mut self, inst: Inst) {
        if self.has_result(inst) {
            let value = self.inst_result(inst);
            assert!(!self.has_uses(value));
            self.remove_value(value);
        }
        let data = self.data.dfg.insts.remove(inst);
        self.remove_uses(inst, data);
        self.data.dfg.results.remove(inst);
    }

    /// Create values for the arguments in a signature.
    pub(crate) fn make_args_for_signature(&mut self, sig: &Signature) {
        for arg in sig.args() {
            let value = self.add_value(ValueData::Arg {
                ty: sig.arg_type(arg),
                arg: arg,
            });
            self.data.dfg.args.add(arg, value);
        }
    }

    /// Set the name of a value.
    pub fn set_name(&mut self, value: Value, name: String) {
        self.data.dfg.names.insert(value, name);
    }

    /// Clear the name of a value.
    pub fn clear_name(&mut self, value: Value) -> Option<String> {
        self.data.dfg.names.remove(&value)
    }

    /// Set the anonymous name hint of a value.
    pub fn set_anonymous_hint(&mut self, value: Value, hint: u32) {
        self.data.dfg.anonymous_hints.insert(value, hint);
    }

    /// Clear the anonymous name hint of a value.
    pub fn clear_anonymous_hint(&mut self, value: Value) -> Option<u32> {
        self.data.dfg.anonymous_hints.remove(&value)
    }

    /// Replace all uses of a value with another.
    ///
    /// Returns how many uses were replaced.
    pub fn replace_use(&mut self, from: Value, to: Value) -> usize {
        let mut count = 0;
        for inst in self
            .data
            .dfg
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
        self.data
            .dfg
            .value_uses
            .entry(from)
            .or_default()
            .remove(&inst);
        self.update_uses(inst);
        count
    }

    /// Replace all uses of a block with another.
    ///
    /// Returns how many blocks were replaced.
    pub fn replace_block_use(&mut self, from: Block, to: Block) -> usize {
        let mut count = 0;
        for inst in self
            .data
            .dfg
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
        self.data
            .dfg
            .block_uses
            .entry(from)
            .or_default()
            .remove(&inst);
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
            .data
            .dfg
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
        self.data
            .dfg
            .block_uses
            .entry(block)
            .or_default()
            .remove(&inst);
        self.update_uses(inst);
        count
    }

    /// Add a location hint to an instruction.
    ///
    /// Annotates the byte offset of an instruction in the input file.
    pub fn set_location_hint(&mut self, inst: Inst, loc: usize) {
        self.data.dfg.location_hints.insert(inst, loc);
    }
}

/// # Basic Block Layout
///
/// The following functions are used to modify the basic block layout.
impl<'a> UnitBuilder<'a> {
    /// Append a BB to the end of the function.
    pub fn append_block(&mut self, bb: Block) {
        let layout = &mut self.data.layout;
        layout.bbs.add(
            bb,
            BlockNode {
                prev: layout.last_bb,
                next: None,
                layout: Default::default(),
            },
        );
        if let Some(prev) = layout.last_bb {
            layout.bbs[prev].next = Some(bb);
        }
        if layout.first_bb.is_none() {
            layout.first_bb = Some(bb);
        }
        layout.last_bb = Some(bb);
    }

    /// Prepend a BB to the beginning of a function.
    ///
    /// This effectively makes `bb` the new entry block.
    pub fn prepend_block(&mut self, bb: Block) {
        let layout = &mut self.data.layout;
        layout.bbs.add(
            bb,
            BlockNode {
                prev: None,
                next: layout.first_bb,
                layout: Default::default(),
            },
        );
        if let Some(next) = layout.first_bb {
            layout.bbs[next].prev = Some(bb);
        }
        if layout.last_bb.is_none() {
            layout.last_bb = Some(bb);
        }
        layout.first_bb = Some(bb);
    }

    /// Insert a BB after another BB.
    pub fn insert_block_after(&mut self, bb: Block, after: Block) {
        let layout = &mut self.data.layout;
        layout.bbs.add(
            bb,
            BlockNode {
                prev: Some(after),
                next: layout.bbs[after].next,
                layout: Default::default(),
            },
        );
        if let Some(next) = layout.bbs[after].next {
            layout.bbs[next].prev = Some(bb);
        }
        layout.bbs[after].next = Some(bb);
        if layout.last_bb == Some(after) {
            layout.last_bb = Some(bb);
        }
    }

    /// Insert a BB before another BB.
    pub fn insert_block_before(&mut self, bb: Block, before: Block) {
        let layout = &mut self.data.layout;
        layout.bbs.add(
            bb,
            BlockNode {
                prev: layout.bbs[before].prev,
                next: Some(before),
                layout: Default::default(),
            },
        );
        if let Some(prev) = layout.bbs[before].prev {
            layout.bbs[prev].next = Some(bb);
        }
        layout.bbs[before].prev = Some(bb);
        if layout.first_bb == Some(before) {
            layout.first_bb = Some(bb);
        }
    }

    /// Remove a BB from the function.
    pub fn remove_block(&mut self, bb: Block) {
        let layout = &mut self.data.layout;
        let node = layout.bbs.remove(bb).unwrap();
        if let Some(next) = node.next {
            layout.bbs[next].prev = node.prev;
        }
        if let Some(prev) = node.prev {
            layout.bbs[prev].next = node.next;
        }
        if layout.first_bb == Some(bb) {
            layout.first_bb = node.next;
        }
        if layout.last_bb == Some(bb) {
            layout.last_bb = node.prev;
        }
    }

    /// Swap the position of two BBs.
    pub fn swap_blocks(&mut self, bb0: Block, bb1: Block) {
        let layout = &mut self.data.layout;
        if bb0 == bb1 {
            return;
        }

        let mut bb0_next = layout.bbs[bb0].next;
        let mut bb0_prev = layout.bbs[bb0].prev;
        let mut bb1_next = layout.bbs[bb1].next;
        let mut bb1_prev = layout.bbs[bb1].prev;
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
        layout.bbs[bb0].next = bb1_next;
        layout.bbs[bb0].prev = bb1_prev;
        layout.bbs[bb1].next = bb0_next;
        layout.bbs[bb1].prev = bb0_prev;

        if let Some(next) = bb0_next {
            layout.bbs[next].prev = Some(bb1);
        }
        if let Some(prev) = bb0_prev {
            layout.bbs[prev].next = Some(bb1);
        }
        if let Some(next) = bb1_next {
            layout.bbs[next].prev = Some(bb0);
        }
        if let Some(prev) = bb1_prev {
            layout.bbs[prev].next = Some(bb0);
        }

        if layout.first_bb == Some(bb0) {
            layout.first_bb = Some(bb1);
        } else if layout.first_bb == Some(bb1) {
            layout.first_bb = Some(bb0);
        }
        if layout.last_bb == Some(bb0) {
            layout.last_bb = Some(bb1);
        } else if layout.last_bb == Some(bb1) {
            layout.last_bb = Some(bb0);
        }
    }
}

/// # Instruction Layout
///
/// The following functions are used to modify the instruction layout within a
/// block.
impl<'a> UnitBuilder<'a> {
    /// Append an instruction to the end of a BB.
    pub fn append_inst(&mut self, inst: Inst, bb: Block) {
        self.data.layout.bbs[bb].layout.append_inst(inst);
        self.data.layout.map_inst(inst, bb);
    }

    /// Prepend an instruction to the beginning of a BB.
    pub fn prepend_inst(&mut self, inst: Inst, bb: Block) {
        self.data.layout.bbs[bb].layout.prepend_inst(inst);
        self.data.layout.map_inst(inst, bb);
    }

    /// Insert an instruction after another instruction.
    pub fn insert_inst_after(&mut self, inst: Inst, after: Inst) {
        let bb = self.inst_block(after).expect("`after` not inserted");
        self.data.layout.bbs[bb]
            .layout
            .insert_inst_after(inst, after);
        self.data.layout.map_inst(inst, bb);
    }

    /// Insert an instruction before another instruction.
    pub fn insert_inst_before(&mut self, inst: Inst, before: Inst) {
        let bb = self.inst_block(before).expect("`before` not inserted");
        self.data.layout.bbs[bb]
            .layout
            .insert_inst_before(inst, before);
        self.data.layout.map_inst(inst, bb);
    }

    /// Remove an instruction from the function.
    pub fn remove_inst(&mut self, inst: Inst) {
        let bb = self.inst_block(inst).expect("`inst` not inserted");
        self.data.layout.bbs[bb].layout.remove_inst(inst);
        self.data.layout.unmap_inst(inst);
    }
}

// Allow builders to be borrowed as the unit being built.

impl<'a> std::borrow::Borrow<Unit<'a>> for UnitBuilder<'a> {
    fn borrow(&self) -> &Unit<'a> {
        &self.unit
    }
}

// Allow immutable indexing into `Unit`.

impl Index<Value> for Unit<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.data.dfg.index(idx)
    }
}

impl Index<Inst> for Unit<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.data.dfg.index(idx)
    }
}

impl Index<ExtUnit> for Unit<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.data.dfg.index(idx)
    }
}

impl Index<Block> for Unit<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.data.cfg.index(idx)
    }
}

// Allow immutable and mutable indexing into `UnitBuilder`.

impl Index<Value> for UnitBuilder<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.data.dfg.index(idx)
    }
}

impl Index<Inst> for UnitBuilder<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.data.dfg.index(idx)
    }
}

impl Index<ExtUnit> for UnitBuilder<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.data.dfg.index(idx)
    }
}

impl Index<Block> for UnitBuilder<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.data.cfg.index(idx)
    }
}

impl IndexMut<Value> for UnitBuilder<'_> {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.data.dfg.index_mut(idx)
    }
}

impl IndexMut<Inst> for UnitBuilder<'_> {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.data.dfg.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for UnitBuilder<'_> {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.data.dfg.index_mut(idx)
    }
}

impl IndexMut<Block> for UnitBuilder<'_> {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.data.cfg.index_mut(idx)
    }
}

/// The position where new instructions will be inserted.
#[derive(Clone, Copy)]
enum InsertPos {
    None,
    Append(Block),
    Prepend(Block),
    After(Inst),
    Before(Inst),
}

#[allow(dead_code)]
mod static_checks {
    use super::*;

    pub fn ensure_send<'a>(u: Unit<'a>, ub: UnitBuilder<'a>) -> impl Send + 'a {
        (u, ub)
    }
}
