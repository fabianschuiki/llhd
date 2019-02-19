// Copyright (c) 2017 Fabian Schuiki

//! This module implements constant values.

use crate::{aggregate::*, ty::*, value::*};
use num::{BigInt, BigRational};
use std;
use std::sync::Arc;

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
    pub fn unwrap_int(&self) -> &ConstInt {
        match *self {
            ConstKind::Int(ref k) => k,
            _ => panic!("unwrap_int called on {}", self.desc()),
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
    delta: usize,
    epsilon: usize,
}

impl ConstTime {
    /// Create a new constant time.
    pub fn new(time: BigRational, delta: usize, epsilon: usize) -> ConstTime {
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
    pub fn delta(&self) -> usize {
        self.delta
    }

    /// Get the epsilon time of the constant.
    pub fn epsilon(&self) -> usize {
        self.epsilon
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
        write_ratio_as_si(&self.time, f)?;
        if !self.delta.is_zero() {
            write!(f, " {}d", self.delta)?;
        }
        if !self.epsilon.is_zero() {
            write!(f, " {}e", self.epsilon)?;
        }
        Ok(())
    }
}

fn write_ratio_as_si(ratio: &BigRational, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use num::{One, Zero};
    if ratio.is_zero() {
        return write!(f, "0s");
    }
    let prefices = ["", "m", "u", "n", "p", "f", "a"];
    let mut scaled = ratio.clone();
    let mut prefix = 0;
    let mut shift = 0;
    while prefix + 1 < prefices.len() && shift < 9 {
        if scaled >= One::one() {
            if scaled.is_integer() {
                break;
            } else {
                shift += 3;
            }
        } else {
            prefix += 1;
        }
        scaled = scaled * BigRational::from_integer(BigInt::from(1000));
    }
    let rounded = format!("{}", scaled.round());
    if shift > 0 {
        write!(
            f,
            "{}.{}{}s",
            &rounded[0..3],
            &rounded[3..],
            prefices[prefix]
        )?;
    } else {
        write!(f, "{}{}s", rounded, prefices[prefix])?;
    }
    Ok(())
}

/// Create a new integer constant.
pub fn const_int(width: usize, value: BigInt) -> Const {
    Const::new(ConstKind::Int(ConstInt::new(width, value)))
}

/// Create a new time constant.
pub fn const_time(time: BigRational, delta: usize, epsilon: usize) -> Const {
    Const::new(ConstKind::Time(ConstTime::new(time, delta, epsilon)))
}

/// Create a new array constant.
pub fn const_array(element_ty: Type, elements: Vec<ValueRef>) -> ValueRef {
    let ty = array_ty(elements.len(), element_ty);
    Aggregate::new(ArrayAggregate::new(ty, elements).into()).into()
}

/// Create a new array constant where each element has the same value.
pub fn const_uniform_array(length: usize, element_value: ValueRef) -> ValueRef {
    let ty = array_ty(
        length,
        match element_value {
            ValueRef::Const(ref k) => k.ty(),
            ValueRef::Aggregate(ref a) => a.ty(),
            _ => panic!(
                "const_uniform_array from non-const/non-aggregate element value {:?}",
                element_value
            ),
        },
    );
    Aggregate::new(
        ArrayAggregate::new(ty, std::iter::repeat(element_value).take(length).collect()).into(),
    )
    .into()
}

/// Create a new struct constant.
pub fn const_struct(fields: Vec<ValueRef>) -> ValueRef {
    let field_tys = fields
        .iter()
        .map(|field| match *field {
            ValueRef::Const(ref k) => k.ty(),
            ValueRef::Aggregate(ref a) => a.ty(),
            _ => panic!(
                "const_struct from non-const/non-aggregate field value {:?}",
                field
            ),
        })
        .collect();
    let ty = struct_ty(field_tys);
    Aggregate::new(StructAggregate::new(ty, fields).into()).into()
}

/// Create a constant zero value of the requested type. Panics if there is no
/// zero value for the given type.
pub fn const_zero(ty: &Type) -> ValueRef {
    use num::Zero;
    match **ty {
        IntType(sz) => const_int(sz, BigInt::zero()).into(),
        TimeType => const_time(BigRational::zero(), 0, 0).into(),
        ArrayType(sz, ref elem_ty) => const_uniform_array(sz, const_zero(elem_ty)),
        StructType(ref field_tys) => const_struct(field_tys.iter().map(const_zero).collect()),
        ref x => panic!("no const zero value for type {}", x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_formatting() {
        let make = |num: usize, denom: usize, delta: usize, epsilon: usize| {
            format!(
                "{}",
                ConstTime::new(
                    BigRational::new(num.into(), denom.into()),
                    delta.into(),
                    epsilon.into()
                )
            )
        };
        assert_eq!(make(0, 1, 0, 0), "0s");
        assert_eq!(make(0, 1, 0, 1), "0s 1e");
        assert_eq!(make(0, 1, 1, 0), "0s 1d");
        assert_eq!(make(0, 1, 1, 1), "0s 1d 1e");

        assert_eq!(make(1, 1, 0, 0), "1s");
        assert_eq!(make(1, 1, 0, 1), "1s 1e");
        assert_eq!(make(1, 1, 1, 0), "1s 1d");
        assert_eq!(make(1, 1, 1, 1), "1s 1d 1e");

        assert_eq!(make(1, 10, 0, 0), "100ms");
        assert_eq!(make(1, 100, 0, 0), "10ms");
        assert_eq!(make(1, 1000, 0, 0), "1ms");
        assert_eq!(make(1, 10000, 0, 0), "100us");
        assert_eq!(make(1, 100000, 0, 0), "10us");
        assert_eq!(make(1, 1000000, 0, 0), "1us");
        assert_eq!(make(1, 10000000, 0, 0), "100ns");
        assert_eq!(make(1, 100000000, 0, 0), "10ns");
        assert_eq!(make(1, 1000000000, 0, 0), "1ns");
        assert_eq!(make(1, 10000000000, 0, 0), "100ps");
        assert_eq!(make(1, 100000000000, 0, 0), "10ps");
        assert_eq!(make(1, 1000000000000, 0, 0), "1ps");
        assert_eq!(make(1, 10000000000000, 0, 0), "100fs");
        assert_eq!(make(1, 100000000000000, 0, 0), "10fs");
        assert_eq!(make(1, 1000000000000000, 0, 0), "1fs");
        assert_eq!(make(1, 10000000000000000, 0, 0), "100as");
        assert_eq!(make(1, 100000000000000000, 0, 0), "10as");
        assert_eq!(make(1, 1000000000000000000, 0, 0), "1as");

        assert_eq!(make(500, 1, 0, 0), "500s");
        assert_eq!(make(50, 1, 0, 0), "50s");
        assert_eq!(make(5, 1, 0, 0), "5s");
        assert_eq!(make(5, 10, 0, 0), "500ms");
        assert_eq!(make(5, 100, 0, 0), "50ms");
        assert_eq!(make(5, 1000, 0, 0), "5ms");

        assert_eq!(make(1, 3, 0, 0), "333.333333333ms");
    }
}
