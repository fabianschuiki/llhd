// Copyright (c) 2017-2021 Fabian Schuiki

//! Simulation structure creation
//!
//! The simulation builder creates the structure necessary for simulating a
//! design.

use crate::{
    state::{Instance, InstanceKind, InstanceState, Scope, Signal, SignalRef, State, ValueSlot},
    value::{ArrayValue, IntValue, StructValue, TimeValue, Value},
};
use anyhow::{anyhow, Result};
use num::bigint::ToBigInt;
use std::{collections::HashMap, sync::Mutex};

struct Builder<'ll> {
    module: &'ll llhd::ir::Module,
    signals: Vec<Signal>,
    probes: HashMap<SignalRef, Vec<String>>,
    insts: Vec<Instance<'ll>>,
    scope_stack: Vec<Scope>,
}

impl<'ll> Builder<'ll> {
    /// Create a new builder for the given module.
    fn new(module: &llhd::ir::Module) -> Builder {
        Builder {
            module: module,
            signals: Vec::new(),
            probes: HashMap::new(),
            insts: Vec::new(),
            scope_stack: Vec::new(),
        }
    }

    /// Build the root unit for a simulation.
    fn build_root(&mut self, unit: llhd::ir::Unit<'ll>) {
        let sig = unit.sig();

        // Allocate the input and output signals for the top-level module.
        // TODO(fschuiki): Assign proper default signal values.
        let inputs: Vec<_> = sig
            .inputs()
            .map(|arg| self.alloc_signal(sig.arg_type(arg), Value::Void))
            .collect();
        let outputs: Vec<_> = sig
            .outputs()
            .map(|arg| self.alloc_signal(sig.arg_type(arg), Value::Void))
            .collect();

        // Instantiate the top-level module.
        self.push_scope(unit.name().to_string());
        self.instantiate(unit, inputs, outputs);
    }

    /// Allocate a new signal in the simulation and return a reference to it.
    fn alloc_signal(&mut self, ty: llhd::Type, init: Value) -> SignalRef {
        let id = SignalRef::new(self.signals.len());
        self.signals.push(Signal::new(ty, init));
        id
    }

    /// Allocate a new signal probe in the simulation. This essentially assigns
    /// a name to a signal which is also known to the user.
    pub fn alloc_signal_probe(&mut self, signal: SignalRef, name: String) {
        self.probes
            .entry(signal)
            .or_insert(Vec::new())
            .push(name.clone());
        self.scope_stack.last_mut().unwrap().add_probe(signal, name);
    }

    /// Instantiate a process or entity for simulation. This recursively builds
    /// the simulation structure for all subunits as necessary.
    pub fn instantiate(
        &mut self,
        unit: llhd::ir::Unit<'ll>,
        inputs: Vec<SignalRef>,
        outputs: Vec<SignalRef>,
    ) {
        debug!("Instantiating {}", unit.name());

        // Create signal probes for the input and output arguments of the unit.
        let input_iter = unit.sig().inputs().zip(inputs.iter());
        let output_iter = unit.sig().outputs().zip(outputs.iter());
        let args_iter = input_iter.chain(output_iter);
        let mut values: HashMap<llhd::ir::Value, ValueSlot> = args_iter
            .map(|(arg, &sig)| {
                let v = unit.arg_value(arg);
                if let Some(name) = unit.get_name(v) {
                    self.alloc_signal_probe(sig, name.to_string());
                }
                (v, ValueSlot::Signal(sig))
            })
            .collect();

        // Make a list of signals that this instance is sensitive to.
        let mut signals = inputs;
        signals.extend(outputs);

        // Gather the process-/entity-specific information.
        let kind = if unit.is_process() {
            // Allocate signals.
            for block in unit.blocks() {
                for inst in unit.insts(block) {
                    if unit[inst].opcode() == llhd::ir::Opcode::Sig {
                        let value = unit.inst_result(inst);
                        let init = self.const_value(unit, unit[inst].args()[0]);
                        let sig = self.alloc_signal(unit.value_type(value), init);
                        signals.push(sig); // entity is re-evaluated when this signal changes
                        if let Some(name) = unit.get_name(value) {
                            self.alloc_signal_probe(sig, name.to_string());
                        }
                        values.insert(value, ValueSlot::Signal(sig));
                    }
                    // Hotfix for const insts in moore output not dominating their uses
                    else if unit[inst].opcode() == llhd::ir::Opcode::ConstInt
                        || unit[inst].opcode() == llhd::ir::Opcode::ConstTime
                    {
                        let value = unit.inst_result(inst);
                        values.insert(value, ValueSlot::Const(self.const_value(unit, value)));
                    }
                }
            }

            InstanceKind::Process {
                prok: unit,
                next_block: unit.first_block(),
            }
        } else if unit.is_entity() {
            // Allocate signals and instantiate subunits.
            for inst in unit.all_insts() {
                if unit[inst].opcode() == llhd::ir::Opcode::Sig {
                    let value = unit.inst_result(inst);
                    let init = self.const_value(unit, unit[inst].args()[0]);
                    let sig = self.alloc_signal(unit.value_type(value), init);
                    signals.push(sig); // entity is re-evaluated when this signal changes
                    if let Some(name) = unit.get_name(value) {
                        self.alloc_signal_probe(sig, name.to_string());
                    }
                    values.insert(value, ValueSlot::Signal(sig));
                } else if unit[inst].opcode() == llhd::ir::Opcode::Inst {
                    let ext_unit = unit[inst].get_ext_unit().unwrap();
                    let name = &unit[ext_unit].name;
                    let mod_subunit = match self.module.lookup_ext_unit(ext_unit, unit.id()) {
                        Some(llhd::ir::LinkedUnit::Def(s)) => s,
                        _ => panic!("external unit {} not linked", name),
                    };
                    self.push_scope(name.to_string());
                    let resolve_signal = |v| match values[v] {
                        ValueSlot::Signal(sig) => sig,
                        _ => panic!("value does not resolve to a signal"),
                    };
                    let inputs = unit[inst]
                        .input_args()
                        .iter()
                        .map(&resolve_signal)
                        .collect();
                    let outputs = unit[inst]
                        .output_args()
                        .iter()
                        .map(&resolve_signal)
                        .collect();
                    self.instantiate(self.module.unit(mod_subunit), inputs, outputs);
                    self.pop_scope();
                }
            }

            InstanceKind::Entity { entity: unit }
        } else {
            unreachable!()
        };

        // Create a mapping from signals to the values which correspond to them.
        // This resolves signals to arguments or `sig` instructions.
        let signal_values: HashMap<SignalRef, llhd::ir::Value> = values
            .iter()
            .flat_map(|(&v, s)| match s {
                &ValueSlot::Signal(s) => Some((s, v)),
                _ => None,
            })
            .collect();

        // Create the unit instance.
        self.insts.push(Instance {
            values,
            kind,
            state: InstanceState::Ready,
            signals,
            signal_values,
        })
    }

    /// Consume the builder and assemble the simulation state.
    pub fn finish(self) -> State<'ll> {
        State {
            module: self.module,
            signals: self.signals,
            probes: self.probes,
            scope: self.scope_stack.into_iter().next().unwrap(),
            insts: self.insts.into_iter().map(Mutex::new).collect(),
            time: TimeValue::new(num::zero(), 0, 0),
            events: Default::default(),
            timed: Default::default(),
        }
    }

    /// Push a new scope onto the stack.
    fn push_scope(&mut self, name: impl Into<String>) {
        self.scope_stack.push(Scope::new(name));
    }

    /// Pop a scope off the stack.
    fn pop_scope(&mut self) {
        let scope = self.scope_stack.pop().unwrap();
        self.scope_stack.last_mut().unwrap().add_subscope(scope);
    }

    /// Map an LLHD value to a constant value.
    ///
    /// This is useful for initializing the value of variables and signals.
    fn const_value(&self, unit: llhd::ir::Unit, value: llhd::ir::Value) -> Value {
        use llhd::ir::Opcode;
        let ty = unit.value_type(value);
        let inst = unit.value_inst(value);
        let data = &unit[inst];
        match data.opcode() {
            Opcode::ConstInt => IntValue::from_signed(
                ty.unwrap_int(),
                data.get_const_int()
                    .unwrap()
                    .value
                    .to_bigint()
                    .unwrap()
                    .clone(),
            )
            .into(),
            Opcode::ConstTime => data.get_const_time().unwrap().clone().into(),
            Opcode::ArrayUniform => {
                ArrayValue::new_uniform(data.imms()[0], self.const_value(unit, data.args()[0]))
                    .into()
            }
            Opcode::Array => ArrayValue::new(
                data.args()
                    .iter()
                    .map(|&arg| self.const_value(unit, arg))
                    .collect(),
            )
            .into(),
            Opcode::Struct => StructValue::new(
                data.args()
                    .iter()
                    .map(|&arg| self.const_value(unit, arg))
                    .collect(),
            )
            .into(),
            _ => panic!(
                "{} cannot be turned into a constant ({})",
                data.opcode(),
                inst.dump(&unit)
            ),
        }
    }
}

/// Build the simulation for a module.
pub fn build(module: &llhd::ir::Module) -> Result<State> {
    let mut builder = Builder::new(module);

    // Find the last process or entity in the module, which we will use as the
    // simulation's root unit.
    let root = match module
        .units()
        .filter(|&unit| unit.is_process() || unit.is_entity())
        .last()
    {
        Some(r) => r,
        None => Err(anyhow!("no process or entity found that can be simulated"))?,
    };
    info!("Found simulation root: {}", root.name());

    // Build the simulation for this root module.
    builder.build_root(root);

    // Build the simulation state.
    Ok(builder.finish())
}
