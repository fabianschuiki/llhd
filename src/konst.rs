// Copyright (c) 2017 Fabian Schuiki

//! This module implements constant values.

use num::BigInt;
use std::sync::Arc;
use value::*;
use ty::*;

pub type Const = Arc<ConstKind>;

impl Into<ValueRef> for Const {
	fn into(self) -> ValueRef {
		ValueRef::Const(self)
	}
}

#[derive(Debug)]
pub enum ConstKind {
	Int(ConstInt),
}

impl Value for ConstKind {
	fn id(&self) -> ValueId {
		INLINE_VALUE_ID
	}

	fn ty(&self) -> Type {
		match *self {
			ConstKind::Int(ref k) => int_ty(k.width()),
		}
	}

	fn name(&self) -> Option<&str> {
		None
	}
}


/// A constant integer value.
#[derive(Debug)]
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


/// Create a new integer constant.
pub fn const_int(width: usize, value: BigInt) -> Const {
	Const::new(ConstKind::Int(ConstInt::new(width, value)))
}

/// Create a constant zero value of the requested type. Panics if there is no
/// zero value for the given type.
pub fn const_zero(ty: &Type) -> Const {
	use num::Zero;
	match **ty {
		IntType(sz) => const_int(sz, BigInt::zero()),
		ref x => panic!("no const zero value for type {}", x),
	}
}
