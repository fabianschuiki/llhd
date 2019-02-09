// Copyright (c) 2017 Fabian Schuiki
#![allow(dead_code)]

pub use self::InstKind::*;
use crate::{ty::*, unit::UnitContext, value::*};

#[derive(Debug)]
pub struct Inst {
    id: InstRef,
    /// An optional name for the instruction used when emitting assembly.
    name: Option<String>,
    /// The instruction data.
    kind: InstKind,
}

impl Inst {
    /// Create a new instruction.
    pub fn new(name: Option<String>, kind: InstKind) -> Inst {
        Inst {
            id: InstRef::new(ValueId::alloc()),
            name: name,
            kind: kind,
        }
    }

    /// Obtain a reference to this instruction.
    pub fn as_ref(&self) -> InstRef {
        self.id
    }

    /// Determine the mnemonic for this instruction. The mnemonic is a short
    /// sequence of characters that uniquely identifies the instruction in human
    /// readable assembly text.
    pub fn mnemonic(&self) -> Mnemonic {
        self.kind.mnemonic()
    }

    /// Obtain a reference to the data for this instruction. See `InstKind`.
    pub fn kind(&self) -> &InstKind {
        &self.kind
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

    fn is_global(&self) -> bool {
        false
    }
}

pub struct InstIter<'tf> {
    refs: std::slice::Iter<'tf, InstRef>,
    ctx: &'tf UnitContext,
}

impl<'tf> InstIter<'tf> {
    pub fn new(refs: std::slice::Iter<'tf, InstRef>, ctx: &'tf UnitContext) -> InstIter<'tf> {
        InstIter {
            refs: refs,
            ctx: ctx,
        }
    }
}

impl<'tf> std::iter::Iterator for InstIter<'tf> {
    type Item = &'tf Inst;

    fn next(&mut self) -> Option<&'tf Inst> {
        let n = self.refs.next();
        n.map(|r| self.ctx.inst(*r))
    }
}

/// A relative position of an instruction. Used to insert or move an instruction
/// to a position relative to the surrounding unit, block, or another
/// instruction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InstPosition {
    /// The very first position in the entity, or the first position in the
    /// first block of the function/process.
    Begin,
    /// The very last position in the entity, or the last position in the last
    /// block of the function/process.
    End,
    /// The position just before another instruction.
    Before(InstRef),
    /// The position just after another instruction.
    After(InstRef),
    /// The very first position in the block. Only valid in functions and
    /// processes.
    BlockBegin(BlockRef),
    /// The very last position in the block. Only valid in functions and
    /// processes.
    BlockEnd(BlockRef),
}

/// The different forms an instruction can take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstKind {
    UnaryInst(UnaryOp, Type, ValueRef),
    BinaryInst(BinaryOp, Type, ValueRef, ValueRef),
    CompareInst(CompareOp, Type, ValueRef, ValueRef),
    CallInst(Type, ValueRef, Vec<ValueRef>),
    InstanceInst(Type, ValueRef, Vec<ValueRef>, Vec<ValueRef>),
    WaitInst(BlockRef, Option<ValueRef>, Vec<ValueRef>),
    ReturnInst(ReturnKind),
    BranchInst(BranchKind),
    SignalInst(Type, Option<ValueRef>),
    ProbeInst(Type, ValueRef),
    DriveInst(ValueRef, ValueRef, Option<ValueRef>),
    /// `<result> = var <type>`
    VariableInst(Type),
    /// `<result> = load <type> <ptr>`
    LoadInst(Type, ValueRef),
    /// `store <type> <ptr> <value>`
    StoreInst(Type, ValueRef, ValueRef),
    /// `halt`
    HaltInst,
    /// The `insert` instruction.
    InsertInst(Type, ValueRef, SliceMode, ValueRef),
    /// The `extract` instruction.
    ExtractInst(Type, ValueRef, SliceMode),
}

impl InstKind {
    /// Get the result type of the instruction.
    pub fn ty(&self) -> Type {
        match *self {
            UnaryInst(_, ref ty, _) => ty.clone(),
            BinaryInst(_, ref ty, _, _) => ty.clone(),
            CompareInst(..) => int_ty(1),
            CallInst(ref ty, _, _) => ty.unwrap_func().1.clone(),
            InstanceInst(..) | WaitInst(..) | ReturnInst(_) | BranchInst(_) => void_ty(),
            SignalInst(ref ty, _) => signal_ty(ty.clone()),
            ProbeInst(ref ty, _) => ty.clone(),
            DriveInst(..) => void_ty(),
            VariableInst(ref ty) => pointer_ty(ty.clone()),
            LoadInst(ref ty, ..) => ty.clone(),
            StoreInst(..) => void_ty(),
            HaltInst => void_ty(),
            InsertInst(ref ty, ..) => ty.clone(),
            ExtractInst(ref ty, _, mode) => determine_sliced_type(ty, mode),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        match *self {
            UnaryInst(op, _, _) => Mnemonic::Unary(op.mnemonic()),
            BinaryInst(op, _, _, _) => Mnemonic::Binary(op.mnemonic()),
            CompareInst(..) => Mnemonic::Cmp,
            CallInst(..) => Mnemonic::Call,
            InstanceInst(..) => Mnemonic::Inst,
            WaitInst(..) => Mnemonic::Wait,
            ReturnInst(..) => Mnemonic::Ret,
            BranchInst(..) => Mnemonic::Br,
            SignalInst(..) => Mnemonic::Sig,
            ProbeInst(..) => Mnemonic::Prb,
            DriveInst(..) => Mnemonic::Drv,
            VariableInst(..) => Mnemonic::Var,
            LoadInst(..) => Mnemonic::Load,
            StoreInst(..) => Mnemonic::Store,
            HaltInst => Mnemonic::Halt,
            InsertInst(..) => Mnemonic::Insert,
            ExtractInst(..) => Mnemonic::Extract,
        }
    }

    /// Check if this is an instantiation instruction.
    pub fn is_instance(&self) -> bool {
        match *self {
            InstanceInst(..) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOp {
    Not,
}

impl UnaryOp {
    pub fn mnemonic(&self) -> UnaryMnemonic {
        match *self {
            UnaryOp::Not => UnaryMnemonic::Not,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Rem,
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

impl BinaryOp {
    pub fn mnemonic(&self) -> BinaryMnemonic {
        match *self {
            BinaryOp::Add => BinaryMnemonic::Add,
            BinaryOp::Sub => BinaryMnemonic::Sub,
            BinaryOp::Mul => BinaryMnemonic::Mul,
            BinaryOp::Div => BinaryMnemonic::Div,
            BinaryOp::Mod => BinaryMnemonic::Mod,
            BinaryOp::Rem => BinaryMnemonic::Rem,
            BinaryOp::Shl => BinaryMnemonic::Shl,
            BinaryOp::Shr => BinaryMnemonic::Shr,
            BinaryOp::And => BinaryMnemonic::And,
            BinaryOp::Or => BinaryMnemonic::Or,
            BinaryOp::Xor => BinaryMnemonic::Xor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompareOp {
    Eq,
    Neq,
    Slt,
    Sgt,
    Sle,
    Sge,
    Ult,
    Ugt,
    Ule,
    Uge,
}

impl CompareOp {
    pub fn to_str(self) -> &'static str {
        match self {
            CompareOp::Eq => "eq",
            CompareOp::Neq => "neq",
            CompareOp::Slt => "slt",
            CompareOp::Sgt => "sgt",
            CompareOp::Sle => "sle",
            CompareOp::Sge => "sge",
            CompareOp::Ult => "ult",
            CompareOp::Ugt => "ugt",
            CompareOp::Ule => "ule",
            CompareOp::Uge => "uge",
        }
    }

    pub fn from_str(s: &str) -> Option<CompareOp> {
        Some(match s {
            "eq" => CompareOp::Eq,
            "neq" => CompareOp::Neq,
            "slt" => CompareOp::Slt,
            "sgt" => CompareOp::Sgt,
            "sle" => CompareOp::Sle,
            "sge" => CompareOp::Sge,
            "ult" => CompareOp::Ult,
            "ugt" => CompareOp::Ugt,
            "ule" => CompareOp::Ule,
            "uge" => CompareOp::Uge,
            _ => return None,
        })
    }
}

/// The return instruction flavor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnKind {
    /// Return from a void function.
    Void,
    /// Return from a non-void function.
    Value(Type, ValueRef),
}

/// The branch flavor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKind {
    /// An unconditional branch to a block.
    Uncond(BlockRef),
    /// A conditional branch, transferring control to one block if the condition
    /// is 1, or another block if it is 0.
    Cond(ValueRef, BlockRef, BlockRef),
}

/// The insert/extract flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceMode {
    /// Extract or insert a single element.
    Element(usize),
    /// Extract or insert multiple consecutive elements.
    ///
    /// The first field corresponds to the first element to pick. The second
    /// field determines the number of elements to pick.
    Slice(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic {
    Unary(UnaryMnemonic),
    Binary(BinaryMnemonic),
    Call,
    Inst,
    Cmp,
    Wait,
    Ret,
    Br,
    Phi,
    Sig,
    Prb,
    Drv,
    Var,
    Load,
    Store,
    Halt,
    Insert,
    Extract,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnaryMnemonic {
    Not,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryMnemonic {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Rem,
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

impl Mnemonic {
    /// Convert the mnemonic to its textual representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Mnemonic::Unary(m) => m.as_str(),
            Mnemonic::Binary(m) => m.as_str(),
            Mnemonic::Call => "call",
            Mnemonic::Inst => "inst",
            Mnemonic::Cmp => "cmp",
            Mnemonic::Wait => "wait",
            Mnemonic::Ret => "ret",
            Mnemonic::Br => "br",
            Mnemonic::Phi => "phi",
            Mnemonic::Sig => "sig",
            Mnemonic::Prb => "prb",
            Mnemonic::Drv => "drv",
            Mnemonic::Var => "var",
            Mnemonic::Load => "load",
            Mnemonic::Store => "store",
            Mnemonic::Halt => "halt",
            Mnemonic::Insert => "insert",
            Mnemonic::Extract => "extract",
        }
    }
}

impl UnaryMnemonic {
    /// Convert the unary mnemonic to its textual representation.
    pub fn as_str(self) -> &'static str {
        match self {
            UnaryMnemonic::Not => "not",
        }
    }
}

impl BinaryMnemonic {
    /// Convert the binary mnemonic to its textual representation.
    pub fn as_str(self) -> &'static str {
        match self {
            BinaryMnemonic::Add => "add",
            BinaryMnemonic::Sub => "sub",
            BinaryMnemonic::Mul => "mul",
            BinaryMnemonic::Div => "div",
            BinaryMnemonic::Mod => "mod",
            BinaryMnemonic::Rem => "rem",
            BinaryMnemonic::Shl => "shl",
            BinaryMnemonic::Shr => "shr",
            BinaryMnemonic::And => "and",
            BinaryMnemonic::Or => "or",
            BinaryMnemonic::Xor => "xor",
        }
    }
}

/// Determine the accessed type of an extract/insert operation.
pub(crate) fn determine_sliced_type(ty: &TypeKind, mode: SliceMode) -> Type {
    match (ty, mode) {
        (&IntType(_), SliceMode::Element(_)) => int_ty(1),
        (&IntType(_), SliceMode::Slice(_, len)) => int_ty(len),
        (&ArrayType(_, ref ty), SliceMode::Element(_)) => ty.clone(),
        (&ArrayType(_, ref ty), SliceMode::Slice(_, len)) => array_ty(len, ty.clone()),
        (&StructType(ref tys), SliceMode::Element(i)) => tys[i].clone(),
        (&PointerType(ref ty), _) => pointer_ty(determine_sliced_type(ty, mode)),
        (&SignalType(ref ty), _) => signal_ty(determine_sliced_type(ty, mode)),
        _ => panic!(
            "invalid type/mode combination for `extract` instruction; type = {:?}, mode = {:?}",
            ty, mode
        ),
    }
}
