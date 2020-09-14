// Copyright (c) 2017-2020 Fabian Schuiki

//! Simulation execution engine
//!
//! The execution engine that advances the simulation step by step.

#![allow(dead_code, unused_variables, unused_imports)]

use crate::{
    state::{
        Event, Instance, InstanceKind, InstanceRef, InstanceState, Signal, SignalRef, State,
        TimedInstance, ValuePointer, ValueSelect, ValueSlice, ValueSlot, ValueTarget,
    },
    tracer::Tracer,
    value::{ArrayValue, IntValue, StructValue, TimeValue, Value},
};
use llhd::ir::{Opcode, Unit};
use num::{bigint::ToBigInt, BigInt, BigUint, One, ToPrimitive};
use rayon::prelude::*;
use std::{
    borrow::BorrowMut,
    collections::VecDeque,
    collections::{HashMap, HashSet},
};

pub struct Engine<'ts, 'tm: 'ts> {
    step: usize,
    state: &'ts mut State<'tm>,
    parallelize: bool,
    last_heartbeat: std::time::SystemTime,
}

impl<'ts, 'tm> Engine<'ts, 'tm> {
    /// Create a new engine to advance some simulation state.
    pub fn new(state: &'ts mut State<'tm>, parallelize: bool) -> Engine<'ts, 'tm> {
        Engine {
            step: 0,
            state,
            parallelize,
            last_heartbeat: std::time::UNIX_EPOCH,
        }
    }

    /// Run the simulation to completion.
    pub fn run(&mut self, tracer: &mut dyn Tracer, until_step: Option<usize>) {
        if let Some(until_step) = until_step {
            while self.step < until_step && self.step(tracer) {}
        } else {
            while self.step(tracer) {}
        }
        println!(
            "\rSimulating -- {} (#{})\x1b[0K",
            self.state.time, self.step
        );
    }

    /// Perform one simulation step. Returns true if there are remaining events
    /// in the queue, false otherwise. This can be used as an indication as to
    /// when the simulation is finished.
    pub fn step(&mut self, tracer: &mut dyn Tracer) -> bool {
        info!("STEP {}: {}", self.step, self.state.time);
        let now = std::time::SystemTime::now();
        if now
            .duration_since(self.last_heartbeat)
            .map(|x| x.as_millis())
            .unwrap_or(0)
            > 250
        {
            use std::io::Write;
            print!(
                "\rSimulating -- {} (#{})\x1b[0K",
                self.state.time, self.step
            );
            let _ = std::io::stdout().flush();
            self.last_heartbeat = now;
        }
        let first = self.step == 0;
        self.step += 1;

        // Apply events at this time, note changed signals.
        let mut changed_signals = HashSet::new();
        for (signal, value) in self.state.take_next_events() {
            // Determine the current state of all targeted signals.
            let signals = signal.0.iter().map(|s| s.target.unwrap_signal());
            let mut modified: Vec<_> = signals
                .clone()
                .map(|sig| self.state[sig].value().clone())
                .collect();
            for sig in signals.clone() {
                trace!("Event: {}", self.state.probes[&sig][0]);
            }

            // Modify the signals.
            let old = modified.clone();
            write_pointer(&signal, &mut modified, &value);

            // Store the modified state back.
            for ((sig, modified), old) in signals.zip(modified.into_iter()).zip(old.into_iter()) {
                if self.state[sig].set_value(modified.clone()) && modified != old {
                    changed_signals.insert(sig);
                    debug!(
                        "Change {} {} -> {}",
                        self.state.probes[&sig][0], old, modified
                    );
                }
            }
        }

        // Wake up units whose timed wait has run out.
        for inst in self.state.take_next_timed() {
            debug!("Wakeup {} (time)", self.state[inst].lock().unwrap().name(),);
            self.state[inst].lock().unwrap().state = InstanceState::Ready;
        }

        // Wake up units that were sensitive to one of the changed signals.
        for inst in &mut self.state.insts {
            let mut inst = inst.lock().unwrap();
            let trigger = if let InstanceState::Wait(_, ref signals) = inst.state {
                signals.iter().any(|s| changed_signals.contains(s))
            } else {
                false
            };
            if trigger {
                debug!("Wakeup {} (sense)", inst.name());
                inst.state = InstanceState::Ready;
            }
        }

        // Call output hook to write simulation trace to disk.
        tracer.step(self.state, &changed_signals);

        // Execute the instances that are ready.
        let ready_insts: Vec<_> = self
            .state
            .insts
            .iter()
            .enumerate()
            .filter(|&(_, u)| u.lock().unwrap().state == InstanceState::Ready)
            .map(|(i, _)| i)
            .collect();
        let events = if self.parallelize {
            ready_insts
                .par_iter()
                .map(|&index| {
                    let mut lk = self.state.insts[index].lock().unwrap();
                    self.step_instance(lk.borrow_mut(), &changed_signals, first)
                })
                .reduce(
                    || Vec::new(),
                    |mut a, b| {
                        a.extend(b);
                        a
                    },
                )
        } else {
            let mut events = Vec::new();
            for &index in &ready_insts {
                let mut lk = self.state.insts[index].lock().unwrap();
                events.extend(self.step_instance(lk.borrow_mut(), &changed_signals, first));
            }
            events
        };
        // for event in &events {
        //     let s = stringify_value(&event.value);
        //     for p in &self.state.probes()[&event.signal.target] {
        //         trace!("[{}] {} = {} @[{}]", self.state.time, p, s, event.time);
        //     }
        // }
        self.state.schedule_events(events.into_iter());

        // Gather a list of instances that perform a timed wait and schedule
        // them as to be woken up.
        let timed: Vec<_> = ready_insts
            .iter()
            .filter_map(
                |index| match self.state.insts[*index].lock().unwrap().state {
                    InstanceState::Wait(Some(ref time), _) => Some(TimedInstance {
                        time: time.clone(),
                        inst: InstanceRef::new(*index),
                    }),
                    _ => None,
                },
            )
            .collect();
        self.state.schedule_timed(timed.into_iter());

        // Advance time to next event or process wake, or finish
        match self.state.next_time() {
            Some(t) => {
                self.state.time = t;
                true
            }
            None => false,
        }
    }

    /// Continue execution of one single process or entity instance, until it is
    /// suspended by an instruction.
    fn step_instance(
        &self,
        instance: &mut Instance,
        changed_signals: &HashSet<SignalRef>,
        first: bool,
    ) -> Vec<Event> {
        match instance.kind {
            InstanceKind::Process { prok, next_block } => {
                self.step_process(instance, prok, next_block)
            }
            InstanceKind::Entity { entity } => {
                self.step_entity(instance, entity, changed_signals, first)
            }
        }
    }

    /// Continue execution of one single process until it is suspended by an
    /// instruction.
    fn step_process(
        &self,
        instance: &mut Instance,
        unit: llhd::ir::Unit,
        block: Option<llhd::ir::Block>,
    ) -> Vec<Event> {
        debug!("Step process {}", unit.name());
        let mut events = Vec::new();
        let mut next_block = block;
        while let Some(block) = next_block {
            next_block = None;
            for inst in unit.insts(block) {
                let action =
                    self.execute_instruction(inst, unit, &instance.values, &self.state.signals);
                match action {
                    Action::None => (),
                    Action::Value(vs) => {
                        let v = unit.inst_result(inst);
                        trace!("{} = {}", v, vs);
                        instance.set_value(v, vs)
                    }
                    Action::Store(ptr, value) => {
                        // Determine the current state of all targeted variables.
                        let vars = ptr.0.iter().map(|s| s.target.unwrap_variable());
                        let mut modified: Vec<_> = vars
                            .clone()
                            .map(|var| match instance.value(var) {
                                ValueSlot::Variable(ref k) => k.clone(),
                                x => panic!(
                                    "variable targeted by store action has value {:?} instead of \
                                     Variable(...)",
                                    x
                                ),
                            })
                            .collect();

                        // Modify the variables.
                        write_pointer(&ptr, &mut modified, &value);

                        // Store the modified state back.
                        for (var, modified) in vars.zip(modified.into_iter()) {
                            instance.set_value(var, ValueSlot::Variable(modified));
                        }
                    }
                    Action::Event(e) => {
                        // debug!("Enqueue {:?}", e);
                        events.push(e)
                    }
                    Action::Jump(blk) => {
                        next_block = Some(blk);
                        break;
                    }
                    Action::Suspend(blk, st) => {
                        instance.state = st;
                        match instance.kind {
                            InstanceKind::Process {
                                ref mut next_block, ..
                            } => *next_block = blk,
                            _ => unreachable!(),
                        }
                        return events;
                    }
                }
            }
        }

        // We should never arrive here, since every block ends with a
        // terminator instruction that redirects control flow.
        panic!("process starved of instructions");
    }

    /// Continue execution of one single entity.
    fn step_entity(
        &self,
        instance: &mut Instance,
        unit: llhd::ir::Unit,
        changed_signals: &HashSet<SignalRef>,
        first: bool,
    ) -> Vec<Event> {
        debug!("Step entity {}", unit.name());
        let mut events = Vec::new();

        // First collect the probe instructions that react to the changed
        // signals. This will be the root of the data flow update.
        trace!("Collecting triggered instructions");
        let mut dirty = VecDeque::new();
        let mut dirty_set = HashSet::new();
        if first {
            for inst in unit.all_insts() {
                dirty.push_back(inst);
                dirty_set.insert(inst);
            }
        } else {
            for sig in instance
                .signals
                .iter()
                .filter(|sig| changed_signals.contains(sig))
            {
                let value = instance.signal_values[&sig];
                trace!("  Triggering {} ({})", self.state.probes[&sig][0], value);
                for &inst in unit.uses(value) {
                    match unit[inst].opcode() {
                        Opcode::Drv | Opcode::Inst | Opcode::Sig => continue,
                        _ => (),
                    }
                    trace!("    -> {}", inst.dump(&unit));
                    if !dirty_set.contains(&inst) {
                        dirty.push_back(inst);
                        dirty_set.insert(inst);
                    }
                }
            }
        }

        // Work the set of dirty instructions. Grab an instruction from the set
        // and execute it. If the action causes a value change, add dependent
        // instructions to the set.
        while let Some(inst) = dirty.pop_front() {
            dirty_set.remove(&inst);
            let action =
                self.execute_instruction(inst, unit, &instance.values, &self.state.signals);
            match action {
                Action::None => (),
                Action::Value(new) => {
                    let v = unit.inst_result(inst);
                    trace!("%{} = {}", v, new);
                    let changed = instance
                        .values
                        .get(&v)
                        .map(|old| match old {
                            // Check actual values for a change.
                            ValueSlot::Const(_) => old != &new,
                            // Everything else is always considered changed,
                            // especially pointers.
                            _ => true,
                        })
                        .unwrap_or(true);
                    instance.set_value(v, new);
                    if changed {
                        for &inst in unit.uses(v) {
                            if !dirty_set.contains(&inst) {
                                // trace!("  -> triggering {}", inst.dump(unit));
                                dirty.push_back(inst);
                                dirty_set.insert(inst);
                            }
                        }
                    }
                }
                Action::Store(ptr, new) => panic!("cannot store in entity"),
                Action::Event(e) => events.push(e),
                Action::Jump(..) => panic!("cannot jump in entity"),
                Action::Suspend(..) => panic!("cannot suspend entity"),
            }
        }

        // Suspend entity execution until any of the input and output signals
        // change.
        instance.state = InstanceState::Wait(None, instance.signals.clone());

        events
    }

    /// Execute a single instruction. Returns an action to be taken in response
    /// to the instruction.
    fn execute_instruction(
        &self,
        inst: llhd::ir::Inst,
        unit: llhd::ir::Unit,
        values: &HashMap<llhd::ir::Value, ValueSlot>,
        signals: &[Signal],
    ) -> Action {
        InstContext {
            unit,
            values,
            signals,
            time: &self.state.time,
        }
        .exec(inst)
    }
}

struct InstContext<'a> {
    unit: llhd::ir::Unit<'a>,
    values: &'a HashMap<llhd::ir::Value, ValueSlot>,
    signals: &'a [Signal],
    time: &'a TimeValue,
}

impl<'a> InstContext<'a> {
    /// Execute a single instruction. Returns an action to be taken in response
    /// to the instruction.
    fn exec(&self, inst: llhd::ir::Inst) -> Action {
        use llhd::ir::Opcode;
        let data = &self.unit[inst];
        let ty = self.unit.inst_type(inst);
        match data.opcode() {
            Opcode::Inst | Opcode::Sig => (),
            _ => trace!("{}", inst.dump(&self.unit)),
        }

        match data.opcode() {
            // Constants
            Opcode::ConstInt => {
                let v = IntValue::from_signed(
                    ty.unwrap_int(),
                    data.get_const_int()
                        .unwrap()
                        .value
                        .to_bigint()
                        .unwrap()
                        .clone(),
                );
                Action::Value(ValueSlot::Const(v.into()))
            }
            Opcode::ConstTime => {
                let v = data.get_const_time().unwrap().clone();
                Action::Value(ValueSlot::Const(v.into()))
            }

            // Aggregates
            Opcode::ArrayUniform => {
                let v = ArrayValue::new_uniform(data.imms()[0], self.resolve_value(data.args()[0]));
                Action::Value(ValueSlot::Const(v.into()))
            }
            Opcode::Array => {
                let vs = data
                    .args()
                    .iter()
                    .map(|&arg| self.resolve_value(arg))
                    .collect();
                let v = ArrayValue::new(vs);
                Action::Value(ValueSlot::Const(v.into()))
            }
            Opcode::Struct => {
                let vs = data
                    .args()
                    .iter()
                    .map(|&arg| self.resolve_value(arg))
                    .collect();
                let v = StructValue::new(vs);
                Action::Value(ValueSlot::Const(v.into()))
            }

            // Alias
            Opcode::Alias => Action::Value(ValueSlot::Const(self.resolve_value(data.args()[0]))),

            // Branches
            Opcode::Br => Action::Jump(data.blocks()[0]),
            Opcode::BrCond => {
                let cond = self.resolve_value(data.args()[0]);
                if cond.is_zero() {
                    Action::Jump(data.blocks()[0])
                } else {
                    Action::Jump(data.blocks()[1])
                }
            }

            // Waits
            Opcode::Wait => {
                let sigs = data
                    .args()
                    .iter()
                    .map(|&s| self.resolve_signal(s))
                    .collect();
                Action::Suspend(Some(data.blocks()[0]), InstanceState::Wait(None, sigs))
            }
            Opcode::WaitTime => {
                let delay = self.time_after_delay(&self.resolve_delay(data.args()[0]));
                let sigs = data.args()[1..]
                    .iter()
                    .map(|&s| self.resolve_signal(s))
                    .collect();
                Action::Suspend(
                    Some(data.blocks()[0]),
                    InstanceState::Wait(Some(delay), sigs),
                )
            }

            // Memory
            Opcode::Var => Action::Value(ValueSlot::Variable(self.resolve_value(data.args()[0]))),
            Opcode::Ld => {
                let ptr = self.resolve_variable_pointer(data.args()[0]);
                Action::Value(ValueSlot::Const(self.read_pointer(&ty, &ptr)))
            }
            Opcode::St => {
                let ptr = self.resolve_variable_pointer(data.args()[0]);
                let value = self.resolve_value(data.args()[1]);
                Action::Store(ptr, value)
            }

            // Signals are simply ignored, as they are handled by the builder.
            Opcode::Sig => Action::None,
            Opcode::Prb => {
                let sig = self.resolve_signal_pointer(data.args()[0]);
                Action::Value(ValueSlot::Const(self.read_pointer(&ty, &sig)))
            }
            Opcode::Drv => {
                let delay = self.resolve_delay(data.args()[2]);
                let ev = Event {
                    time: self.time_after_delay(&delay),
                    signal: self.resolve_signal_pointer(data.args()[0]),
                    value: self.resolve_value(data.args()[1]),
                };
                Action::Event(ev)
            }

            // Unary operators
            Opcode::Not | Opcode::Neg => {
                if ty.is_int() {
                    let arg = self.resolve_value(data.args()[0]);
                    let arg = arg.unwrap_int();
                    let v = IntValue::unary_op(data.opcode(), arg);
                    Action::Value(ValueSlot::Const(v.into()))
                } else {
                    panic!("{} on {} not supported", data.opcode(), ty);
                }
            }

            // Binary operators
            Opcode::Add
            | Opcode::Sub
            | Opcode::And
            | Opcode::Or
            | Opcode::Xor
            | Opcode::Smul
            | Opcode::Sdiv
            | Opcode::Smod
            | Opcode::Srem
            | Opcode::Umul
            | Opcode::Udiv
            | Opcode::Umod
            | Opcode::Urem => {
                if ty.is_int() {
                    let lhs = self.resolve_value(data.args()[0]);
                    let rhs = self.resolve_value(data.args()[1]);
                    let lhs = lhs.unwrap_int();
                    let rhs = rhs.unwrap_int();
                    let v = IntValue::binary_op(data.opcode(), lhs, rhs);
                    Action::Value(ValueSlot::Const(v.into()))
                } else {
                    panic!("{} on {} not supported", data.opcode(), ty);
                }
            }

            // Comparisons
            Opcode::Eq
            | Opcode::Neq
            | Opcode::Slt
            | Opcode::Sgt
            | Opcode::Sle
            | Opcode::Sge
            | Opcode::Ult
            | Opcode::Ugt
            | Opcode::Ule
            | Opcode::Uge => {
                if ty.is_int() {
                    let lhs = self.resolve_value(data.args()[0]);
                    let rhs = self.resolve_value(data.args()[1]);
                    let lhs = lhs.unwrap_int();
                    let rhs = rhs.unwrap_int();
                    let v = IntValue::compare_op(data.opcode(), lhs, rhs);
                    Action::Value(ValueSlot::Const(v.into()))
                } else {
                    panic!("{} on {} not supported", data.opcode(), ty);
                }
            }

            // Shifts
            Opcode::Shl | Opcode::Shr => {
                let (base, hidden) = if ty.is_pointer() {
                    (
                        self.resolve_variable_pointer(data.args()[0]),
                        self.resolve_variable_pointer(data.args()[1]),
                    )
                } else if ty.is_signal() {
                    (
                        self.resolve_signal_pointer(data.args()[0]),
                        self.resolve_signal_pointer(data.args()[1]),
                    )
                } else {
                    (
                        self.resolve_value_pointer(data.args()[0]),
                        self.resolve_value_pointer(data.args()[1]),
                    )
                };
                let amount = self.resolve_value(data.args()[2]);
                let ptr = self.exec_shift(data.opcode(), &base, &hidden, &amount);
                if ty.is_pointer() {
                    Action::Value(ValueSlot::VariablePointer(ptr))
                } else if ty.is_signal() {
                    Action::Value(ValueSlot::SignalPointer(ptr))
                } else {
                    Action::Value(ValueSlot::Const(self.read_pointer(&ty, &ptr)))
                }
            }

            // Insert and extract fields and slices
            Opcode::InsField | Opcode::InsSlice => {
                let target = self.resolve_value_pointer(data.args()[0]);
                let target_ty = self.unit.value_type(data.args()[0]);
                let value = self.resolve_value(data.args()[1]);
                let ptr = self.exec_insext(data.opcode(), &target_ty, &target, data.imms());
                let mut results = [self.resolve_value(data.args()[0])];
                write_pointer(&ptr, &mut results, &value);
                let [result] = results;
                Action::Value(ValueSlot::Const(result))
            }
            Opcode::ExtField | Opcode::ExtSlice => {
                let target = if ty.is_pointer() {
                    self.resolve_variable_pointer(data.args()[0])
                } else if ty.is_signal() {
                    self.resolve_signal_pointer(data.args()[0])
                } else {
                    self.resolve_value_pointer(data.args()[0])
                };
                let target_ty = self.unit.value_type(data.args()[0]);
                let ptr = self.exec_insext(data.opcode(), &target_ty, &target, data.imms());
                if ty.is_pointer() {
                    Action::Value(ValueSlot::VariablePointer(ptr))
                } else if ty.is_signal() {
                    Action::Value(ValueSlot::SignalPointer(ptr))
                } else {
                    Action::Value(ValueSlot::Const(self.read_pointer(&ty, &ptr)))
                }
            }

            // Multiplexing
            Opcode::Mux => {
                let ways = self.resolve_value(data.args()[0]);
                let index = self.resolve_value(data.args()[1]);
                match ways {
                    Value::Array(v) => {
                        let index = index.unwrap_int().to_usize();
                        let index = std::cmp::min(v.0.len() - 1, index);
                        Action::Value(ValueSlot::Const(v.extract_field(index)))
                    }
                    _ => panic!("mux on {}", ways),
                }
            }

            // Instantiations are handled by the builder.
            Opcode::Inst => Action::None,

            // Halt trivially suspends the process indefinitely.
            Opcode::Halt if self.unit.is_entity() => Action::None,
            Opcode::Halt => Action::Suspend(None, InstanceState::Done),

            // TODO(fschuiki): Implement the rest or explicitly report errors
            // upon encountering unsupported instructions.
            opc => unimplemented!("{:?}", opc),
        }
    }

    /// Resolve a value to a constant.
    fn resolve_value(&self, id: llhd::ir::Value) -> Value {
        match self.values.get(&id) {
            Some(ValueSlot::Const(k)) => k.clone(),
            x => panic!(
                "expected value {:?} to resolve to a constant, got {:?}",
                id, x
            ),
        }
    }

    // Resolve a value ref to a constant time value.
    fn resolve_delay(&self, id: llhd::ir::Value) -> TimeValue {
        let v = self.resolve_value(id);
        match v.get_time() {
            Some(x) => x.clone(),
            None => panic!(
                "expected value {:?} to resolve to a time constant, got {:?}",
                id, v
            ),
        }
    }

    // Resolve a value to a signal.
    fn resolve_signal(&self, id: llhd::ir::Value) -> SignalRef {
        match self.values.get(&id) {
            Some(ValueSlot::Signal(r)) => *r,
            x => panic!(
                "expected value {:?} to resolve to a signal, got {:?}",
                id, x
            ),
        }
    }

    // Resolve a value to a variable pointer.
    fn resolve_variable_pointer(&self, id: llhd::ir::Value) -> ValuePointer {
        match self.values.get(&id) {
            Some(ValueSlot::Variable(_)) => ValuePointer(vec![ValueSlice {
                target: ValueTarget::Variable(id),
                select: vec![],
                width: self.pointer_width(id),
            }]),
            Some(ValueSlot::VariablePointer(ref ptr)) => ptr.clone(),
            x => panic!(
                "expected value {:?} to resolve to a variable pointer, got {:?}",
                id, x
            ),
        }
    }

    // Resolve a value to a signal pointer.
    fn resolve_signal_pointer(&self, id: llhd::ir::Value) -> ValuePointer {
        match self.values.get(&id) {
            Some(ValueSlot::Signal(sig)) => ValuePointer(vec![ValueSlice {
                target: ValueTarget::Signal(*sig),
                select: vec![],
                width: self.pointer_width(id),
            }]),
            Some(ValueSlot::SignalPointer(ref ptr)) => ptr.clone(),
            x => panic!(
                "expected value {:?} to resolve to a signal pointer, got {:?}",
                id, x
            ),
        }
    }

    // Resolve a value to a value pointer.
    fn resolve_value_pointer(&self, id: llhd::ir::Value) -> ValuePointer {
        ValuePointer(vec![ValueSlice {
            target: ValueTarget::Value(id),
            select: vec![],
            width: self.pointer_width(id),
        }])
    }

    /// Determine the pointer result type.
    fn pointer_result_type<'b>(&self, ty: &'b llhd::Type) -> &'b llhd::Type {
        match **ty {
            llhd::PointerType(ref ty) => ty,
            llhd::SignalType(ref ty) => ty,
            _ => ty,
        }
    }

    /// Determine the width of a pointer or value, for the purpose of pointer
    /// operations.
    ///
    /// Returns the length of arrays or integers, or 0 for structs.
    fn pointer_width(&self, id: llhd::ir::Value) -> usize {
        let ty = self.unit.value_type(id);
        let ty = match &*ty {
            llhd::PointerType(ty) => ty,
            llhd::SignalType(ty) => ty,
            _ => &ty,
        };
        match **ty {
            llhd::IntType(w) => w,
            llhd::ArrayType(w, _) => w,
            llhd::StructType(..) => 0,
            _ => panic!("{} has no pointer width", ty),
        }
    }

    /// Calculate the time at which an event occurs, given an optional delay. If
    /// the delay is omitted, the next delta cycle is returned.
    fn time_after_delay(&self, delay: &TimeValue) -> TimeValue {
        use num::{zero, Zero};
        let mut time = self.time.time().clone();
        let mut delta = self.time.delta();
        let mut epsilon = self.time.epsilon();
        if !delay.time().is_zero() {
            time += delay.time();
            delta = 0;
            epsilon = 0;
        }
        if delay.delta() != 0 {
            delta += delay.delta();
            epsilon = 0;
        }
        epsilon += delay.epsilon();
        TimeValue::new(time, delta, epsilon)
    }

    /// Calculate the absolute time of the next delta step.
    fn time_after_delta(&self) -> TimeValue {
        TimeValue::new(self.time.time().clone(), self.time.delta() + 1, 0)
    }

    /// Read the target value of a pointer.
    pub fn read_pointer(&self, ty: &llhd::Type, ptr: &ValuePointer) -> Value {
        // Map each slice to its corresponding subresult.
        let mut results = ptr.0.iter().map(|s| (self.read_pointer_slice(s), s.width));

        // Determine the type of the resulting value.
        let inner_ty = match **ty {
            llhd::PointerType(ref ty) => ty,
            llhd::SignalType(ref ty) => ty,
            _ => ty,
        };

        // Otherwise concatenate the results.
        match **ty {
            llhd::IntType(w) => {
                let mut value = IntValue::from_usize(w, 0);
                let mut offset = 0;
                for (result, width) in results {
                    value.insert_slice(offset, width, result.unwrap_int());
                    offset += width;
                }
                assert_eq!(offset, w);
                value.into()
            }
            llhd::ArrayType(w, _) => {
                let mut values = vec![];
                for (result, _) in results {
                    values.extend(result.unwrap_array().0.iter().cloned());
                }
                assert_eq!(values.len(), w);
                ArrayValue::new(values).into()
            }
            _ if ptr.0.len() == 1 => results.next().unwrap().0,
            _ => panic!("multi-slice concat on {}", ty),
        }
    }

    /// Read the target value of a pointer slice.
    pub fn read_pointer_slice(&self, ptr: &ValueSlice) -> Value {
        let mut value = self.read_pointer_target(ptr.target);
        for &select in &ptr.select {
            match select {
                ValueSelect::Field(idx) => match value {
                    Value::Array(v) => value = v.extract_field(idx),
                    Value::Struct(v) => value = v.extract_field(idx),
                    _ => panic!("access field {} in {} ({:?})", idx, value, ptr.target),
                },
                ValueSelect::Slice(off, len) => match value {
                    Value::Int(v) => value = v.extract_slice(off, len).into(),
                    Value::Array(v) => value = v.extract_slice(off, len).into(),
                    _ => panic!(
                        "access slice {},{} in {} ({:?})",
                        off, len, value, ptr.target
                    ),
                },
            }
        }
        value
    }

    /// Read the value of a pointer target.
    pub fn read_pointer_target(&self, target: ValueTarget) -> Value {
        match target {
            ValueTarget::Value(v) => self.resolve_value(v),
            ValueTarget::Variable(v) => match self.values[&v] {
                ValueSlot::Variable(ref k) => k.clone(),
                _ => panic!(
                    "pointer target {:?} did not resolve to a variable value",
                    target
                ),
            },
            ValueTarget::Signal(v) => self.signals[v.as_usize()].value().clone(),
        }
    }

    /// Execute a shift operation.
    ///
    /// This merely modifies pointers. Actual computation of the shift result
    /// is performed in `read_pointer`.
    pub fn exec_shift(
        &self,
        op: Opcode,
        base: &ValuePointer,
        hidden: &ValuePointer,
        amount: &Value,
    ) -> ValuePointer {
        // Map the shift amount to a usize and clamp to the maximum shift.
        let amount = amount.unwrap_int().to_usize();
        let amount = std::cmp::min(hidden.width(), amount);

        // Compute the length of the selected slices from the base and hidden
        // pointers.
        let base_len = base.width() as isize - amount as isize;
        let hidden_len = amount as isize;

        // Compute the offsets of the selected slice from the base and hidden
        // pointers, and determine whether in the output the base or hidden
        // values come first (i.e. shift the hidden in at the bottom or top).
        let (base_off, hidden_off, base_first) = match op {
            Opcode::Shl => (0, hidden.width() as isize - amount as isize, false),
            Opcode::Shr => (amount as isize, 0, true),
            _ => panic!("{} is not a shift op", op),
        };
        let base_slice = (base_off, base_len);
        let hidden_slice = (hidden_off, hidden_len);

        // Assemble an iterator over the base and hidden pointers, plus their
        // slicing information, in the right order.
        let order = match base_first {
            true => vec![base, hidden]
                .into_iter()
                .zip(vec![base_slice, hidden_slice].into_iter()),
            false => vec![hidden, base]
                .into_iter()
                .zip(vec![hidden_slice, base_slice].into_iter()),
        };

        // Create an updated set of pointer slices by fusing the base and hidden
        // pointers according to the slicing and ordering determined above.
        let slices = order
            .flat_map(|(ptr, (off, len))| {
                ptr.offset_slices().flat_map(move |(slice_offset, slice)| {
                    use std::cmp::{max, min};
                    let clamp = |x| max(0isize, min(slice.width as isize, x)) as usize;

                    // Translate the offset and length determined for the shift
                    // to the slice-local equivalent.
                    let actual_off = clamp(off - slice_offset as isize);
                    let actual_end = clamp(off + len - slice_offset as isize);
                    let actual_len = actual_end - actual_off;

                    // Extract exactly the slice determined above.
                    if actual_off == 0 && actual_len == slice.width {
                        Some(slice.clone())
                    } else if actual_len == 0 {
                        None
                    } else {
                        let mut s = slice.clone();
                        s.width = actual_len;
                        s.select.push(ValueSelect::Slice(actual_off, actual_len));
                        Some(s)
                    }
                })
            })
            .collect();

        ValuePointer(slices)
    }

    /// Execute an insert or extract operation.
    ///
    /// This merely modifies pointers. Actual computation of the insertion or
    /// extraction result is performed in `read_pointer`.
    pub fn exec_insext(
        &self,
        op: Opcode,
        target_ty: &llhd::Type,
        target: &ValuePointer,
        imms: &[usize],
    ) -> ValuePointer {
        match op {
            // Field accesses simply go through the slices until they find the
            // one which is being targeted, at which point that specific slice
            // is extracted and a `Field(..)` access is added.
            Opcode::InsField | Opcode::ExtField => {
                let field = imms[0];
                for (offset, slice) in target.offset_slices() {
                    // Match this slice if it's either a struct slice, or the
                    // accessed index is in range.
                    if slice.width == 0 || (field >= offset && field < offset + slice.width) {
                        let mut s = slice.clone();
                        let ty = match **target_ty {
                            llhd::PointerType(ref ty) => ty,
                            llhd::SignalType(ref ty) => ty,
                            _ => target_ty,
                        };
                        let field_ty = match **ty {
                            llhd::ArrayType(_, ref ty) => ty,
                            llhd::StructType(ref f) => &f[field],
                            _ => panic!("cannot field access into {}", ty),
                        };
                        s.width = match **field_ty {
                            llhd::IntType(w) => w,
                            llhd::ArrayType(w, _) => w,
                            _ => 0,
                        };
                        s.select.push(ValueSelect::Field(field - offset));
                        return ValuePointer(vec![s]);
                    }
                }
                panic!("field {} out of bounds in {:?}", field, target);
            }

            // Slice accesses go through the slices and filter out all the ones
            // that do not fall within the range, and adjust the ranges of the
            // ones that do.
            Opcode::InsSlice | Opcode::ExtSlice => {
                let offset = imms[0];
                let length = imms[1];
                let mut slices = vec![];
                for (slice_offset, slice) in target.offset_slices() {
                    use std::cmp::{max, min};
                    let clamp = |x| max(0isize, min(slice.width as isize, x)) as usize;

                    // Shift the global slice bounds into this slice's domain.
                    let actual_off = clamp(offset as isize - slice_offset as isize);
                    let actual_end =
                        clamp(offset as isize + length as isize - slice_offset as isize);
                    let actual_len = actual_end - actual_off;

                    // Extract exactly the slice determined above.
                    if actual_off == 0 && actual_len == slice.width {
                        slices.push(slice.clone());
                    } else if actual_len == 0 {
                        continue;
                    } else {
                        let mut s = slice.clone();
                        s.width = actual_len;
                        s.select.push(ValueSelect::Slice(actual_off, actual_len));
                        slices.push(s);
                    }
                }
                ValuePointer(slices)
            }

            _ => panic!("{} is not an insert/extract op", op),
        }
    }
}

/// An action to be taken as the result of an instruction's execution.
#[derive(Debug)]
enum Action {
    /// No action.
    None,
    /// Change the instruction's entry in the value table. Used by instructions
    /// that yield a value to change that value.
    Value(ValueSlot),
    /// Change another value's entry in the value table. Used by instructions
    /// to simulate writing to memory.
    Store(ValuePointer, Value),
    /// Add an event to the event queue.
    Event(Event),
    /// Transfer control to a different block, executing that block's
    /// instructions.
    Jump(llhd::ir::Block),
    /// Suspend execution of the current instance and change the instance's
    /// state.
    Suspend(Option<llhd::ir::Block>, InstanceState),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Action::None => write!(f, "<no action>"),
            Action::Value(ref v) => write!(f, "= {:?}", v),
            Action::Store(ref ptr, ref v) => write!(f, "*{:?} = {:?}", ptr, v),
            Action::Event(ref ev) => write!(f, "@{} {:?} <= {:?}", ev.time, ev.signal, ev.value),
            Action::Jump(..) | Action::Suspend(..) => write!(f, "{:?}", self),
        }
    }
}

/// Modify a pointer.
///
/// This applies a value to a pointer and returns the modified values for
/// each of the pointer slices.
pub fn write_pointer(ptr: &ValuePointer, into: &mut [Value], value: &Value) {
    for (i, (off, s)) in ptr.offset_slices().enumerate() {
        // Extract the slice of the value that maps to this pointer
        // slice.
        let subvalue = if s.width != 0 {
            match value {
                Value::Int(v) => v.extract_slice(off, s.width).into(),
                Value::Array(v) => v.extract_slice(off, s.width).into(),
                _ => panic!(
                    "cannot slice {} into {},{} for write to pointer slice {:?}",
                    value, off, s.width, s
                ),
            }
        } else {
            value.clone()
        };

        // Write to this pointer slice.
        write_pointer_slice(s, &mut into[i], subvalue);
    }
}

/// Modify a pointer slice.
///
/// This applies a value to a pointer slice and returns the modified value.
pub fn write_pointer_slice(ptr: &ValueSlice, into: &mut Value, value: Value) {
    // trace!("writing {} to {:?}", value, ptr);
    write_pointer_select(&ptr.select, into, value);
}

/// Modify a pointer selection.
///
/// This applies one single select operation to a pointer. Modifies the
/// value and returns an updated version.
pub fn write_pointer_select(select: &[ValueSelect], into: &mut Value, value: Value) {
    if select.is_empty() {
        *into = value;
        return;
    }
    match select[0] {
        ValueSelect::Field(index) => match into {
            Value::Array(v) => {
                let mut sub = v.extract_field(index);
                write_pointer_select(&select[1..], &mut sub, value);
                v.insert_field(index, sub);
            }
            Value::Struct(v) => {
                let mut sub = v.extract_field(index);
                write_pointer_select(&select[1..], &mut sub, value);
                v.insert_field(index, sub);
            }
            _ => panic!("access field {} in {}", index, into),
        },
        ValueSelect::Slice(offset, length) => match into {
            Value::Int(ref mut v) => {
                let mut sub = v.extract_slice(offset, length).into();
                write_pointer_select(&select[1..], &mut sub, value);
                v.insert_slice(offset, length, sub.unwrap_int());
            }
            Value::Array(v) => {
                let mut sub = v.extract_slice(offset, length).into();
                write_pointer_select(&select[1..], &mut sub, value);
                v.insert_slice(offset, length, sub.unwrap_array());
            }
            _ => panic!("access slice {},{} in {}", offset, length, into),
        },
    }
}
