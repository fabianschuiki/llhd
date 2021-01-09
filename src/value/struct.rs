// Copyright (c) 2017-2021 Fabian Schuiki

//! Struct values
//!
//! This module implements array value operations.

use crate::{
    ty::{struct_ty, Type},
    value::Value,
};
use std::fmt::{Debug, Display};

/// A struct value.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructValue(pub Vec<Value>);

impl StructValue {
    /// Create a new struct.
    pub fn new(values: Vec<Value>) -> Self {
        StructValue(values)
    }

    /// Create a new zero-valued struct.
    pub fn zero<'a>(tys: impl IntoIterator<Item = &'a Type>) -> Self {
        StructValue::new(tys.into_iter().map(Value::zero).collect())
    }

    /// Get the type of the value.
    pub fn ty(&self) -> Type {
        struct_ty(self.0.iter().map(Value::ty).collect())
    }
}

impl Display for StructValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        write!(f, "{{")?;
        for v in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
            first = false;
        }
        write!(f, "}}")
    }
}

impl Debug for StructValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Slicing.
impl StructValue {
    /// Extract a single field from the struct.
    pub fn extract_field(&self, idx: usize) -> Value {
        self.0[idx].clone()
    }

    /// Insert a field into the struct.
    pub fn insert_field(&mut self, idx: usize, value: Value) {
        self.0[idx] = value;
    }
}
