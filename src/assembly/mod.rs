// Copyright (c) 2017 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

pub mod writer;
pub mod reader;

pub use self::writer::Writer;
pub use self::reader::parse_str;
