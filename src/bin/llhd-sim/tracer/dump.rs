// Copyright (c) 2017-2021 Fabian Schuiki

//! A simple tracer for debugging and regression testing.

use crate::{
    state::{Scope, SignalRef, State},
    tracer::Tracer,
    value::Value,
};
use num::{traits::Pow, BigInt, BigRational, FromPrimitive};
use std::{
    collections::{HashMap, HashSet},
    iter::{once, repeat},
};

/// A tracer that emits the simulation trace as a human-readable change dump.
pub struct DumpTracer<T> {
    writer: T,
    precision: BigRational,
    signals: HashMap<SignalRef, String>,
}

impl<T> DumpTracer<T>
where
    T: std::io::Write,
{
    /// Create a new VCD tracer which will write its VCD to `writer`.
    pub fn new(writer: T) -> Self {
        DumpTracer {
            writer: writer,
            precision: BigInt::from_usize(10).unwrap().pow(12usize).into(), // ps
            signals: Default::default(),
        }
    }

    /// Write the value of a signal.
    fn write_value(&mut self, value: &Value) {
        match value {
            Value::Void => (),
            Value::Int(v) => {
                write!(self.writer, "0x{0:01$x}", v.value, (v.width + 3) / 4).unwrap();
            }
            Value::Time(_) => (),
            Value::Array(v) => {
                write!(self.writer, "[").unwrap();
                for (elem, sep) in v.0.iter().zip(once("").chain(repeat(", "))) {
                    write!(self.writer, "{}", sep).unwrap();
                    self.write_value(elem);
                }
                write!(self.writer, "]").unwrap();
            }
            Value::Struct(v) => {
                write!(self.writer, "{{").unwrap();
                for (field, sep) in v.0.iter().zip(once("").chain(repeat(", "))) {
                    write!(self.writer, "{}", sep).unwrap();
                    self.write_value(field);
                }
                write!(self.writer, "}}").unwrap();
            }
        };
    }

    /// Allocate names for all signals.
    fn prepare_scope(&mut self, state: &State, scope: &Scope, prefix: &str) {
        let prefix = format!("{}{}/", prefix, &scope.name[1..]);
        for (&signal, names) in &scope.probes {
            for name in names {
                self.signals
                    .entry(signal)
                    .or_insert_with(|| format!("{}{}", prefix, name));
            }
        }
        for subscope in scope.subscopes.iter() {
            self.prepare_scope(state, subscope, &prefix);
        }
    }
}

impl<T> Tracer for DumpTracer<T>
where
    T: std::io::Write,
{
    fn init(&mut self, state: &State) {
        self.prepare_scope(state, &state.scope, "");

        // let mut signals: Vec<SignalRef> = self.signals.keys().cloned().collect();
        // signals.sort_by_key(|s| &self.signals[s]);
        // for signal in signals {
        //     write!(self.writer, "  {} = ", self.signals[&signal]).unwrap();
        //     self.write_value(state[signal].value());
        //     write!(self.writer, "\n").unwrap();
        // }
    }

    fn step(&mut self, state: &State, changed: &HashSet<SignalRef>) {
        write!(
            self.writer,
            "{}ps {}d {}e\n",
            (state.time.time() * &self.precision).trunc(),
            state.time.delta(),
            state.time.epsilon()
        )
        .unwrap();

        let mut changed: Vec<SignalRef> = changed.iter().cloned().collect();
        changed.sort_by_key(|s| &self.signals[s]);
        for signal in changed {
            write!(self.writer, "  {} = ", self.signals[&signal]).unwrap();
            self.write_value(state[signal].value());
            write!(self.writer, "\n").unwrap();
        }
    }

    fn finish(&mut self, _: &State) {}
}
