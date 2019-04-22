// Copyright (c) 2017-2019 Fabian Schuiki

//! Common functionality of `Function`, `Process`, and `Entity`.

use crate::{
    ir::{
        Arg, Block, DataFlowGraph, ExtUnit, ExtUnitData, Inst, InstData, Signature, Value,
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
pub trait Unit: Sized {
    /// Get the unit's DFG.
    #[inline]
    fn dfg(&self) -> &DataFlowGraph;

    /// Get the unit's mutable DFG.
    #[inline]
    fn dfg_mut(&mut self) -> &mut DataFlowGraph;

    /// Get the unit's signature.
    #[inline]
    fn sig(&self) -> &Signature;

    /// Get the unti's mutable signature.
    #[inline]
    fn sig_mut(&mut self) -> &mut Signature;

    /// Dump the unit in human-readable form.
    fn dump(&self) -> UnitDumper<Self> {
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
pub struct UnitDumper<'unit, U>(&'unit U);

impl<U: Unit> std::fmt::Display for UnitDumper<'_, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.dump_fmt(f)
    }
}

/// A temporary object used to populate a `Function`, `Process` or `Entity`.
pub trait UnitBuilder: Sized {
    /// The type returned by `unit()` and `unit_mut()`.
    type Unit: Unit;

    /// Return the unit being built.
    fn unit(&self) -> &Self::Unit;

    /// Return the mutable unit being built.
    fn unit_mut(&mut self) -> &mut Self::Unit;

    /// Add a new instruction.
    fn build_inst(&mut self, data: InstData, ty: Type) -> Inst;

    /// Remove an instruction.
    fn remove_inst(&mut self, inst: Inst);

    /// Create a new BB.
    ///
    /// Panics if the unit is an `Entity`.
    fn block(&mut self) -> Block;

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

    /// Import an external unit for use within this unit.
    fn add_extern(&mut self, data: ExtUnitData) -> ExtUnit {
        self.dfg_mut().ext_units.add(data)
    }
}
