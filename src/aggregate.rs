// Copyright (c) 2019 Fabian Schuiki

//! Aggregate values such as structs and arrays.

#![deny(missing_docs)]

use crate::{Type, Value, ValueId, ValueRef, INLINE_VALUE_ID};
use std::sync::Arc;

/// An aggregate value.
pub type Aggregate = Arc<AggregateKind>;

/// The different forms an aggregate can take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateKind {
    /// A struct.
    Struct(StructAggregate),
    /// An array.
    Array(ArrayAggregate),
}

impl From<StructAggregate> for AggregateKind {
    fn from(a: StructAggregate) -> AggregateKind {
        AggregateKind::Struct(a)
    }
}

impl From<ArrayAggregate> for AggregateKind {
    fn from(a: ArrayAggregate) -> AggregateKind {
        AggregateKind::Array(a)
    }
}

impl Value for AggregateKind {
    fn id(&self) -> ValueId {
        INLINE_VALUE_ID
    }

    fn ty(&self) -> Type {
        match *self {
            AggregateKind::Struct(ref a) => a.ty().clone(),
            AggregateKind::Array(ref a) => a.ty().clone(),
        }
    }

    fn name(&self) -> Option<&str> {
        None
    }
}

/// A struct aggregate value such as `{i32 42, i64 9001}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructAggregate {
    ty: Type,
    fields: Vec<ValueRef>,
}

impl StructAggregate {
    /// Create a new struct aggregate value.
    ///
    /// Constructs a new struct aggregate value with the given `fields`. The
    /// length of `fields` must match the number of fields in `ty`, and `ty`
    /// must be a struct type.
    pub fn new(ty: Type, fields: Vec<ValueRef>) -> StructAggregate {
        assert!(ty.is_struct());
        assert_eq!(ty.unwrap_struct().len(), fields.len());
        StructAggregate { ty, fields }
    }

    /// Get the type of the struct.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get the fields of the struct.
    pub fn fields(&self) -> &[ValueRef] {
        &self.fields
    }

    /// Get a mutable reference to the fields of the struct.
    pub fn fields_mut(&mut self) -> &mut [ValueRef] {
        &mut self.fields
    }
}

/// An array aggregate value such as `[42 x i64 1337]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayAggregate {
    ty: Type,
    elements: Vec<ValueRef>,
}

impl ArrayAggregate {
    /// Create a new array aggregate value.
    ///
    /// Constructs a new array aggregate value with the given `elements`. The
    /// length of `elements` must match the number of elements in `ty`, and `ty`
    /// must be an array type.
    pub fn new(ty: Type, elements: Vec<ValueRef>) -> ArrayAggregate {
        assert!(ty.is_array());
        assert_eq!(ty.unwrap_array().0, elements.len());
        ArrayAggregate { ty, elements }
    }

    /// Get the type of the array.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get the elements of the array.
    pub fn elements(&self) -> &[ValueRef] {
        &self.elements
    }

    /// Get a mutable reference to the elements of the array.
    pub fn fields_mut(&mut self) -> &mut [ValueRef] {
        &mut self.elements
    }
}
