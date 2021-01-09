// Copyright (c) 2017-2021 Fabian Schuiki

//! Analysis passes on the IR
//!
//! This module implements various analysis passes on the IR.

mod domtree;
mod preds;
mod trg;

pub use self::domtree::*;
pub use self::preds::*;
pub use self::trg::*;
