// Copyright (c) 2017 Fabian Schuiki

//! Facilities to emit a module as human-readable assembly, or to parse such
//! assembly back into a module.

pub mod reader;
pub mod writer;

pub use self::reader::parse_str;
pub use self::writer::Writer;
