// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code)]

use std::collections::HashMap;
use ty::*;
use value::*;
pub use self::InstKind::*;

pub struct Inst {
	id: InstRef,
	/// An optional name for the instruction used when emitting assembly.
	name: Option<String>,
	/// The instruction data.
	kind: InstKind,
}

impl Inst {
	/// Create a new instruction.
	pub fn new<T: Into<String>>(name: Option<T>, kind: InstKind) -> Inst {
		Inst {
			id: InstRef(ValueId::alloc()),
			name: name.map(|x| x.into()),
			kind: kind,
		}
	}

	/// Obtain a reference to this instruction.
	pub fn as_ref(&self) -> InstRef {
		self.id
	}
}

impl Value for Inst {
	fn id(&self) -> ValueId {
		self.id.into()
	}

	fn ty(&self) -> Type {
		self.kind.ty()
	}

	fn name(&self) -> Option<&str> {
		self.name.as_ref().map(|x| x as &str)
	}
}

declare_ref!(InstRef, Inst);


pub enum InstKind {
	BinaryInst(BinaryOp, Type, ValueRef, ValueRef),
}

impl InstKind {
	/// Get the result type of the instruction.
	pub fn ty(&self) -> Type {
		match *self {
			BinaryInst(_, ref ty, _, _) => ty.clone(),
		}
	}
}

pub enum BinaryOp {
	Add,
}

pub struct InstPool(HashMap<InstRef, Inst>);
