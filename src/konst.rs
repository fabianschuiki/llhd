// Copyright (c) 2017 Fabian Schuiki

//! This module implements constant values.

use num::{BigInt, BigRational};
use std;
use std::sync::Arc;
use ty::*;
use value::*;

pub type Const = Arc<ConstKind>;

impl Into<ValueRef> for Const {
    fn into(self) -> ValueRef {
        ValueRef::Const(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstKind {
    Int(ConstInt),
    Time(ConstTime),
}

impl Value for ConstKind {
    fn id(&self) -> ValueId {
        INLINE_VALUE_ID
    }

    fn ty(&self) -> Type {
        match *self {
            ConstKind::Int(ref k) => int_ty(k.width()),
            ConstKind::Time(_) => time_ty(),
        }
    }

    fn name(&self) -> Option<&str> {
        None
    }
}

impl ConstKind {
    /// Return a static string describing the nature of the value reference.
    fn desc(&self) -> &'static str {
        match *self {
            ConstKind::Int(_) => "ConstKind::Int",
            ConstKind::Time(_) => "ConstKind::Time",
        }
    }

    /// Yield a reference to this constant's embedded integer. Panics if the
    /// constant is not an integer.
    pub fn as_int(&self) -> &ConstInt {
        match *self {
            ConstKind::Int(ref k) => k,
            _ => panic!("as_int called on {}", self.desc()),
        }
    }

    /// Yield a reference to this constant's embedded time. Panics if the
    /// constant is not a time.
    pub fn as_time(&self) -> &ConstTime {
        match *self {
            ConstKind::Time(ref k) => k,
            _ => panic!("as_time called on {}", self.desc()),
        }
    }
}

/// A constant integer value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstInt {
    width: usize,
    value: BigInt,
}

impl ConstInt {
    /// Create a new constant integer.
    pub fn new(width: usize, value: BigInt) -> ConstInt {
        ConstInt {
            width: width,
            value: value,
        }
    }

    /// Get the width of the constant in bits.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the value of the constant.
    pub fn value(&self) -> &BigInt {
        &self.value
    }
}

/// A constant time value.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct ConstTime {
    time: BigRational,
    delta: BigInt,
    epsilon: BigInt,
}

impl ConstTime {
    /// Create a new constant time.
    pub fn new(time: BigRational, delta: BigInt, epsilon: BigInt) -> ConstTime {
        ConstTime {
            time: time,
            delta: delta,
            epsilon: epsilon,
        }
    }

    /// Get the physical time of the constant.
    pub fn time(&self) -> &BigRational {
        &self.time
    }

    /// Get the delta time of the constant.
    pub fn delta(&self) -> &BigInt {
        &self.delta
    }

    /// Get the epsilon time of the constant.
    pub fn epsilon(&self) -> &BigInt {
        &self.epsilon
    }

    /// Check whether all components of this time constant are zero.
    pub fn is_zero(&self) -> bool {
        use num::Zero;
        self.time.is_zero() && self.delta.is_zero() && self.epsilon.is_zero()
    }
}

impl std::fmt::Debug for ConstTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ConstTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use num::Zero;
        let mut any = false;
        if !self.time.is_zero() || self.is_zero() {
            write!(f, "{}s", self.time)?;
            any = true;
        }
        if !self.delta.is_zero() {
            if any {
                write!(f, " ")?
            };
            write!(f, "{}d", self.delta)?;
            any = true;
        }
        if !self.epsilon.is_zero() {
            if any {
                write!(f, " ")?
            };
            write!(f, "{}e", self.epsilon)?;
        }
        Ok(())
    }
}

/// Create a new integer constant.
pub fn const_int(width: usize, value: BigInt) -> Const {
    Const::new(ConstKind::Int(ConstInt::new(width, value)))
}

/// Create a new time constant.
pub fn const_time(time: BigRational, delta: BigInt, epsilon: BigInt) -> Const {
    Const::new(ConstKind::Time(ConstTime::new(time, delta, epsilon)))
}

/// Create a constant zero value of the requested type. Panics if there is no
/// zero value for the given type.
pub fn const_zero(ty: &Type) -> Const {
    use num::Zero;
    match **ty {
        IntType(sz) => const_int(sz, BigInt::zero()),
        TimeType => const_time(BigRational::zero(), BigInt::zero(), BigInt::zero()),
        ref x => panic!("no const zero value for type {}", x),
    }
}
