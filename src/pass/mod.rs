// Copyright (c) 2017-2019 Fabian Schuiki

//! Optimization and analysis passes on LLHD IR.
//!
//! This module implements various passes that analyze or mutate an LLHD
//! intermediate representation.

pub mod cf;
pub mod cfs;
pub mod dce;
pub mod deseq;
pub mod gcse;
pub mod insim;
pub mod licm;
pub mod proclower;
pub mod tcm;
pub mod vtpp;

pub use cf::ConstFolding;
pub use cfs::ControlFlowSimplification;
pub use dce::DeadCodeElim;
pub use deseq::Desequentialization;
pub use gcse::GlobalCommonSubexprElim;
pub use insim::InstSimplification;
pub use licm::LoopIndepCodeMotion;
pub use proclower::ProcessLowering;
pub use tcm::TemporalCodeMotion;
pub use vtpp::VarToPhiPromotion;
