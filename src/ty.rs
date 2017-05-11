// Copyright (c) 2017 Fabian Schuiki

//! Types of values.

use std;
use std::sync::Arc;
pub use self::TypeKind::*;
use util::write_implode;

pub type Type = Arc<TypeKind>;

#[derive(Debug)]
pub enum TypeKind {
	/// The `void` type.
	VoidType,
	/// Integer types like `i32`.
	IntType(usize),
	/// Pointer types like `i32*`.
	PointerType(Type),
	/// Signal types like `i32$`.
	SignalType(Type),
	/// Vector types like `<4 x i32>`.
	VectorType(usize, Type),
	/// Struct types like `{i8, i32}`.
	StructType(Vec<Type>),
	/// Function types like `(i32) void`.
	FuncType(Vec<Type>, Type),
	/// Entity types like `(i8, i8; i32)`.
	EntityType(Vec<Type>, Vec<Type>),
}


impl std::fmt::Display for TypeKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			VoidType => write!(f, "void"),
			IntType(l) => write!(f, "i{}", l),
			PointerType(ref ty) => write!(f, "{}*", ty),
			SignalType(ref ty) => write!(f, "{}$", ty),
			VectorType(l, ref ty) => write!(f, "<{} x {}>", l, ty),
			StructType(ref tys) => {
				write!(f, "{{")?;
				write_implode(f, ", ", tys.iter())?;
				write!(f, "}}")?;
				Ok(())
			},
			FuncType(ref args, ref ret) => {
				write!(f, "(")?;
				write_implode(f, ", ", args.iter())?;
				write!(f, ") {}", ret)?;
				Ok(())
			},
			EntityType(ref ins, ref outs) => {
				write!(f, "(")?;
				write_implode(f, ", ", ins.iter())?;
				write!(f, ";")?;
				write_implode(f, ", ", outs.iter())?;
				write!(f, ")")?;
				Ok(())
			},
		}
	}
}


impl TypeKind {
	/// Unwrap the type into arguments and return type, or panic if the type is
	/// not a function.
	pub fn as_func(&self) -> (&[Type], &Type) {
		match *self {
			FuncType(ref args, ref ret) => (args, ret),
			_ => panic!("unwrap_func called on {}", self)
		}
	}
}
