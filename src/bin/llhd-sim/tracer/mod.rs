// Copyright (c) 2017-2021 Fabian Schuiki

//! A simulation tracer that can store the generated waveform to disk.

use crate::state::{SignalRef, State};
use std::collections::HashSet;

/// A simulation tracer that can operate on the simulation trace as it is being
/// generated.
pub trait Tracer {
    /// Called once at the beginning of the simulation.
    fn init(&mut self, state: &State);

    /// Called by the simulation engine after each time step.
    fn step(&mut self, state: &State, changed: &HashSet<SignalRef>);

    /// Called once at the end of the simulation.
    fn finish(&mut self, state: &State);
}

/// A null tracer that does nothing.
pub struct NullTracer;

impl Tracer for NullTracer {
    fn init(&mut self, _: &State) {}
    fn step(&mut self, _: &State, _: &HashSet<SignalRef>) {}
    fn finish(&mut self, _: &State) {}
}

// Import the actual tracers.
mod dump;
mod vcd;
pub use dump::*;
pub use vcd::*;
