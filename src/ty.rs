// Copyright (c) 2017-2021 Fabian Schuiki

//! Types of values.

use itertools::Itertools;
use std::sync::Arc;

pub use self::TypeKind::*;

/// An LLHD type.
pub type Type = Arc<TypeKind>;

/// The different kinds of types.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind {
    /// The `void` type.
    VoidType,
    /// The `time` type.
    TimeType,
    /// Integer types like `i32`.
    IntType(usize),
    /// Enumerated types like `n42`.
    EnumType(usize),
    /// Pointer types like `i32*`.
    PointerType(Type),
    /// Signal types like `i32$`.
    SignalType(Type),
    /// Array types like `[4 x i32]`.
    ArrayType(usize, Type),
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
            TimeType => write!(f, "time"),
            IntType(l) => write!(f, "i{}", l),
            EnumType(l) => write!(f, "n{}", l),
            PointerType(ref ty) => write!(f, "{}*", ty),
            SignalType(ref ty) => write!(f, "{}$", ty),
            ArrayType(l, ref ty) => write!(f, "[{} x {}]", l, ty),
            StructType(ref tys) => write!(f, "{{{}}}", tys.iter().format(", ")),
            FuncType(ref args, ref ret) => write!(f, "({}) {}", args.iter().format(", "), ret),
            EntityType(ref ins, ref outs) => write!(
                f,
                "({}) -> ({})",
                ins.iter().format(", "),
                outs.iter().format(", ")
            ),
        }
    }
}

impl TypeKind {
    /// Unwrap the type to its integer bit width, or panic if the type is not an
    /// integer.
    pub fn unwrap_int(&self) -> usize {
        match *self {
            IntType(size) => size,
            _ => panic!("unwrap_int called on {}", self),
        }
    }

    /// Unwrap the type to its number of enumerated states, or panic if the type
    /// is not an enum.
    pub fn unwrap_enum(&self) -> usize {
        match *self {
            EnumType(size) => size,
            _ => panic!("unwrap_enum called on {}", self),
        }
    }

    /// Unwrap the type to its pointer data type, or panic if the type is not a
    /// pointer. E.g. yields the `i8` type in `i8*`.
    pub fn unwrap_pointer(&self) -> &Type {
        match *self {
            PointerType(ref ty) => ty,
            _ => panic!("unwrap_pointer called on {}", self),
        }
    }

    /// Unwrap the type to its signal data type, or panic if the type is not an
    /// integer. E.g. yields the `i8` type in `i8$`.
    pub fn unwrap_signal(&self) -> &Type {
        match *self {
            SignalType(ref ty) => ty,
            _ => panic!("unwrap_signal called on {}", self),
        }
    }

    /// Unwrap the type to its array length and element type, or panic if the
    /// type is not an array. E.g. yields the `(16, i32)` in `[16 x i32]`.
    pub fn unwrap_array(&self) -> (usize, &Type) {
        match *self {
            ArrayType(len, ref ty) => (len, ty),
            _ => panic!("unwrap_array called on {}", self),
        }
    }

    /// Unwrap the type to its struct fields, or panic if the type is not a
    /// struct. E.g. yields the `[i8, i16]` in `{i8, i16}`.
    pub fn unwrap_struct(&self) -> &[Type] {
        match *self {
            StructType(ref fields) => fields,
            _ => panic!("unwrap_struct called on {}", self),
        }
    }

    /// Unwrap the type into arguments and return type, or panic if the type is
    /// not a function.
    pub fn unwrap_func(&self) -> (&[Type], &Type) {
        match *self {
            FuncType(ref args, ref ret) => (args, ret),
            _ => panic!("unwrap_func called on {}", self),
        }
    }

    /// Unwrap the type into input and output arguments, or panic if the type is
    /// not an entity.
    pub fn unwrap_entity(&self) -> (&[Type], &[Type]) {
        match *self {
            EntityType(ref ins, ref outs) => (ins, outs),
            _ => panic!("unwrap_entity called on {}", self),
        }
    }

    /// Check if this is a void type.
    pub fn is_void(&self) -> bool {
        match *self {
            VoidType => true,
            _ => false,
        }
    }

    /// Check if this is a time type.
    pub fn is_time(&self) -> bool {
        match *self {
            TimeType => true,
            _ => false,
        }
    }

    /// Check if this is an integer type.
    pub fn is_int(&self) -> bool {
        match *self {
            IntType(..) => true,
            _ => false,
        }
    }

    /// Check if this is an enum type.
    pub fn is_enum(&self) -> bool {
        match *self {
            EnumType(..) => true,
            _ => false,
        }
    }

    /// Check if this is a pointer type.
    pub fn is_pointer(&self) -> bool {
        match *self {
            PointerType(..) => true,
            _ => false,
        }
    }

    /// Check if this is a signal type.
    pub fn is_signal(&self) -> bool {
        match *self {
            SignalType(..) => true,
            _ => false,
        }
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        match *self {
            ArrayType(..) => true,
            _ => false,
        }
    }

    /// Check if this is a struct type.
    pub fn is_struct(&self) -> bool {
        match *self {
            StructType(..) => true,
            _ => false,
        }
    }

    /// Check if this is a func type.
    pub fn is_func(&self) -> bool {
        match *self {
            FuncType(..) => true,
            _ => false,
        }
    }

    /// Check if this is an entity type.
    pub fn is_entity(&self) -> bool {
        match *self {
            EntityType(..) => true,
            _ => false,
        }
    }

    /// Extract the length of the type.
    ///
    /// This is the number of:
    /// - bits in an integer
    /// - states in an enum
    /// - elements in an array
    /// - fields in a struct
    ///
    /// Returns zero for all other types.
    pub fn len(&self) -> usize {
        match *self {
            IntType(l) | EnumType(l) | ArrayType(l, _) => l,
            StructType(ref f) => f.len(),
            _ => 0,
        }
    }
}

/// Create a void type.
pub fn void_ty() -> Type {
    Type::new(VoidType)
}

/// Create a time type.
pub fn time_ty() -> Type {
    Type::new(TimeType)
}

/// Create an integer type of the requested size.
pub fn int_ty(size: usize) -> Type {
    Type::new(IntType(size))
}

/// Create an enum type of the requested size.
pub fn enum_ty(size: usize) -> Type {
    Type::new(EnumType(size))
}

/// Create a pointer type with the requested data type.
pub fn pointer_ty(ty: Type) -> Type {
    Type::new(PointerType(ty))
}

/// Create a signal type with the requested data type.
pub fn signal_ty(ty: Type) -> Type {
    Type::new(SignalType(ty))
}

/// Create a array type. `size` is the number of elements in the array, and `ty`
/// the type of each individual element.
pub fn array_ty(size: usize, ty: Type) -> Type {
    Type::new(ArrayType(size, ty))
}

/// Create a struct type. `fields` is an list of types, one for each field.
pub fn struct_ty(fields: Vec<Type>) -> Type {
    Type::new(StructType(fields))
}

/// Create a function type with the given arguments and return type.
pub fn func_ty(args: Vec<Type>, ret: Type) -> Type {
    Type::new(FuncType(args, ret))
}

/// Create an entity type with the given input and output arguments.
pub fn entity_ty(ins: Vec<Type>, outs: Vec<Type>) -> Type {
    Type::new(EntityType(ins, outs))
}
