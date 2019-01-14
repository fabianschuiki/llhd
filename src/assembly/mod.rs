// Copyright (c) 2017 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

mod reader;
mod writer;

pub use self::{reader::parse_str, writer::Writer};
