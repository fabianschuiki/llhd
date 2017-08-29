// Copyright (c) 2017 Fabian Schuiki

//! Unreachable block deletion.
//!
//! This optimization pass removes all blocks from the surrounding
//! sequential body that cannot be reached from the entry point.
//!
//! ## Run After
//!
//! - constant branch elision
//!
//! ## Algorithm
//!
//! - Iterate over every sequential body in the module.
//! - Start at the entry block of the sequential body.
//! - Add the block to the set of reachable blocks. If the block has a
//!   branch terminator, recur for each destination block.
//! - Iterate over all blocks in the sequential body. If the block is
//!   not in the reachable set, remove it. Make sure it is also removed
//!   from any of the phi nodes.
