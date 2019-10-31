// Copyright (c) 2017-2019 Fabian Schuiki

//! Optimization infrastructure.
//!
//! This module implements infrastructure used by the optimization system which
//! operates on LLHD IR.

mod pass;

pub use pass::*;

pub mod prelude {
    pub use super::pass::*;
}
