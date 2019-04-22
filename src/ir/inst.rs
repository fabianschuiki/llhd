// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD instructions.
//!
//! This module implements the various instructions of the intermediate
//! representation.

use crate::{
    ir::{Block, DataFlowGraph, ExtUnit, Inst, Unit, UnitBuilder, Value},
    ty::{array_ty, int_ty, pointer_ty, signal_ty, struct_ty, time_ty, void_ty, Type},
    ConstTime,
};
use num::BigInt;

/// An instruction format.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum InstData {
    /// `a = const iN imm`
    ConstInt { opcode: Opcode, imm: BigInt },
    /// `a = const time imm`
    ConstTime { opcode: Opcode, imm: ConstTime },
    /// `opcode imm, type x`
    Array {
        opcode: Opcode,
        imm: usize,
        args: [Value; 1],
    },
    /// `opcode args`
    Aggregate { opcode: Opcode, args: Vec<Value> },
    /// `opcode`
    Nullary { opcode: Opcode },
    /// `opcode type x`
    Unary { opcode: Opcode, args: [Value; 1] },
    /// `opcode type x, y`
    Binary { opcode: Opcode, args: [Value; 2] },
    /// `opcode type x, y, z`
    Ternary { opcode: Opcode, args: [Value; 3] },
    /// `opcode bb`
    Jump { opcode: Opcode, bbs: [Block; 1] },
    /// `opcode x, bb0, bb1`
    Branch {
        opcode: Opcode,
        args: [Value; 1],
        bbs: [Block; 2],
    },
    /// `opcode bb, args`
    Wait {
        opcode: Opcode,
        bbs: [Block; 1],
        args: Vec<Value>,
    },
    /// `a = opcode type unit (inputs) -> (outputs)`
    Call {
        opcode: Opcode,
        unit: ExtUnit,
        ins: u16,
        args: Vec<Value>,
    },
    /// `a = opcode type x, y, imm0, imm1`
    InsExt {
        opcode: Opcode,
        args: [Value; 2],
        imms: [usize; 2],
    },
}

impl InstData {
    /// Get the opcode of the instruction.
    pub fn opcode(&self) -> Opcode {
        match *self {
            InstData::ConstInt { opcode, .. } => opcode,
            InstData::ConstTime { opcode, .. } => opcode,
            InstData::Array { opcode, .. } => opcode,
            InstData::Aggregate { opcode, .. } => opcode,
            InstData::Nullary { opcode, .. } => opcode,
            InstData::Unary { opcode, .. } => opcode,
            InstData::Binary { opcode, .. } => opcode,
            InstData::Ternary { opcode, .. } => opcode,
            InstData::Jump { opcode, .. } => opcode,
            InstData::Branch { opcode, .. } => opcode,
            InstData::Wait { opcode, .. } => opcode,
            InstData::Call { opcode, .. } => opcode,
            InstData::InsExt { opcode, .. } => opcode,
        }
    }

    /// Get the arguments of an instruction.
    pub fn args(&self) -> &[Value] {
        match self {
            InstData::ConstInt { .. } => &[],
            InstData::ConstTime { .. } => &[],
            InstData::Array { args, .. } => args,
            InstData::Aggregate { args, .. } => args,
            InstData::Nullary { .. } => &[],
            InstData::Unary { args, .. } => args,
            InstData::Binary { args, .. } => args,
            InstData::Ternary { args, .. } => args,
            InstData::Jump { .. } => &[],
            InstData::Branch { args, .. } => args,
            InstData::Wait { args, .. } => args,
            InstData::Call { args, .. } => args,
            InstData::InsExt {
                opcode: Opcode::ExtField,
                args,
                ..
            }
            | InstData::InsExt {
                opcode: Opcode::ExtSlice,
                args,
                ..
            } => &args[0..1],
            InstData::InsExt { args, .. } => args,
        }
    }

    /// Mutable access to the arguments of an instruction.
    pub fn args_mut(&mut self) -> &mut [Value] {
        match self {
            InstData::ConstInt { .. } => &mut [],
            InstData::ConstTime { .. } => &mut [],
            InstData::Array { args, .. } => args,
            InstData::Aggregate { args, .. } => args,
            InstData::Nullary { .. } => &mut [],
            InstData::Unary { args, .. } => args,
            InstData::Binary { args, .. } => args,
            InstData::Ternary { args, .. } => args,
            InstData::Jump { .. } => &mut [],
            InstData::Branch { args, .. } => args,
            InstData::Wait { args, .. } => args,
            InstData::Call { args, .. } => args,
            InstData::InsExt {
                opcode: Opcode::ExtField,
                args,
                ..
            }
            | InstData::InsExt {
                opcode: Opcode::ExtSlice,
                args,
                ..
            } => &mut args[0..1],
            InstData::InsExt { args, .. } => args,
        }
    }

    /// Get the input arguments of a call instruction.
    pub fn input_args(&self) -> &[Value] {
        match *self {
            InstData::Call { ref args, ins, .. } => &args[0..ins as usize],
            _ => &[],
        }
    }

    /// Get the output arguments of a call instruction.
    pub fn output_args(&self) -> &[Value] {
        match *self {
            InstData::Call { ref args, ins, .. } => &args[ins as usize..],
            _ => &[],
        }
    }

    /// Get the BBs of an instruction.
    pub fn blocks(&self) -> &[Block] {
        match self {
            InstData::ConstInt { .. } => &[],
            InstData::ConstTime { .. } => &[],
            InstData::Array { .. } => &[],
            InstData::Aggregate { .. } => &[],
            InstData::Nullary { .. } => &[],
            InstData::Unary { .. } => &[],
            InstData::Binary { .. } => &[],
            InstData::Ternary { .. } => &[],
            InstData::Jump { bbs, .. } => bbs,
            InstData::Branch { bbs, .. } => bbs,
            InstData::Wait { bbs, .. } => bbs,
            InstData::Call { .. } => &[],
            InstData::InsExt { .. } => &[],
        }
    }

    /// Mutable access to the BBs of an instruction.
    pub fn blocks_mut(&mut self) -> &mut [Block] {
        match self {
            InstData::ConstInt { .. } => &mut [],
            InstData::ConstTime { .. } => &mut [],
            InstData::Array { .. } => &mut [],
            InstData::Aggregate { .. } => &mut [],
            InstData::Nullary { .. } => &mut [],
            InstData::Unary { .. } => &mut [],
            InstData::Binary { .. } => &mut [],
            InstData::Ternary { .. } => &mut [],
            InstData::Jump { bbs, .. } => bbs,
            InstData::Branch { bbs, .. } => bbs,
            InstData::Wait { bbs, .. } => bbs,
            InstData::Call { .. } => &mut [],
            InstData::InsExt { .. } => &mut [],
        }
    }

    /// Return the const int constructed by this instruction.
    pub fn get_const_int(&self) -> Option<&BigInt> {
        match self {
            InstData::ConstInt { imm, .. } => Some(imm),
            _ => None,
        }
    }

    /// Return the const time constructed by this instruction.
    pub fn get_const_time(&self) -> Option<&ConstTime> {
        match self {
            InstData::ConstTime { imm, .. } => Some(imm),
            _ => None,
        }
    }
}

/// An instruction opcode.
///
/// This enum represents the actual instruction, whereas `InstData` covers the
/// format and arguments of the instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    ConstInt,
    ConstTime,
    Alias,
    ArrayUniform,
    Array,
    Struct,

    Not,
    Neg,

    Add,
    Sub,
    And,
    Or,
    Xor,
    Smul,
    Sdiv,
    Smod,
    Srem,
    Umul,
    Udiv,
    Umod,
    Urem,

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

    Shl,
    Shr,
    Mux,
    Reg,
    InsField,
    InsSlice,
    ExtField,
    ExtSlice,

    Call,
    Inst,

    Sig,
    Drv,
    Prb,

    Var,
    Ld,
    St,

    Halt,
    Ret,
    RetValue,
    Br,
    BrCond,
    Wait,
    WaitTime,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Opcode::ConstInt => "const",
                Opcode::ConstTime => "const",
                Opcode::Alias => "alias",
                Opcode::ArrayUniform => "array",
                Opcode::Array => "array",
                Opcode::Struct => "struct",
                Opcode::Not => "not",
                Opcode::Neg => "neg",
                Opcode::Add => "add",
                Opcode::Sub => "sub",
                Opcode::And => "and",
                Opcode::Or => "or",
                Opcode::Xor => "xor",
                Opcode::Smul => "smul",
                Opcode::Sdiv => "sdiv",
                Opcode::Smod => "smod",
                Opcode::Srem => "srem",
                Opcode::Umul => "umul",
                Opcode::Udiv => "udiv",
                Opcode::Umod => "umod",
                Opcode::Urem => "urem",
                Opcode::Eq => "eq",
                Opcode::Neq => "neq",
                Opcode::Slt => "slt",
                Opcode::Sgt => "sgt",
                Opcode::Sle => "sle",
                Opcode::Sge => "sge",
                Opcode::Ult => "ult",
                Opcode::Ugt => "ugt",
                Opcode::Ule => "ule",
                Opcode::Uge => "uge",
                Opcode::Shl => "shl",
                Opcode::Shr => "shr",
                Opcode::Mux => "mux",
                Opcode::Reg => "reg",
                Opcode::InsField => "insf",
                Opcode::InsSlice => "inss",
                Opcode::ExtField => "extf",
                Opcode::ExtSlice => "exts",
                Opcode::Call => "call",
                Opcode::Inst => "inst",
                Opcode::Sig => "sig",
                Opcode::Drv => "drv",
                Opcode::Prb => "prb",
                Opcode::Var => "var",
                Opcode::Ld => "ld",
                Opcode::St => "st",
                Opcode::Halt => "halt",
                Opcode::Ret => "ret",
                Opcode::RetValue => "ret",
                Opcode::Br => "br",
                Opcode::BrCond => "br",
                Opcode::Wait => "wait",
                Opcode::WaitTime => "wait",
            }
        )
    }
}

impl Opcode {
    /// Check if this instruction is a constant.
    pub fn is_const(self) -> bool {
        match self {
            Opcode::ConstInt => true,
            Opcode::ConstTime => true,
            _ => false,
        }
    }

    /// Check if this instruction is a terminator.
    pub fn is_terminator(self) -> bool {
        match self {
            Opcode::Halt
            | Opcode::Ret
            | Opcode::RetValue
            | Opcode::Br
            | Opcode::BrCond
            | Opcode::Wait
            | Opcode::WaitTime => true,
            _ => false,
        }
    }

    /// Check if this is a return instruction.
    pub fn is_return(self) -> bool {
        match self {
            Opcode::Ret | Opcode::RetValue => true,
            _ => false,
        }
    }
}
