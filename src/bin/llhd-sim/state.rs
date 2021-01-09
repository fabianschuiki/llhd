// Copyright (c) 2017-2021 Fabian Schuiki

//! The simulation state.

#![allow(unused_imports)]

use crate::value::{TimeValue, Value};
use llhd::ir::Unit;
use num::zero;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet},
    fmt,
    ops::{Index, IndexMut},
    sync::Mutex,
};

/// A simulation state.
pub struct State<'ll> {
    /// The LLHD module being simulated.
    pub module: &'ll llhd::ir::Module,
    /// The signals present in the simulation.
    pub signals: Vec<Signal>,
    /// The probed signals.
    pub probes: HashMap<SignalRef, Vec<String>>,
    /// The root scope of the simulation.
    pub scope: Scope,
    /// The process and entity instances in the simulation.
    pub insts: Vec<Mutex<Instance<'ll>>>,
    /// The current simulation time.
    pub time: TimeValue,

    /// The current state of the event queue.
    pub events: BTreeMap<TimeValue, HashMap<ValuePointer, Value>>,
    /// The current wakeup queue for instances.
    pub timed: BTreeMap<TimeValue, HashSet<InstanceRef>>,
}

impl<'ll> State<'ll> {
    //     /// Create a new simulation state.
    //     pub fn new(
    //         module: &'ll Module,
    //         signals: Vec<Signal>,
    //         probes: HashMap<SignalRef, Vec<String>>,
    //         scope: Scope,
    //         insts: Vec<Mutex<Instance<'ll>>>,
    //     ) -> State<'ll> {
    //         State {
    //             module: module,
    //             context: ModuleContext::new(module),
    //             time: TimeValue::new(zero(), zero(), zero()),
    //             signals,
    //             probes,
    //             scope,
    //             insts,
    //             events: BinaryHeap::new(),
    //             timed: BinaryHeap::new(),
    //         }
    //     }

    //     /// Get the module whose state this object holds.
    //     pub fn module(&self) -> &'ll Module {
    //         self.module
    //     }

    //     /// Get the module context for the module whose state this object holds
    //     pub fn context(&self) -> &ModuleContext {
    //         &self.context
    //     }

    //     /// Get the current simulation time.
    //     pub fn time(&self) -> &TimeValue {
    //         &self.time
    //     }

    //     /// Change the current simulation time.
    //     pub fn set_time(&mut self, time: TimeValue) {
    //         self.time = time
    //     }

    //     /// Get a slice of instances in the state.
    //     pub fn instances(&self) -> &[Mutex<Instance<'ll>>] {
    //         &self.insts
    //     }

    //     /// Get a mutable slice of instances in the state.
    //     pub fn instances_mut(&mut self) -> &mut [Mutex<Instance<'ll>>] {
    //         &mut self.insts
    //     }

    //     /// Get a reference to an instance in the state.
    //     pub fn instance(&self, ir: InstanceRef) -> &Mutex<Instance<'ll>> {
    //         &self.insts[ir.0]
    //     }

    // /// Obtain a reference to one of the state's signals.
    // pub fn signal(&self, sr: SignalRef) -> &Signal {
    //     &self.signals[sr.0]
    // }

    // /// Obtain a mutable reference to one of the state's signals.
    // pub fn signal_mut(&mut self, sr: SignalRef) -> &mut Signal {
    //     &mut self.signals[sr.0]
    // }

    //     /// Get a reference to all signals of this state.
    //     pub fn signals(&self) -> &[Signal] {
    //         &self.signals
    //     }

    //     /// Get a map of all probe signals and the corresponding names.
    //     pub fn probes(&self) -> &HashMap<SignalRef, Vec<String>> {
    //         &self.probes
    //     }

    //     /// Get the root scope of the design.
    //     pub fn scope(&self) -> &Scope {
    //         &self.scope
    //     }

    /// Add a set of events to the schedule.
    pub fn schedule_events<I>(&mut self, iter: I)
    where
        I: Iterator<Item = Event>,
    {
        let time = self.time.clone();
        let probes = self.probes.clone();
        for i in iter {
            assert!(i.time >= time);
            debug!(
                "Schedule {} <- {}  [@ {}]",
                i.signal
                    .0
                    .iter()
                    .map(|s| {
                        let sig = s.target.unwrap_signal();
                        probes
                            .get(&sig)
                            .map(|n| n[0].clone())
                            .unwrap_or_else(|| format!("{:?}", sig))
                    })
                    .collect::<String>(),
                i.value,
                i.time,
            );
            self.events
                .entry(i.time)
                .or_insert_with(Default::default)
                .insert(i.signal, i.value);
        }
    }

    /// Add a set of timed instances to the schedule.
    pub fn schedule_timed<I>(&mut self, iter: I)
    where
        I: Iterator<Item = TimedInstance>,
    {
        let time = self.time.clone();
        for i in iter {
            assert!(i.time >= time);
            debug!("Schedule {:?}  [@ {}]", i.inst, i.time);
            self.timed
                .entry(i.time)
                .or_insert_with(Default::default)
                .insert(i.inst);
        }
    }

    /// Dequeue all events due at the current time.
    pub fn take_next_events(&mut self) -> impl Iterator<Item = (ValuePointer, Value)> {
        if let Some(x) = self.events.remove(&self.time) {
            x.into_iter()
        } else {
            HashMap::new().into_iter()
        }
    }

    /// Dequeue all timed instances due at the current time.
    pub fn take_next_timed(&mut self) -> impl Iterator<Item = InstanceRef> {
        if let Some(x) = self.timed.remove(&self.time) {
            x.into_iter()
        } else {
            HashSet::new().into_iter()
        }
    }

    /// Determine the time of the next simulation step. This is the lowest time
    /// value of any event or wake up request in the schedule. If both the event
    /// and timed instances queue are empty, None is returned.
    pub fn next_time(&self) -> Option<TimeValue> {
        use std::cmp::min;
        match (self.events.keys().next(), self.timed.keys().next()) {
            (Some(e), Some(t)) => Some(min(e, t).clone()),
            (Some(e), None) => Some(e.clone()),
            (None, Some(t)) => Some(t.clone()),
            (None, None) => None,
        }
    }
}

impl Index<SignalRef> for State<'_> {
    type Output = Signal;

    fn index(&self, idx: SignalRef) -> &Self::Output {
        &self.signals[idx.0]
    }
}

impl IndexMut<SignalRef> for State<'_> {
    fn index_mut(&mut self, idx: SignalRef) -> &mut Self::Output {
        &mut self.signals[idx.0]
    }
}

impl<'ll> Index<InstanceRef> for State<'ll> {
    type Output = Mutex<Instance<'ll>>;

    fn index(&self, idx: InstanceRef) -> &Self::Output {
        &self.insts[idx.0]
    }
}

impl IndexMut<InstanceRef> for State<'_> {
    fn index_mut(&mut self, idx: InstanceRef) -> &mut Self::Output {
        &mut self.insts[idx.0]
    }
}

/// A unique handle to a signal in a simulation state.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SignalRef(usize);

impl SignalRef {
    /// Create a new signal reference.
    pub fn new(id: usize) -> SignalRef {
        SignalRef(id)
    }

    /// Return the underlying index of this reference.
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl fmt::Debug for SignalRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

/// A signal in a simulation state.
pub struct Signal {
    ty: llhd::Type,
    value: Value,
}

impl Signal {
    /// Create a new signal.
    pub fn new(ty: llhd::Type, value: Value) -> Signal {
        Signal {
            ty: ty,
            value: value,
        }
    }

    /// Get the signal's type.
    pub fn ty(&self) -> &llhd::Type {
        &self.ty
    }

    /// Get the signal's current value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Change the signal's current value. Returns whether the values were
    /// identical.
    pub fn set_value(&mut self, value: Value) -> bool {
        if self.value != value {
            self.value = value;
            true
        } else {
            false
        }
    }
}

/// An instance of a process or entity.
pub struct Instance<'ll> {
    pub values: HashMap<llhd::ir::Value, ValueSlot>,
    pub kind: InstanceKind<'ll>,
    pub state: InstanceState,
    pub signals: Vec<SignalRef>,
    pub signal_values: HashMap<SignalRef, llhd::ir::Value>,
}

impl<'ll> Instance<'ll> {
    //     pub fn new(
    //         values: HashMap<llhd::ir::Value, ValueSlot>,
    //         kind: InstanceKind<'ll>,
    //         inputs: Vec<SignalRef>,
    //         outputs: Vec<SignalRef>,
    //     ) -> Instance<'ll> {
    //         Instance {
    //             values: values,
    //             kind: kind,
    //             state: InstanceState::Ready,
    //             inputs: inputs,
    //             outputs: outputs,
    //         }
    //     }

    //     /// Get the instance's current state.
    //     pub fn state(&self) -> &InstanceState {
    //         &self.state
    //     }

    //     /// Change the instance's current state.
    //     pub fn set_state(&mut self, state: InstanceState) {
    //         self.state = state;
    //     }

    //     pub fn kind(&self) -> &InstanceKind<'ll> {
    //         &self.kind
    //     }

    //     pub fn kind_mut(&mut self) -> &mut InstanceKind<'ll> {
    //         &mut self.kind
    //     }

    //     /// Get a reference to the value table of this instance.
    //     pub fn values(&self) -> &HashMap<llhd::ir::Value, ValueSlot> {
    //         &self.values
    //     }

    /// Access an entry in this instance's value table.
    pub fn value(&self, id: llhd::ir::Value) -> &ValueSlot {
        self.values.get(&id).unwrap()
    }

    /// Change an entry in this instance's value table.
    pub fn set_value(&mut self, id: llhd::ir::Value, value: ValueSlot) {
        self.values.insert(id, value);
    }

    //     /// Get a slice of the instance's input signals.
    //     pub fn inputs(&self) -> &[SignalRef] {
    //         &self.inputs
    //     }

    //     /// Get a slice of the instance's output signals.
    //     pub fn outputs(&self) -> &[SignalRef] {
    //         &self.outputs
    //     }

    /// Get the name of the entity or process.
    pub fn name(&self) -> String {
        match self.kind {
            InstanceKind::Process { prok, .. } => prok.name().to_string(),
            InstanceKind::Entity { entity, .. } => entity.name().to_string(),
        }
    }
}

/// A slot that carries a single value.
///
/// Slots are assigned to each entity in the LLHD graph that may carry a value.
/// Execution of instructions change the value slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSlot {
    /// A signal.
    Signal(SignalRef),
    /// A variable with its current value.
    Variable(Value),
    /// A constant value.
    Const(Value),
    /// A pointer to a variable.
    VariablePointer(ValuePointer),
    /// A pointer to a signal.
    SignalPointer(ValuePointer),
}

impl fmt::Display for ValueSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueSlot::Signal(v) => fmt::Debug::fmt(v, f),
            ValueSlot::Variable(v) => fmt::Display::fmt(v, f),
            ValueSlot::Const(v) => fmt::Display::fmt(v, f),
            ValueSlot::VariablePointer(v) => fmt::Display::fmt(v, f),
            ValueSlot::SignalPointer(v) => fmt::Display::fmt(v, f),
        }
    }
}

/// A pointer to a value.
///
/// A `ValuePointer` represents a variable or signal that is either referenced
/// in its entirety, or by selecting a subset of its elements, bits, or fields.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValuePointer(pub Vec<ValueSlice>);

impl fmt::Display for ValuePointer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let single = self.0.len() == 1;
        if !single {
            write!(f, "[")?;
        }
        let mut first = true;
        for s in &self.0 {
            if f.alternate() && !single {
                write!(f, "\n    ")?;
            } else if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", s)?;
            first = false;
        }
        if !single {
            if f.alternate() {
                write!(f, "\n]")
            } else {
                write!(f, "]")
            }
        } else {
            Ok(())
        }
    }
}

impl ValuePointer {
    /// Compute the width of the pointed at value.
    ///
    /// Returns 0 if it is a struct.
    pub fn width(&self) -> usize {
        self.0.iter().map(|s| s.width).sum()
    }

    /// Get an iterator over the slices which tracks slice offsets.
    pub fn offset_slices(&self) -> impl Iterator<Item = (usize, &ValueSlice)> {
        let mut i = 0;
        self.0.iter().map(move |s| {
            let v = i;
            i += s.width;
            (v, s)
        })
    }
}

/// A slice of a pointer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueSlice {
    /// The targeted value, variable, or signal.
    pub target: ValueTarget,
    /// The selection into the target.
    pub select: Vec<ValueSelect>,
    /// The width of this slice, or 0 if it is a struct.
    pub width: usize,
}

impl fmt::Display for ValueSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.target)?;
        for s in &self.select {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}

/// A pointer target.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueTarget {
    Value(llhd::ir::Value),
    Variable(llhd::ir::Value),
    Signal(SignalRef),
}

impl fmt::Display for ValueTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueTarget::Value(v) => write!(f, "{}", v),
            ValueTarget::Variable(v) => write!(f, "*{}", v),
            ValueTarget::Signal(v) => write!(f, "${:?}", v),
        }
    }
}

impl ValueTarget {
    /// Unwrap the underlying value, or panic.
    #[allow(dead_code)]
    pub fn unwrap_value(&self) -> llhd::ir::Value {
        match *self {
            ValueTarget::Value(v) => v,
            _ => panic!("value target is not a value"),
        }
    }

    /// Unwrap the underlying variable, or panic.
    pub fn unwrap_variable(&self) -> llhd::ir::Value {
        match *self {
            ValueTarget::Variable(v) => v,
            _ => panic!("value target is not a variable"),
        }
    }

    /// Unwrap the underlying signal, or panic.
    pub fn unwrap_signal(&self) -> SignalRef {
        match *self {
            ValueTarget::Signal(v) => v,
            _ => panic!("value target is not a signal"),
        }
    }
}

/// A selection of a part of a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueSelect {
    /// An individual array element or struct field.
    Field(usize),
    /// A slice of array elements or integer bits, given by `(offset, length)`.
    Slice(usize, usize),
}

impl fmt::Display for ValueSelect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueSelect::Field(i) => write!(f, ".{}", i),
            ValueSelect::Slice(o, l) => write!(f, "[{}+:{}]", o, l),
        }
    }
}

/// An instantiation.
pub enum InstanceKind<'ll> {
    Process {
        prok: llhd::ir::Unit<'ll>,
        next_block: Option<llhd::ir::Block>,
    },
    Entity {
        entity: llhd::ir::Unit<'ll>,
    },
}

/// The state an instance can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceState {
    Ready,
    Wait(Option<TimeValue>, Vec<SignalRef>),
    Done,
}

/// A unique reference to an instance in the simulation.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct InstanceRef(usize);

impl InstanceRef {
    /// Create a new instance reference.
    pub fn new(id: usize) -> InstanceRef {
        InstanceRef(id)
    }
}

/// An event that can be scheduled in a binary heap, forming an event queue. The
/// largest element, i.e. the one at the top of the heap, is the one with the
/// lowest time value.
#[derive(Debug, Eq, PartialEq)]
pub struct Event {
    pub time: TimeValue,
    pub signal: ValuePointer,
    pub value: Value,
}

impl Ord for Event {
    fn cmp(&self, rhs: &Event) -> Ordering {
        match self.time.cmp(&rhs.time) {
            Ordering::Equal => self.signal.cmp(&rhs.signal),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, rhs: &Event) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

/// A notice that an instance is in a wait state and wants to be resumed once a
/// certain simulation time has been reached. TimedInstance objects can be
/// scheduled in a binary heap, which forms a wake up queue. The largest
/// element, i.e. the one at the top of the heap, is the one with the lowest
/// time value.
#[derive(Debug, Eq, PartialEq)]
pub struct TimedInstance {
    pub time: TimeValue,
    pub inst: InstanceRef,
}

impl Ord for TimedInstance {
    fn cmp(&self, rhs: &TimedInstance) -> Ordering {
        match self.time.cmp(&rhs.time) {
            Ordering::Equal => self.inst.cmp(&rhs.inst),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl PartialOrd for TimedInstance {
    fn partial_cmp(&self, rhs: &TimedInstance) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

/// A level of hierarchy.
///
/// The scope represents the hierarchy of a design. Each instantiation or
/// process creates a new subscope with its own set of probes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scope {
    /// The name of the scope.
    pub name: String,
    /// The probes in this scope.
    pub probes: HashMap<SignalRef, Vec<String>>,
    /// The subscopes.
    pub subscopes: Vec<Scope>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new(name: impl Into<String>) -> Scope {
        Scope {
            name: name.into(),
            probes: Default::default(),
            subscopes: vec![],
        }
    }

    /// Add a subscope.
    pub fn add_subscope(&mut self, scope: Scope) {
        self.subscopes.push(scope);
    }

    /// Add a probe.
    pub fn add_probe(&mut self, signal: SignalRef, name: String) {
        self.probes.entry(signal).or_insert(Vec::new()).push(name);
    }
}
