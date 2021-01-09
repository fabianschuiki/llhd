// Copyright (c) 2017-2021 Fabian Schuiki

//! Integer values
//!
//! This module implements integer value arithmetic.

use crate::ir::prelude::*;
use crate::ty::{int_ty, Type};
use num::{bigint::ToBigInt, traits::*, BigInt, BigUint};
use std::fmt::{Debug, Display};

/// An integer value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IntValue {
    /// The width of the value in bits.
    pub width: usize,
    /// The value itself.
    pub value: BigUint,
}

impl IntValue {
    /// Create a zero value.
    pub fn zero(width: usize) -> Self {
        Self {
            width,
            value: BigUint::zero(),
        }
    }
    /// Create a value with all bits set to one.
    pub fn all_ones(width: usize) -> Self {
        Self {
            width,
            value: (BigUint::one() << width) - 1usize,
        }
    }

    /// Create a new integer value from a `usize`.
    pub fn from_usize(width: usize, value: usize) -> Self {
        Self {
            width,
            value: value.into(),
        }
    }

    /// Create a new integer value from an `isize`.
    pub fn from_isize(width: usize, value: isize) -> Self {
        Self::from_signed(width, value.into())
    }

    /// Create a new integer value from a signed `BigInt` value.
    pub fn from_signed(width: usize, value: BigInt) -> Self {
        let modulus = BigInt::one() << width;
        let mut v = value % &modulus;
        if v.is_negative() {
            v += modulus;
        }
        assert!(!v.is_negative());
        Self::from_unsigned(width, v.to_biguint().unwrap())
    }

    /// Create a new integer value from an unsigned `BigUint` value.
    pub fn from_unsigned(width: usize, value: BigUint) -> Self {
        let value = value % (BigUint::one() << width);
        Self { width, value }
    }

    /// Convert the value to a signed `BigInt`.
    pub fn to_signed(&self) -> BigInt {
        let sign_mask = BigUint::one() << (self.width - 1);
        if (&self.value & &sign_mask).is_zero() {
            self.value.to_bigint().unwrap()
        } else {
            self.value.to_bigint().unwrap() - (BigInt::one() << self.width)
        }
    }

    /// Convert the value to a usize.
    pub fn to_usize(&self) -> usize {
        self.value.to_usize().unwrap()
    }

    /// Check if the value is zero.
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    /// Check if the value is one.
    pub fn is_one(&self) -> bool {
        self.value.is_one()
    }

    /// Check if the value has every bit set to one.
    pub fn is_all_ones(&self) -> bool {
        self.value == Self::all_ones(self.width).value
    }

    /// Get the type of the value.
    pub fn ty(&self) -> Type {
        int_ty(self.width)
    }
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "i{} {}", self.width, self.value)
    }
}

impl Debug for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<(usize, usize)> for IntValue {
    fn from(v: (usize, usize)) -> Self {
        IntValue::from_usize(v.0, v.1)
    }
}

impl From<(usize, BigInt)> for IntValue {
    fn from(v: (usize, BigInt)) -> Self {
        IntValue::from_signed(v.0, v.1)
    }
}

impl From<(usize, BigUint)> for IntValue {
    fn from(v: (usize, BigUint)) -> Self {
        IntValue::from_unsigned(v.0, v.1)
    }
}

/// Slicing.
impl IntValue {
    /// Extract a slice of bits from the value.
    pub fn extract_slice(&self, off: usize, len: usize) -> IntValue {
        let shifted = self.value.clone() >> off;
        let modulus = BigUint::one() << len;
        IntValue::from_unsigned(len, shifted % modulus)
    }

    /// Insert a slice of bits into the value.
    pub fn insert_slice(&mut self, off: usize, len: usize, value: &IntValue) {
        assert_eq!(len, value.width);
        let mask = ((BigUint::one() << len) - BigUint::one()) << off;
        let mask_inv = ((BigUint::one() << self.width) - BigUint::one()) ^ mask;
        self.value &= mask_inv;
        self.value |= &value.value << off;
    }
}

/// Unary operators.
impl IntValue {
    /// Compute `not`.
    pub fn not(&self) -> IntValue {
        let max = (BigUint::one() << self.width) - BigUint::one();
        let v = &max - &self.value;
        IntValue::from_unsigned(self.width, v)
    }

    /// Compute `neg`.
    pub fn neg(&self) -> IntValue {
        let max = BigUint::one() << self.width;
        let v = &max - &self.value;
        IntValue::from_unsigned(self.width, v)
    }
}

/// Binary operators.
impl IntValue {
    /// Compute `add`.
    pub fn add(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value + &other.value)
    }

    /// Compute `sub`.
    pub fn sub(&self, other: &Self) -> IntValue {
        IntValue::from_signed(self.width, self.to_signed() - other.to_signed())
    }

    /// Compute `and`.
    pub fn and(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value & &other.value)
    }

    /// Compute `or`.
    pub fn or(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value | &other.value)
    }

    /// Compute `xor`.
    pub fn xor(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value ^ &other.value)
    }

    /// Compute `umul`.
    pub fn umul(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value * &other.value)
    }

    /// Compute `udiv`.
    pub fn udiv(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value / &other.value)
    }

    /// Compute `umod`.
    pub fn umod(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value % &other.value)
    }

    /// Compute `urem`.
    pub fn urem(&self, other: &Self) -> IntValue {
        IntValue::from_unsigned(self.width, &self.value % &other.value)
    }

    /// Compute `smul`.
    pub fn smul(&self, other: &Self) -> IntValue {
        IntValue::from_signed(self.width, self.to_signed() * other.to_signed())
    }

    /// Compute `sdiv`.
    pub fn sdiv(&self, other: &Self) -> IntValue {
        IntValue::from_signed(self.width, self.to_signed() / other.to_signed())
    }

    /// Compute `smod`.
    pub fn smod(&self, other: &Self) -> IntValue {
        let a = self.to_signed();
        let b = other.to_signed();
        let mut r = &a % &b;
        if !r.is_zero() && a.is_negative() != b.is_negative() {
            r += b;
        }
        IntValue::from_signed(self.width, r)
    }

    /// Compute `srem`.
    pub fn srem(&self, other: &Self) -> IntValue {
        IntValue::from_signed(self.width, self.to_signed() % other.to_signed())
    }
}

/// Comparisons.
impl IntValue {
    /// Compute `==`.
    pub fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value == other.value
    }

    /// Compute `!=`.
    pub fn neq(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value != other.value
    }

    /// Compute unsigned `<`.
    pub fn ult(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value < other.value
    }

    /// Compute unsigned `>`.
    pub fn ugt(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value > other.value
    }

    /// Compute unsigned `<=`.
    pub fn ule(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value <= other.value
    }

    /// Compute unsigned `>=`.
    pub fn uge(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.value >= other.value
    }

    /// Compute signed `<`.
    pub fn slt(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.to_signed() < other.to_signed()
    }

    /// Compute signed `>`.
    pub fn sgt(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.to_signed() > other.to_signed()
    }

    /// Compute signed `<=`.
    pub fn sle(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.to_signed() <= other.to_signed()
    }

    /// Compute signed `>=`.
    pub fn sge(&self, other: &Self) -> bool {
        assert_eq!(self.width, other.width);
        self.to_signed() >= other.to_signed()
    }
}

/// Opcode implementations.
impl IntValue {
    /// Execute a unary opcode.
    pub fn try_unary_op(op: Opcode, arg: &IntValue) -> Option<IntValue> {
        // trace!("{} ({}, {})", op, lhs, rhs);
        Some(match op {
            Opcode::Not => arg.not(),
            Opcode::Neg => arg.neg(),
            _ => return None,
        })
    }

    /// Execute a unary opcode.
    pub fn unary_op(op: Opcode, arg: &IntValue) -> IntValue {
        match Self::try_unary_op(op, arg) {
            Some(r) => r,
            None => panic!("{} is not a unary op", op),
        }
    }

    /// Execute a binary opcode.
    pub fn try_binary_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> Option<IntValue> {
        // trace!("{} ({}, {})", op, lhs, rhs);
        Some(match op {
            Opcode::Add => lhs.add(rhs),
            Opcode::Sub => lhs.sub(rhs),
            Opcode::And => lhs.and(rhs),
            Opcode::Or => lhs.or(rhs),
            Opcode::Xor => lhs.xor(rhs),
            Opcode::Smul => lhs.smul(rhs),
            Opcode::Sdiv => lhs.sdiv(rhs),
            Opcode::Smod => lhs.smod(rhs),
            Opcode::Srem => lhs.srem(rhs),
            Opcode::Umul => lhs.umul(rhs),
            Opcode::Udiv => lhs.udiv(rhs),
            Opcode::Umod => lhs.umod(rhs),
            Opcode::Urem => lhs.urem(rhs),
            _ => return None,
        })
    }

    /// Execute a binary opcode.
    pub fn binary_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> IntValue {
        match Self::try_binary_op(op, lhs, rhs) {
            Some(r) => r,
            None => panic!("{} is not a binary op", op),
        }
    }

    /// Execute a comparison opcode.
    pub fn try_compare_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> Option<IntValue> {
        // trace!("{} ({}, {})", op, lhs, rhs);
        let v = match op {
            Opcode::Eq => lhs.eq(rhs),
            Opcode::Neq => lhs.neq(rhs),
            Opcode::Ult => lhs.ult(rhs),
            Opcode::Ugt => lhs.ugt(rhs),
            Opcode::Ule => lhs.ule(rhs),
            Opcode::Uge => lhs.uge(rhs),
            Opcode::Slt => lhs.slt(rhs),
            Opcode::Sgt => lhs.sgt(rhs),
            Opcode::Sle => lhs.sle(rhs),
            Opcode::Sge => lhs.sge(rhs),
            _ => return None,
        };
        Some(IntValue::from_usize(1, v as usize))
    }

    /// Execute a comparison opcode.
    pub fn compare_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> IntValue {
        match Self::try_compare_op(op, lhs, rhs) {
            Some(r) => r,
            None => panic!("{} is not a compare op", op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign() {
        assert_eq!(IntValue::from_isize(8, 9), IntValue::from_usize(8, 9));
        assert_eq!(IntValue::from_isize(8, -9), IntValue::from_usize(8, 247));
        assert_eq!(IntValue::from_isize(8, 9).to_signed(), BigInt::from(9));
        assert_eq!(IntValue::from_isize(8, -9).to_signed(), BigInt::from(-9));
    }

    #[test]
    fn add() {
        let a = IntValue::from_usize(8, 7);
        let b = IntValue::from_usize(8, 2);
        let an = a.neg();
        let bn = b.neg();

        assert_eq!(a.add(&b), IntValue::from_usize(8, 9));
        assert_eq!(a.add(&bn), IntValue::from_usize(8, 5));
        assert_eq!(an.add(&b), IntValue::from_isize(8, -5));
        assert_eq!(an.add(&bn), IntValue::from_isize(8, -9));
    }

    #[test]
    fn sub() {
        let a = IntValue::from_usize(8, 7);
        let b = IntValue::from_usize(8, 2);
        let an = a.neg();
        let bn = b.neg();

        assert_eq!(a.sub(&b), IntValue::from_usize(8, 5));
        assert_eq!(a.sub(&bn), IntValue::from_usize(8, 9));
        assert_eq!(an.sub(&b), IntValue::from_isize(8, -9));
        assert_eq!(an.sub(&bn), IntValue::from_isize(8, -5));
    }

    #[test]
    fn smod() {
        let a = IntValue::from_usize(8, 9);
        let b = IntValue::from_usize(8, 5);
        let c = IntValue::from_usize(8, 10);
        let an = a.neg();
        let bn = b.neg();
        let cn = c.neg();

        assert_eq!(a.smod(&b), IntValue::from_isize(8, 4));
        assert_eq!(a.smod(&bn), IntValue::from_isize(8, -1));
        assert_eq!(an.smod(&b), IntValue::from_isize(8, 1));
        assert_eq!(an.smod(&bn), IntValue::from_isize(8, -4));
        assert_eq!(c.smod(&b), IntValue::from_isize(8, 0));
        assert_eq!(c.smod(&bn), IntValue::from_isize(8, 0));
        assert_eq!(cn.smod(&b), IntValue::from_isize(8, 0));
        assert_eq!(cn.smod(&bn), IntValue::from_isize(8, 0));
    }

    #[test]
    fn srem() {
        let a = IntValue::from_usize(8, 9);
        let b = IntValue::from_usize(8, 5);
        let c = IntValue::from_usize(8, 10);
        let an = a.neg();
        let bn = b.neg();
        let cn = c.neg();

        assert_eq!(a.srem(&b), IntValue::from_isize(8, 4));
        assert_eq!(a.srem(&bn), IntValue::from_isize(8, 4));
        assert_eq!(an.srem(&b), IntValue::from_isize(8, -4));
        assert_eq!(an.srem(&bn), IntValue::from_isize(8, -4));
        assert_eq!(c.srem(&b), IntValue::from_isize(8, 0));
        assert_eq!(c.srem(&bn), IntValue::from_isize(8, 0));
        assert_eq!(cn.srem(&b), IntValue::from_isize(8, 0));
        assert_eq!(cn.srem(&bn), IntValue::from_isize(8, 0));
    }
}
