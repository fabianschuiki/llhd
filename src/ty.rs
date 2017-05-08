// Copyright (c) 2017 Fabian Schuiki

//! Types of values.

use std;
use std::sync::Arc;
pub use self::TypeKind::*;

pub type Type = Arc<TypeKind>;

#[derive(Debug)]
pub enum TypeKind {
	/// The `void` type.
	VoidType,
	/// Integer types like `i32`.
	IntType(usize),
	/// Pointer types like `i32*`.
	PointerType(Type),
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
			VectorType(l, ref ty) => write!(f, "<{} x {}>", l, ty),
			StructType(ref tys) => {
				write!(f, "{{")?;
				write_commas(f, tys)?;
				write!(f, "}}")?;
				Ok(())
			},
			FuncType(ref args, ref ret) => {
				write!(f, "(")?;
				write_commas(f, args)?;
				write!(f, ") {}", ret)?;
				Ok(())
			},
			EntityType(ref ins, ref outs) => {
				write!(f, "(")?;
				write_commas(f, ins)?;
				write!(f, ";")?;
				write_commas(f, outs)?;
				write!(f, ")")?;
				Ok(())
			},
		}
	}
}


impl TypeKind {
	/// Unwrap the type into arguments and return type, or panic if the type is
	/// not a function.
	pub fn unwrap_func(&self) -> (&[Type], &Type) {
		match *self {
			FuncType(ref args, ref ret) => (args, ret),
			_ => panic!("unwrap_func called on {}", self)
		}
	}
}


/// Formats a slice of elements that implement the `std::fmt::Display` trait as
/// a comma separated list.
fn write_commas<T: std::fmt::Display>(f: &mut std::fmt::Formatter, v: &[T]) -> std::fmt::Result {
	let mut it = v.iter();
	if let Some(x) = it.next() {
		write!(f, "{}", x)?;
	}
	for x in it {
		write!(f, ", {}", x)?;
	}
	Ok(())
}
