// Copyright (c) 2017-2021 Fabian Schuiki

//! Value computation
//!
//! This module implements representations for LLHD values.

mod array;
mod int;
mod r#struct;
mod time;

pub use self::time::*;
pub use array::*;
pub use int::*;
pub use r#struct::*;

use crate::ty::Type;
use std::fmt::{Debug, Display};

/// A value.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Value {
    Void,
    Time(TimeValue),
    Int(IntValue),
    Array(ArrayValue),
    Struct(StructValue),
}

impl Value {
    /// Create the zero value for a type.
    pub fn zero(ty: &Type) -> Value {
        use crate::ty::TypeKind::*;
        match ty.as_ref() {
            VoidType => Value::Void,
            IntType(w) => IntValue::zero(*w).into(),
            EnumType(_) => unimplemented!("zero value for {}", ty),
            ArrayType(l, ty) => ArrayValue::zero(*l, ty).into(),
            StructType(tys) => StructValue::zero(tys).into(),
            _ => panic!("no zero value for {}", ty),
        }
    }

    /// If this value is a time, access it.
    pub fn get_time(&self) -> Option<&TimeValue> {
        match self {
            Value::Time(v) => Some(v),
            _ => None,
        }
    }

    /// Unwrap this value as a time, or panic.
    pub fn unwrap_time(&self) -> &TimeValue {
        self.get_time().expect("value is not a time")
    }

    /// If this value is an integer, access it.
    pub fn get_int(&self) -> Option<&IntValue> {
        match self {
            Value::Int(v) => Some(v),
            _ => None,
        }
    }

    /// Unwrap this value as an integer, or panic.
    pub fn unwrap_int(&self) -> &IntValue {
        self.get_int().expect("value is not an integer")
    }

    /// If this value is an array, access it.
    pub fn get_array(&self) -> Option<&ArrayValue> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Unwrap this value as an array, or panic.
    pub fn unwrap_array(&self) -> &ArrayValue {
        self.get_array().expect("value is not an array")
    }

    /// If this value is a struct, access it.
    pub fn get_struct(&self) -> Option<&StructValue> {
        match self {
            Value::Struct(v) => Some(v),
            _ => None,
        }
    }

    /// Unwrap this value as a struct, or panic.
    pub fn unwrap_struct(&self) -> &StructValue {
        self.get_struct().expect("value is not a struct")
    }

    /// Get the type of the value.
    pub fn ty(&self) -> Type {
        use crate::ty::*;
        match self {
            Value::Void => void_ty(),
            Value::Time(v) => v.ty(),
            Value::Int(v) => v.ty(),
            Value::Array(v) => v.ty(),
            Value::Struct(v) => v.ty(),
        }
    }

    /// Check if the value is zero.
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Int(v) => v.is_zero(),
            Value::Time(v) => v.is_zero(),
            Value::Void | Value::Array(..) | Value::Struct(..) => false,
        }
    }

    /// Check if the value is one.
    pub fn is_one(&self) -> bool {
        match self {
            Value::Int(v) => v.is_one(),
            Value::Void | Value::Time(_) | Value::Array(..) | Value::Struct(..) => false,
        }
    }
}

impl From<TimeValue> for Value {
    fn from(v: TimeValue) -> Value {
        Value::Time(v)
    }
}

impl From<IntValue> for Value {
    fn from(v: IntValue) -> Value {
        Value::Int(v)
    }
}

impl From<ArrayValue> for Value {
    fn from(v: ArrayValue) -> Value {
        Value::Array(v)
    }
}

impl From<StructValue> for Value {
    fn from(v: StructValue) -> Value {
        Value::Struct(v)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Time(v) => write!(f, "time {}", v),
            Value::Int(v) => Display::fmt(v, f),
            Value::Array(v) => Display::fmt(v, f),
            Value::Struct(v) => Display::fmt(v, f),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
