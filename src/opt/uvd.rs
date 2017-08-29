// Copyright (c) 2017 Fabian Schuiki

//! Unused value deletion.
//!
//! This optimization pass removes all instructions whose value is never
//! used.
//!
//! ## Algorithm
//!
//! - Iterate over every instruction. If the instruction has no users,
//!   add it to the set of pending removals.
//! - Pop the next instruction off the set and remove it from its
//!   parent.
//! - For every operand to the instruction, if it has no users,
//!   add it to the set of pending removals.
//! - Repeat until the set becomes empty.
