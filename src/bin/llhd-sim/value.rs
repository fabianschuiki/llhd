// Copyright (c) 2017-2021 Fabian Schuiki

//! Value computation
//!
//! This module implements representations for LLHD values as they evolve during
//! the simulation of a design.

use llhd::ir::Opcode;
use num::{bigint::ToBigInt, BigInt, BigUint, One, Signed, ToPrimitive, Zero};
use std::fmt::{Debug, Display};

/// A point in time.
pub type Time = llhd::value::TimeValue;

/// A point in time.
pub type TimeValue = llhd::value::TimeValue;

/// A value.
#[derive(Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Value {
    Void,
    Time(Time),
    Int(IntValue),
    Array(ArrayValue),
    Struct(StructValue),
}

impl Value {
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

    /// Check if the value is zero.
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Array(..) | Value::Struct(..) | Value::Void => false,
            Value::Time(v) => v.time().is_zero() && v.delta() == 0 && v.epsilon() == 0,
            Value::Int(v) => v.is_zero(),
        }
    }

    /// Check if the value is one.
    pub fn is_one(&self) -> bool {
        match self {
            Value::Array(..) | Value::Struct(..) | Value::Void => false,
            Value::Time(_) => false,
            Value::Int(v) => v.is_one(),
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

/// An integer value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntValue {
    /// The width of the value in bits.
    pub width: usize,
    /// The value itself.
    pub value: BigUint,
}

impl IntValue {
    /// Create a new integer value from a `usize`.
    pub fn from_usize(width: usize, value: usize) -> Self {
        Self {
            width,
            value: value.into(),
        }
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
            (BigInt::one() << self.width) - self.value.to_bigint().unwrap()
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
        IntValue::from_signed(self.width, self.to_signed() % other.to_signed())
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
    pub fn unary_op(op: Opcode, arg: &IntValue) -> IntValue {
        // trace!("{} ({}, {})", op, lhs, rhs);
        match op {
            Opcode::Not => arg.not(),
            Opcode::Neg => arg.neg(),
            _ => panic!("{} is not a unary op", op),
        }
    }

    /// Execute a binary opcode.
    pub fn binary_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> IntValue {
        trace!("{} ({}, {})", op, lhs, rhs);
        match op {
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
            _ => panic!("{} is not a binary op", op),
        }
    }

    /// Execute a comparison opcode.
    pub fn compare_op(op: Opcode, lhs: &IntValue, rhs: &IntValue) -> IntValue {
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
            _ => panic!("{} is not a compare op", op),
        };
        IntValue::from_usize(1, v as usize)
    }
}

/// An array value.
#[derive(Clone, PartialEq, Eq)]
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

/// A struct value.
#[derive(Clone, PartialEq, Eq)]
pub struct StructValue(pub Vec<Value>);

impl StructValue {
    /// Create a new struct.
    pub fn new(values: Vec<Value>) -> Self {
        StructValue(values)
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
