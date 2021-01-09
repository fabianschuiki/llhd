// Copyright (c) 2017-2021 Fabian Schuiki

//! Array values
//!
//! This module implements array value operations.

use crate::{
    ty::{array_ty, Type},
    value::Value,
};
use std::fmt::{Debug, Display};

/// An array value.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayValue(pub Vec<Value>);

impl ArrayValue {
    /// Create a new uniform array.
    pub fn new_uniform(length: usize, value: Value) -> Self {
        ArrayValue(std::iter::repeat(value).take(length).collect())
    }

    /// Create a new array.
    pub fn new(values: Vec<Value>) -> Self {
        ArrayValue(values)
    }

    /// Create a new zero-valued array.
    pub fn zero(length: usize, ty: &Type) -> Self {
        ArrayValue::new_uniform(length, Value::zero(ty))
    }

    /// Get the type of the value.
    pub fn ty(&self) -> Type {
        array_ty(
            self.0.len(),
            self.0.get(0).expect("empty array has no proper type").ty(),
        )
    }
}

impl Display for ArrayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        write!(f, "[")?;
        for v in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
            first = false;
        }
        write!(f, "]")
    }
}

impl Debug for ArrayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Slicing.
impl ArrayValue {
    /// Extract a single element from the array.
    pub fn extract_field(&self, idx: usize) -> Value {
        self.0[idx].clone()
    }

    /// Extract a slice of elements from the array.
    pub fn extract_slice(&self, off: usize, len: usize) -> ArrayValue {
        ArrayValue::new(self.0[off..off + len].to_vec())
    }

    /// Insert a single element into the array.
    pub fn insert_field(&mut self, idx: usize, value: Value) {
        self.0[idx] = value;
    }

    /// Insert a slice of elements into the array.
    pub fn insert_slice(&mut self, off: usize, len: usize, value: &ArrayValue) {
        assert_eq!(len, value.0.len());
        self.0[off..off + len].clone_from_slice(&value.0);
    }
}
