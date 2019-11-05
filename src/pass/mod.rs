// Copyright (c) 2017-2019 Fabian Schuiki

//! Optimization and analysis passes on LLHD IR.
//!
//! This module implements various passes that analyze or mutate an LLHD
//! intermediate representation.

pub mod cf;
pub mod cfs;
pub mod dce;
pub mod gcse;
pub mod licm;
pub mod tcm;

pub use cf::ConstFolding;
pub use cfs::ControlFlowSimplification;
pub use dce::DeadCodeElim;
pub use gcse::GlobalCommonSubexprElim;
pub use licm::LoopIndepCodeMotion;
pub use tcm::TemporalCodeMotion;
