// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD instructions.
//!
//! This module implements the various instructions of the intermediate
//! representation.

use crate::ir::Value;
use num::BigInt;

/// An instruction format.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum InstData {
    /// `a = const iN imm`
    ConstInt { opcode: Opcode, imm: BigInt },
}

impl InstData {
    /// Get the opcode of the instruction.
    pub fn opcode(&self) -> Opcode {
        match *self {
            InstData::ConstInt { opcode, .. } => opcode,
        }
    }

    /// Get the arguments of an instruction.
    pub fn args(&self) -> &[Value] {
        match self {
            InstData::ConstInt { .. } => &[],
        }
    }

    /// Mutable access to the arguments of an instruction.
    pub fn args_mut(&mut self) -> &mut [Value] {
        match self {
            InstData::ConstInt { .. } => &mut [],
        }
    }

    /// Return the const int constructed by this instruction.
    pub fn get_const_int(&self) -> Option<&BigInt> {
        match self {
            InstData::ConstInt { imm, .. } => Some(imm),
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
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Opcode::ConstInt => "const",
            }
        )
    }
}

impl Opcode {
    /// Check if this instruction is a constant.
    pub fn is_const(self) -> bool {
        match self {
            Opcode::ConstInt => true,
            _ => false,
        }
    }
}
