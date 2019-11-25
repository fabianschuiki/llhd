// Copyright (c) 2017-2019 Fabian Schuiki

//! Graph-based representation of LLHD functions, processes, and entitites.
//!
//! This module implements a new graph-based IR. The design principles and
//! general ideas are as follows:
//!
//! - Simulatenous editing of nodes, enabled through garbage collection, and
//!   guards.
//! - Unified data, control, memory, and time flow graph.
//! - Fast and lean.
//! - Editing of the underlying data structures through builders/guards.
//! - Data structures use lightweight id references underneath (dense vec).
//! - User-facing handles carry id and pointer to data structure.
//! - Use `mut` not strictly for mutating, but to preserve consistency.
//!
//! Instruction construction shall occur via the following traits:
//! - `BuildInst` covers all purely data flow instructions
//! - `BuildInstImplicit` covers the rest, (memory, time, block derived from "position")
//! - `BuildInstExplicit` covers the rest, (memory, time, block explicitly provided by user)

// #![deny(missing_docs)]
#![allow(unused_variables, unused_imports, dead_code, unused_mut)]

use derive_new::new;
use hibitset::{BitSet, BitSetLike};
use llhd::impl_table_key;
use llhd::ir::{Opcode, UnitName};
use llhd::ty::Type;
use llhd::value::{IntValue, TimeValue};
use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Module {
    present_units: UnsafeCell<BitSet>,
    units: UnsafeCell<Vec<Option<Box<UnitData>>>>,
}

impl Module {
    /// Create a new module.
    pub fn new() -> Self {
        Self {
            present_units: Default::default(),
            units: Default::default(),
        }
    }

    /// Modify the module.
    pub fn modify(&mut self) -> ModuleBuilder {
        ModuleBuilder(UnsafeCell::new(self))
    }

    fn alloc_unit(&self, data: UnitData) -> UnitId {
        // Safe because we only add elements. This may cause the vector to grow,
        // which moves the boxes around. But since they're boxes, the referred
        // to data will not move.
        let mut units = unsafe { &mut *self.units.get() };
        for (id, slot) in units.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(Box::new(data));
                return UnitId(id as u32);
            }
        }
        let id = UnitId(units.len() as u32);
        units.push(Some(Box::new(data)));
        id
    }

    fn dealloc_unit(&mut self, unit: UnitId) -> UnitData {
        // Safe because the function takes &mut, ensuring that no references to
        // any of the units exist.
        unsafe {
            assert!(!(*self.present_units.get()).contains(unit.0));
            *(*self.units.get())[unit.0 as usize].take().unwrap()
        }
    }
}

pub struct ModuleBuilder<'m>(UnsafeCell<&'m mut Module>);

impl<'m> Deref for ModuleBuilder<'m> {
    type Target = &'m mut Module;

    fn deref(&self) -> &Self::Target {
        // Safe because the mutability is reflected in the reference taken on
        // self.
        unsafe { &*self.0.get() }
    }
}

impl<'m> DerefMut for ModuleBuilder<'m> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safe because the mutability is reflected in the reference taken on
        // self.
        unsafe { &mut *self.0.get() }
    }
}

impl<'m> ModuleBuilder<'m> {
    pub fn new_function<'u>(&'u self, name: UnitName) -> UnitBuilder<'m, 'u> {
        self.new_unit(UnitData::new(UnitKind::Function, name))
    }

    pub fn new_process<'u>(&'u self, name: UnitName) -> UnitBuilder<'m, 'u> {
        self.new_unit(UnitData::new(UnitKind::Process, name))
    }

    pub fn new_entity<'u>(&'u self, name: UnitName) -> UnitBuilder<'m, 'u> {
        self.new_unit(UnitData::new(UnitKind::Entity, name))
    }

    fn new_unit<'u>(&'u self, data: UnitData) -> UnitBuilder<'m, 'u> {
        // Safe because we hand out pointers on the heap.
        let id = self.alloc_unit(data);
        let data = unsafe {
            (*self.units.get())[id.0 as usize]
                .as_mut()
                .unwrap()
                .as_mut()
        };
        UnitBuilder(Unit(id, data as *const _, PhantomData), data, PhantomData)
    }

    pub fn add_unit(&self, unit: Unit<'m>) {}
    pub fn remove_unit(&self, unit: Unit<'m>) {}

    /// Modify a unit in the module.
    ///
    /// Only one module can be modified at the time through this function, since
    /// we cannot guarantee that the user requests modification of the same unit
    /// twice. If you want to modify units in parallel, use `modify_units`.
    pub fn modify_unit<'u>(&'u mut self, unit: Unit<'m>) -> UnitBuilder<'m, 'u> {
        // Safe because the function is &mut, enforcing a single reference.
        let data = unsafe {
            (*self.units.get())[(unit.0).0 as usize]
                .as_mut()
                .unwrap()
                .as_mut() as *mut _
        };
        UnitBuilder(unit, data, PhantomData)
    }

    /// Modify all units in the module in parallel.
    pub fn modify_units<'u>(&'u mut self) -> Vec<UnitBuilder<'m, 'u>> {
        vec![]
    }
}

#[derive(Clone, Copy)]
pub struct Unit<'m>(UnitId, *const UnitData, PhantomData<&'m ()>);
unsafe impl Send for Unit<'_> {}

impl<'m> Unit<'m> {
    /// Access the unit's data.
    pub fn data(self) -> &'m UnitData {
        // Safety is ensured since `self` is tied to the lifetime of its
        // enclosing module.
        unsafe { &*self.1 }
    }

    pub fn values(self) -> impl Iterator<Item = Value<'m>> {
        (&self.data().values_used)
            .iter()
            .map(move |id| Value(ValueId(id), self.1, PhantomData))
    }

    pub fn blocks(self) -> impl Iterator<Item = Block<'m>> {
        (&self.data().blocks_used)
            .iter()
            .map(move |id| Block(BlockId(id), self.1, PhantomData))
    }

    pub fn get_entry(self) -> Option<Block<'m>> {
        self.data()
            .entry
            .map(move |id| Block(id, self.1, PhantomData))
    }

    pub fn entry(self) -> Block<'m> {
        self.get_entry().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitId(u32);

pub struct UnitBuilder<'m, 'u>(Unit<'m>, *mut UnitData, PhantomData<&'u ()>);
unsafe impl Send for UnitBuilder<'_, '_> {}

impl<'m, 'u> Deref for UnitBuilder<'m, 'u> {
    type Target = Unit<'m>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'m, 'u> UnitBuilder<'m, 'u> {
    /// Get a handle on the unit being built. Takes a mutable reference to the
    /// unit builder to ensure the returned `Unit` cannot be used to modify the
    /// unit in parallel to the builder.
    pub fn unit<'a>(&self, _: &'a mut ModuleBuilder) -> Unit<'a>
    where
        'm: 'a,
    {
        self.0
    }

    pub fn finish(self) -> Unit<'m> {
        self.0
    }

    fn data(&self) -> &'u mut UnitData {
        // Safe because the only way to construct a UnitBuilder is through a
        // mutable reference to a UnitData, through the module.
        unsafe { &mut *self.1 }
    }

    pub fn build_block(&self, name: impl Into<String>) -> Block<'m> {
        let id = self.data().alloc_block(BlockData { name: name.into() });
        Block(id, self.1 as *const _, PhantomData)
    }

    pub fn build_const_int(&self, value: impl Into<IntValue>) -> Value<'m> {
        let value = value.into();
        let ty = value.ty();
        let data = InstData::ConstInt(value);
        self.add_inst(Opcode::ConstInt, ty, data)
    }

    pub fn build_const_time(&self, value: impl Into<TimeValue>) -> Value<'m> {
        let value = value.into();
        let data = InstData::ConstTime(value);
        self.add_inst(Opcode::ConstInt, llhd::time_ty(), data)
    }

    pub fn build_add(&self, lhs: Value, rhs: Value) -> Value<'m> {
        self.add_binary_inst(Opcode::Add, lhs.ty().clone(), lhs, rhs)
    }

    pub fn build_and(&self, lhs: Value, rhs: Value) -> Value<'m> {
        self.add_binary_inst(Opcode::And, lhs.ty().clone(), lhs, rhs)
    }

    pub fn build_neq(&self, lhs: Value, rhs: Value) -> Value<'m> {
        self.add_binary_inst(Opcode::Neq, llhd::int_ty(1), lhs, rhs)
    }

    pub fn build_sig(&self, init: Value) -> Value<'m> {
        self.add_unary_inst(Opcode::Sig, llhd::signal_ty(init.ty().clone()), init)
    }

    pub fn build_prb(&self, target: Value, after: impl Into<TimeNodeId>) -> Value<'m> {
        self.add_inst(
            Opcode::Prb,
            target.ty().unwrap_signal().clone(),
            InstData::Probe([target.into()], after.into()),
        )
    }

    pub fn build_drv(
        &self,
        target: Value,
        value: Value,
        delay: Value,
        in_bb: Block,
        after: impl Into<TimeNodeId>,
    ) -> Value<'m> {
        self.add_inst(
            Opcode::Drv,
            llhd::void_ty(),
            InstData::Drive(
                [target.into(), value.into(), delay.into()],
                in_bb.into(),
                after.into(),
            ),
        )
    }

    pub fn build_jump(&self, bb: Block, in_bb: Block) -> Value<'m> {
        self.add_inst(
            Opcode::Br,
            llhd::void_ty(),
            InstData::Jump([bb.into()], in_bb.into()),
        )
    }

    pub fn build_branch(&self, cond: Value, bb0: Block, bb1: Block, in_bb: Block) -> Value<'m> {
        self.add_inst(
            Opcode::BrCond,
            llhd::void_ty(),
            InstData::Branch([cond.into()], [bb0.into(), bb1.into()], in_bb.into()),
        )
    }

    pub fn build_wait(
        &self,
        bb: Block,
        sigs: Vec<ValueId>,
        in_bb: Block,
        after: impl Into<TimeNodeId>,
    ) -> Value<'m> {
        self.add_inst(
            Opcode::Wait,
            llhd::void_ty(),
            InstData::Wait(sigs, [bb.into()], in_bb.into(), after.into()),
        )
    }

    fn add_unary_inst(&self, opcode: Opcode, ty: Type, arg: impl Into<ValueId>) -> Value<'m> {
        self.add_inst(opcode, ty, InstData::Unary([arg.into()]))
    }

    fn add_binary_inst(
        &self,
        opcode: Opcode,
        ty: Type,
        arg0: impl Into<ValueId>,
        arg1: impl Into<ValueId>,
    ) -> Value<'m> {
        self.add_inst(opcode, ty, InstData::Binary([arg0.into(), arg1.into()]))
    }

    fn add_ternary_inst(
        &self,
        opcode: Opcode,
        ty: Type,
        arg0: impl Into<ValueId>,
        arg1: impl Into<ValueId>,
        arg2: impl Into<ValueId>,
    ) -> Value<'m> {
        self.add_inst(
            opcode,
            ty,
            InstData::Ternary([arg0.into(), arg1.into(), arg2.into()]),
        )
    }

    fn add_inst(&self, opcode: Opcode, ty: Type, inst: InstData) -> Value<'m> {
        let id = self.data().alloc_value(ValueData { opcode, ty, inst });
        Value(id, self.1 as *const _, PhantomData)
    }

    pub fn set_entry(&self, bb: Block) {
        self.data().entry = Some(bb.into());
    }

    pub fn remove_entry(&self) {
        self.data().entry = None;
    }
}

pub enum UnitKind {
    Function,
    Process,
    Entity,
}

pub struct UnitData {
    kind: UnitKind,
    name: UnitName,
    values: Vec<Box<ValueData>>,
    values_used: BitSet,
    values_free: BitSet,
    blocks: Vec<Box<BlockData>>,
    blocks_used: BitSet,
    blocks_free: BitSet,
    entry: Option<BlockId>,
}

impl UnitData {
    pub fn new(kind: UnitKind, name: UnitName) -> Self {
        Self {
            kind,
            name,
            values: Default::default(),
            values_used: Default::default(),
            values_free: Default::default(),
            blocks: Default::default(),
            blocks_used: Default::default(),
            blocks_free: Default::default(),
            entry: None,
        }
    }

    fn alloc_value(&mut self, data: ValueData) -> ValueId {
        if let Some(id) = (&self.values_free).iter().next() {
            self.values_used.add(id);
            self.values_free.remove(id);
            self.values[id as usize] = Box::new(data);
            return ValueId(id);
        }
        let id = self.values.len() as u32;
        self.values_used.add(id);
        let id = ValueId(id);
        self.values.push(Box::new(data));
        id
    }

    fn dealloc_value(&mut self, inst: ValueId) {
        self.values_used.remove(inst.0);
        self.values_free.add(inst.0);
        let x = &mut self.values[inst.0 as usize];
        x.ty = llhd::ty::void_ty();
        x.inst = InstData::Nullary;
    }

    fn alloc_block(&mut self, data: BlockData) -> BlockId {
        if let Some(id) = (&self.blocks_free).iter().next() {
            self.blocks_used.add(id);
            self.blocks_free.remove(id);
            self.blocks[id as usize] = Box::new(data);
            return BlockId(id);
        }
        let id = self.blocks.len() as u32;
        self.blocks_used.add(id);
        let id = BlockId(id);
        self.blocks.push(Box::new(data));
        id
    }

    fn dealloc_block(&mut self, inst: BlockId) {
        self.blocks_used.remove(inst.0);
        self.blocks_free.add(inst.0);
        let x = &mut self.blocks[inst.0 as usize];
        x.name.clear();
    }
}

#[derive(Clone, Copy)]
pub struct Value<'m>(ValueId, *const UnitData, PhantomData<&'m ()>);

impl std::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'m> Value<'m> {
    /// Access the value's unit.
    ///
    /// This *must* be private, since it's essentially a way to get a shared
    /// reference to a `UnitData` in parallel to a mutable reference.
    fn unit(self) -> &'m UnitData {
        // Safety is ensured since `self` is tied to the lifetime of its
        // enclosing unit.
        unsafe { &*self.1 }
    }

    /// Access the value's data.
    pub fn data(self) -> &'m ValueData {
        self.unit().values[(self.0).0 as usize].as_ref()
    }

    pub fn opcode(self) -> Opcode {
        self.data().opcode
    }

    pub fn ty(self) -> &'m Type {
        &self.data().ty
    }

    pub fn args(self) -> impl Iterator<Item = Value<'m>> {
        self.data()
            .args()
            .iter()
            .map(move |&id| Value(id, self.1, PhantomData))
    }

    pub fn blocks(self) -> impl Iterator<Item = Block<'m>> {
        self.data()
            .blocks()
            .iter()
            .map(move |&id| Block(id, self.1, PhantomData))
    }

    pub fn get_in_block(self) -> Option<Block<'m>> {
        self.data()
            .get_in_block()
            .map(|id| Block(id, self.1, PhantomData))
    }

    pub fn in_block(self) -> Block<'m> {
        self.get_in_block().unwrap()
    }

    pub fn get_after_time(self) -> Option<TimeNode<'m>> {
        self.data()
            .get_after_time()
            .map(|id| TimeNode(id, self.1, PhantomData))
    }

    pub fn after_time(self) -> TimeNode<'m> {
        self.get_after_time().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ValueId(u32);

impl std::fmt::Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

impl From<Value<'_>> for ValueId {
    fn from(x: Value) -> ValueId {
        x.0
    }
}

#[derive(Debug)]
pub struct ValueData {
    opcode: Opcode,
    ty: Type,
    inst: InstData,
}

impl Deref for ValueData {
    type Target = InstData;

    fn deref(&self) -> &InstData {
        &self.inst
    }
}

impl DerefMut for ValueData {
    fn deref_mut(&mut self) -> &mut InstData {
        &mut self.inst
    }
}

#[derive(Debug)]
pub enum InstData {
    Nullary,
    ConstInt(IntValue),
    ConstTime(TimeValue),
    Unary([ValueId; 1]),
    Binary([ValueId; 2]),
    Ternary([ValueId; 3]),
    Probe([ValueId; 1], TimeNodeId),
    Drive([ValueId; 3], BlockId, TimeNodeId),
    Jump([BlockId; 1], BlockId),
    Branch([ValueId; 1], [BlockId; 2], BlockId),
    Wait(Vec<ValueId>, [BlockId; 1], BlockId, TimeNodeId),
}

impl InstData {
    pub fn args(&self) -> &[ValueId] {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Jump(..) => &[],
            InstData::Unary(args) => args,
            InstData::Binary(args) => args,
            InstData::Ternary(args) => args,
            InstData::Probe(args, _) => args,
            InstData::Drive(args, _, _) => args,
            InstData::Branch(args, _, _) => args,
            InstData::Wait(args, _, _, _) => args,
        }
    }

    pub fn args_mut(&mut self) -> &mut [ValueId] {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Jump(..) => &mut [],
            InstData::Unary(args) => args,
            InstData::Binary(args) => args,
            InstData::Ternary(args) => args,
            InstData::Probe(args, _) => args,
            InstData::Drive(args, _, _) => args,
            InstData::Branch(args, _, _) => args,
            InstData::Wait(args, _, _, _) => args,
        }
    }

    pub fn blocks(&self) -> &[BlockId] {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Unary(..)
            | InstData::Binary(..)
            | InstData::Ternary(..)
            | InstData::Probe(..)
            | InstData::Drive(..) => &[],
            InstData::Jump(bbs, _) => bbs,
            InstData::Branch(_, bbs, _) => bbs,
            InstData::Wait(_, bbs, _, _) => bbs,
        }
    }

    pub fn blocks_mut(&mut self) -> &mut [BlockId] {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Unary(..)
            | InstData::Binary(..)
            | InstData::Ternary(..)
            | InstData::Probe(..)
            | InstData::Drive(..) => &mut [],
            InstData::Jump(bbs, _) => bbs,
            InstData::Branch(_, bbs, _) => bbs,
            InstData::Wait(_, bbs, _, _) => bbs,
        }
    }

    pub fn get_in_block(&self) -> Option<BlockId> {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Unary(..)
            | InstData::Binary(..)
            | InstData::Ternary(..)
            | InstData::Probe(..) => None,
            InstData::Drive(_, bb, _)
            | InstData::Jump(_, bb)
            | InstData::Branch(_, _, bb)
            | InstData::Wait(_, _, bb, _) => Some(*bb),
        }
    }

    pub fn in_block(&self) -> BlockId {
        self.get_in_block().unwrap()
    }

    pub fn get_after_time(&self) -> Option<TimeNodeId> {
        match self {
            InstData::Nullary
            | InstData::ConstInt(..)
            | InstData::ConstTime(..)
            | InstData::Unary(..)
            | InstData::Binary(..)
            | InstData::Ternary(..)
            | InstData::Jump(..)
            | InstData::Branch(..) => None,
            InstData::Probe(_, time)
            | InstData::Drive(_, _, time)
            | InstData::Wait(_, _, _, time) => Some(*time),
        }
    }

    pub fn after_time(&self) -> TimeNodeId {
        self.get_after_time().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct Block<'m>(BlockId, *const UnitData, PhantomData<&'m ()>);

impl std::fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'m> Block<'m> {
    /// Access the block's unit.
    ///
    /// This *must* be private, since it's essentially a way to get a shared
    /// reference to a `UnitData` in parallel to a mutable reference.
    fn unit(self) -> &'m UnitData {
        // Safety is ensured since `self` is tied to the lifetime of its
        // enclosing unit.
        unsafe { &*self.1 }
    }

    /// Access the block's data.
    pub fn data(self) -> &'m BlockData {
        self.unit().blocks[(self.0).0 as usize].as_ref()
    }

    pub fn name(self) -> &'m str {
        &self.data().name
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BlockId(u32);

impl From<Block<'_>> for BlockId {
    fn from(x: Block) -> BlockId {
        x.0
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

#[derive(Debug)]
pub struct BlockData {
    name: String,
}

#[derive(Clone, Copy)]
pub struct TimeNode<'m>(TimeNodeId, *const UnitData, PhantomData<&'m ()>);

impl std::fmt::Display for TimeNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'m> TimeNode<'m> {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TimeNodeId {
    Entry,
    Value(ValueId),
    Fence(u32),
}

impl From<TimeNode<'_>> for TimeNodeId {
    fn from(x: TimeNode) -> TimeNodeId {
        x.0
    }
}

impl From<Value<'_>> for TimeNodeId {
    fn from(x: Value) -> TimeNodeId {
        TimeNodeId::Value(x.into())
    }
}

impl From<ValueId> for TimeNodeId {
    fn from(x: ValueId) -> TimeNodeId {
        TimeNodeId::Value(x)
    }
}

impl std::fmt::Display for TimeNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeNodeId::Entry => write!(f, "Rtime"),
            TimeNodeId::Value(id) => write!(f, "{}", id),
            TimeNodeId::Fence(id) => write!(f, "tf{}", id),
        }
    }
}

pub fn plot_unit(unit: Unit) {
    println!("digraph {{");
    for block in unit.blocks() {
        println!("    {} [label=\"{}\", color=red]", block.0, block.name());
    }
    for value in unit.values() {
        println!(
            "    {} [label=\"{} {}\"]",
            value.0,
            value.opcode(),
            value.ty()
        );
        for (i, arg) in value.args().enumerate() {
            println!("    {} -> {} [label={}]", arg.0, value.0, i);
        }
        for (i, bb) in value.blocks().enumerate() {
            println!("    {} -> {} [label={}, color=red]", value.0, bb.0, i);
        }
        if let Some(in_bb) = value.get_in_block() {
            println!(
                "    {} -> {} [color=red, style=dotted, dir=none]",
                in_bb.0, value.0
            );
        }
        if let Some(after) = value.get_after_time() {
            println!("    {} -> {} [color=green, style=dotted]", after.0, value.0);
        }
    }
    println!("    Rbb [label=\"E\", fillcolor=red, style=filled, shape=circle]");
    if let Some(entry) = unit.get_entry() {
        println!("    Rbb -> {} [color=red]", entry.0);
    }
    println!("    Rtime [label=\"T\", fillcolor=green, style=filled, shape=circle]");
    println!("}}");
}

fn optimize(m: &mut Module) {
    use rayon::prelude::*;
    m.modify()
        .modify_units()
        .into_par_iter()
        .for_each(optimize_unit);
}
fn optimize_unit(u: UnitBuilder) {}

fn main() {
    let mut m = Module::new();
    let mut mb = m.modify();
    let mut eb = mb.new_entity(UnitName::global("foo"));
    let mut pb = mb.new_process(UnitName::global("bar"));

    let bb_init = pb.build_block("init");
    let bb_check = pb.build_block("check");
    let bb_event = pb.build_block("event");
    pb.set_entry(bb_init);

    let clk_zero = pb.build_const_int(IntValue::from_usize(1, 0));
    let v1 = pb.build_const_int(IntValue::from_usize(32, 0));
    let clk = pb.build_sig(clk_zero);
    let q = pb.build_sig(v1);

    let v0 = pb.build_const_int(IntValue::from_usize(32, 0));
    let v1 = pb.build_const_int(IntValue::from_usize(32, 19));
    let v0 = pb.build_add(v0, v1);
    let dval = pb.build_add(v0, v0);

    let clk0 = pb.build_prb(clk, TimeNodeId::Entry);
    let wait = pb.build_wait(bb_check, vec![clk.into()], bb_init, clk0);
    let clk1 = pb.build_prb(clk, wait);
    let v0 = pb.build_neq(clk0, clk1);
    let v1 = pb.build_neq(clk1, clk_zero);
    let ev = pb.build_and(v0, v1);

    pb.build_branch(ev, bb_init, bb_event, bb_check);
    pb.build_jump(bb_init, bb_event);

    pb.build_drv(
        q,
        dval,
        pb.build_const_time(TimeValue::new(num::zero(), 1, 0)),
        bb_event,
        clk1,
    );

    let p = pb.finish();
    let e = eb.finish();
    plot_unit(p);
}
