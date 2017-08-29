// Copyright (c) 2017 Fabian Schuiki

//! Constant branch elision.
//!
//! This optimization pass replaces every conditional branch which has a
//! constant condition with a corresponding unconditional branch.
//!
//! ## Run After
//!
//! - constant folding
//!
//! ## Algorithm
//!
//! - Visit every branch instruction.
//! - If the condition is a constant and its value can be evaluated,
//!   replace the instruction with a new unconditional branch to the
//!   corresponding label.
