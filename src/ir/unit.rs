// Copyright (c) 2017-2019 Fabian Schuiki

//! Common functionality of `Function`, `Process`, and `Entity`.

use crate::{
    ir::{
        Arg, Block, ControlFlowGraph, DataFlowGraph, ExtUnit, ExtUnitData, FunctionLayout, Inst,
        InstBuilder, InstData, InstLayout, Signature, Value,
    },
    ty::Type,
};

/// A name of a function, process, or entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnitKind {
    /// A `Function`.
    Function,
    /// A `Process`.
    Process,
    /// An `Entity`.
    Entity,
}

/// A `Function`, `Process`, or `Entity`.
pub trait Unit {
    /// Get the unit's DFG.
    #[inline]
    fn dfg(&self) -> &DataFlowGraph;

    /// Get the unit's mutable DFG.
    #[inline]
    fn dfg_mut(&mut self) -> &mut DataFlowGraph;

    /// Get the unit's CFG.
    #[inline]
    fn cfg(&self) -> &ControlFlowGraph;

    /// Get the unit's mutable CFG.
    #[inline]
    fn cfg_mut(&mut self) -> &mut ControlFlowGraph;

    /// Get the unit's signature.
    #[inline]
    fn sig(&self) -> &Signature;

    /// Get the unit's mutable signature.
    #[inline]
    fn sig_mut(&mut self) -> &mut Signature;

    /// Get the unit's name.
    #[inline]
    fn name(&self) -> &UnitName;

    /// Get the unit's mutable name.
    #[inline]
    fn name_mut(&mut self) -> &mut UnitName;

    /// Get the unit's function/process layout.
    ///
    /// Panics if the unit is an `Entity`.
    #[inline]
    fn func_layout(&self) -> &FunctionLayout;

    /// Get the unit's function/process layout.
    ///
    /// Panics if the unit is an `Entity`.
    #[inline]
    fn func_layout_mut(&mut self) -> &mut FunctionLayout;

    /// Get the unit's entity layout.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    #[inline]
    fn inst_layout(&self) -> &InstLayout;

    /// Get the unit's entity layout.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    #[inline]
    fn inst_layout_mut(&mut self) -> &mut InstLayout;

    /// Dump the unit in human-readable form.
    fn dump(&self) -> UnitDumper<&Self> {
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

    /// Get the value of argument `arg`.
    fn arg_value(&self, arg: Arg) -> Value {
        self.dfg().arg_value(arg)
    }

    /// Return an iterator over the unit's input arguments.
    fn input_args<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
        Box::new(self.sig().inputs().map(move |arg| self.arg_value(arg)))
    }

    /// Return an iterator over the unit's output arguments.
    fn output_args<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
        Box::new(self.sig().outputs().map(move |arg| self.arg_value(arg)))
    }

    /// Return an iterator over the unit's arguments.
    fn args<'a>(&'a self) -> Box<Iterator<Item = Value> + 'a> {
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

    /// Returns whether an instruction produces a result.
    fn has_result(&self, inst: Inst) -> bool {
        self.dfg().has_result(inst)
    }

    /// Returns the result of an instruction.
    fn inst_result(&self, inst: Inst) -> Value {
        self.dfg().inst_result(inst)
    }

    /// Returns the type of a value.
    fn value_type(&self, value: Value) -> Type {
        self.dfg().value_type(value)
    }

    /// Return the name of an external unit.
    fn extern_name(&self, ext: ExtUnit) -> &UnitName {
        &self.dfg()[ext].name
    }

    /// Return the signature of an external unit.
    fn extern_sig(&self, ext: ExtUnit) -> &Signature {
        &self.dfg()[ext].sig
    }
}

/// Temporary object to dump an `Entity` in human-readable form for debugging.
pub struct UnitDumper<U>(U);

impl<U: Unit> std::fmt::Display for UnitDumper<&U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.dump_fmt(f)
    }
}

/// A temporary object used to populate a `Function`, `Process` or `Entity`.
pub trait UnitBuilder {
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

    /// Get the entity layout of the unit being built.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    fn inst_layout(&self) -> &InstLayout {
        self.unit().inst_layout()
    }

    /// Get the entity layout of the unit being built.
    ///
    /// Panics if the unit is a `Function` or `Process`.
    fn inst_layout_mut(&mut self) -> &mut InstLayout {
        self.unit_mut().inst_layout_mut()
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
}
