// Copyright (c) 2017-2019 Fabian Schuiki

//! Optimization and analysis passes on LLHD IR.
//!
//! This module implements various passes that analyze or mutate an LLHD
//! intermediate representation.

pub mod const_folding;
pub mod dead_code_elim;
pub mod gcse;

pub use const_folding::ConstFolding;
pub use dead_code_elim::DeadCodeElim;
pub use gcse::GlobalCommonSubexprElim;
