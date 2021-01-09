// Copyright (c) 2017-2021 Fabian Schuiki

//! Optimization infrastructure.
//!
//! This module implements infrastructure used by the optimization system which
//! operates on LLHD IR.

mod pass;

pub use pass::*;

/// Contains common types that can be glob-imported (`*`) for convenience
/// from pass module.
pub mod prelude {
    pub use super::pass::*;
}
