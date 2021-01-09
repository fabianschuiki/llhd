// Copyright (c) 2017-2021 Fabian Schuiki

//! A Value Change Dump tracer.

use crate::{
    state::{Scope, SignalRef, State},
    tracer::Tracer,
    value::Value,
};
use num::{traits::Pow, BigInt, BigRational, FromPrimitive};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

/// A tracer that emits the simulation trace as VCD.
pub struct VcdTracer<T> {
    writer: RefCell<T>,
    abbrevs: HashMap<SignalRef, Vec<(String, String, usize)>>,
    time: BigRational,
    pending: HashMap<SignalRef, Value>,
    precision: BigRational,
}

impl<T> VcdTracer<T>
where
    T: std::io::Write,
{
    /// Create a new VCD tracer which will write its VCD to `writer`.
    pub fn new(writer: T) -> Self {
        VcdTracer {
            writer: RefCell::new(writer),
            abbrevs: HashMap::new(),
            time: num::zero(),
            pending: HashMap::new(),
            // Hard-code the precision to ps for now. Later on, we might want to
            // make this configurable or automatically determined by the module.
            precision: BigInt::from_usize(10).unwrap().pow(12usize).into(), // ps
        }
    }

    /// Write the value of all signals that have changed since the last flush.
    /// Clears the `pending` set.
    fn flush(&mut self) {
        let time = (&self.time * &self.precision).trunc();
        write!(self.writer.borrow_mut(), "#{}\n", time).unwrap();
        for (signal, value) in std::mem::replace(&mut self.pending, HashMap::new()) {
            for &(ref abbrev, _, offset) in &self.abbrevs[&signal] {
                self.flush_signal(signal, offset, &value, abbrev);
            }
        }
    }

    /// Write the value of a signal. Called at the beginning of the simulation
    /// to perform a variable dump, and during flush once for each signal that
    /// changed.
    fn flush_signal(&self, signal: SignalRef, offset: usize, value: &Value, abbrev: &str) {
        match value {
            Value::Void => (),
            Value::Int(v) => {
                assert_eq!(offset, 0);
                write!(self.writer.borrow_mut(), "b{:b} {}\n", v.value, abbrev).unwrap();
            }
            Value::Time(_) => (),
            Value::Array(v) => {
                let elems = &v.0;
                self.flush_signal(
                    signal,
                    offset / elems.len(),
                    &elems[offset % elems.len()],
                    abbrev,
                );
            }
            Value::Struct(v) => {
                let fields = &v.0;
                self.flush_signal(
                    signal,
                    offset / fields.len(),
                    &fields[offset % fields.len()],
                    abbrev,
                );
            }
            // _ => panic!(
            //     "flush non-const/non-aggregate signal {:?} with value {:?}",
            //     signal, value
            // ),
        };
    }

    /// Allocate short names and emit `$scope` statement.
    fn prepare_scope(&mut self, state: &State, scope: &Scope, index: &mut usize) {
        write!(
            self.writer.borrow_mut(),
            "$scope module {} $end\n",
            scope.name.replace('.', "_")
        )
        .unwrap();
        let mut probed_signals: Vec<_> = scope.probes.keys().cloned().collect();
        probed_signals.sort();
        for sigref in probed_signals {
            for name in &scope.probes[&sigref] {
                self.prepare_signal(
                    state,
                    sigref,
                    state[sigref].ty().unwrap_signal(),
                    name,
                    index,
                    0,
                    1,
                );
            }
        }
        for subscope in scope.subscopes.iter() {
            self.prepare_scope(state, subscope, index);
        }
        write!(self.writer.borrow_mut(), "$upscope $end\n").unwrap();
    }

    /// Expand signals and allocate short names.
    fn prepare_signal(
        &mut self,
        state: &State,
        sigref: SignalRef,
        ty: &llhd::Type,
        name: &str,
        index: &mut usize,
        offset: usize,
        stride: usize,
    ) {
        match **ty {
            llhd::IntType(width) => {
                // Allocate short name for the probed signal.
                let mut idx = *index;
                let mut abbrev = String::new();
                loop {
                    abbrev.push((33 + idx % 94) as u8 as char);
                    idx /= 94;
                    if idx == 0 {
                        break;
                    }
                }
                *index += 1;

                // Write the abbreviations for this signal.
                let abbrevs_for_signal = self.abbrevs.entry(sigref).or_insert_with(Vec::new);
                write!(
                    self.writer.borrow_mut(),
                    "$var wire {} {} {} $end\n",
                    width,
                    abbrev,
                    name
                )
                .unwrap();
                abbrevs_for_signal.push((abbrev, name.to_owned(), offset));
            }
            llhd::ArrayType(width, ref subty) => {
                for i in 0..width {
                    self.prepare_signal(
                        state,
                        sigref,
                        subty,
                        &format!("{}[{}]", name, i),
                        index,
                        offset + i * stride,
                        stride * width,
                    );
                }
            }
            llhd::StructType(ref fields) => {
                for (i, subty) in fields.iter().enumerate() {
                    self.prepare_signal(
                        state,
                        sigref,
                        subty,
                        &format!("{}.{}", name, i),
                        index,
                        offset + i * stride,
                        stride * fields.len(),
                    );
                }
            }
            // _ => (),
            ref x => panic!("signal of type {} not supported in VCD", x),
        }
    }
}

impl<T> Tracer for VcdTracer<T>
where
    T: std::io::Write,
{
    fn init(&mut self, state: &State) {
        // Dump the VCD header.
        write!(
            self.writer.borrow_mut(),
            "$version\nllhd-sim {}\n$end\n",
            clap::crate_version!()
        )
        .unwrap();
        write!(self.writer.borrow_mut(), "$timescale 1ps $end\n").unwrap();
        self.prepare_scope(state, &state.scope, &mut 0);
        write!(self.writer.borrow_mut(), "$enddefinitions $end\n").unwrap();

        // Dump the variables.
        write!(self.writer.borrow_mut(), "$dumpvars\n").unwrap();
        for &signal in state.probes.keys() {
            if let Some(abbrevs) = self.abbrevs.get(&signal) {
                for &(ref abbrev, _, offset) in abbrevs {
                    self.flush_signal(signal, offset, state[signal].value(), abbrev);
                }
            }
        }
        write!(self.writer.borrow_mut(), "$end\n").unwrap();
    }

    fn step(&mut self, state: &State, changed: &HashSet<SignalRef>) {
        // If the physical time in seconds of the simulation changed, flush the
        // aggregated pending changes and update the time.
        if self.time != *state.time.time() {
            self.flush();
            self.time = state.time.time().clone();
        }

        // Mark the changed signals for consideration during the next flush.
        self.pending.extend(
            changed
                .iter()
                .map(|&signal| (signal, state[signal].value().clone())),
        );
    }

    fn finish(&mut self, _: &State) {
        self.flush();
    }
}
