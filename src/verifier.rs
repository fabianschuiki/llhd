// Copyright (c) 2017-2019 Fabian Schuiki

//! Verification of IR integrity.
//!
//! This module implements verification of the intermediate representation. It
//! checks that functions, processes, and entities are well-formed, basic blocks
//! have terminators, and types line up.

use crate::ir::*;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// An IR verifier.
///
/// The `Verifier` acts as a context to call the various IR checking functions
/// on. It keeps track of errors.
#[derive(Default)]
pub struct Verifier {
    errors: VerifierErrors,
    unit: Option<String>,
    flags: UnitFlags,
}

impl Verifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Default::default()
    }

    /// Verify the integrity of a `Function`.
    pub fn verify_function(&mut self, func: &Function) {
        self.unit = Some(format!("func {}", func.name));
        self.flags = UnitFlags::FUNCTION;
        self.verify_function_layout(&func.layout, &func.dfg);
        self.unit = None;
    }

    /// Verify the integrity of a `Process`.
    pub fn verify_process(&mut self, prok: &Process) {
        self.unit = Some(format!("proc {}", prok.name));
        self.flags = UnitFlags::PROCESS;
        self.verify_function_layout(&prok.layout, &prok.dfg);
        self.unit = None;
    }

    /// Verify the integrity of an `Entity`.
    pub fn verify_entity(&mut self, ent: &Entity) {
        self.unit = Some(format!("entity {}", ent.name));
        self.flags = UnitFlags::ENTITY;
        self.verify_inst_layout(&ent.layout, &ent.dfg);
        self.unit = None;
    }

    /// Verify the integrity of the BB and instruction layout.
    pub fn verify_function_layout(&mut self, layout: &FunctionLayout, dfg: &DataFlowGraph) {
        if layout.first_block().is_none() {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: None,
                message: format!("layout has no entry block"),
            });
        }
        for bb in layout.blocks() {
            // Check that the block has at least one instruction.
            if layout.first_inst(bb).is_none() {
                self.errors.push(VerifierError {
                    unit: self.unit.clone(),
                    object: Some(bb.into()),
                    message: format!("block is empty"),
                })
            }

            for inst in layout.insts(bb) {
                // Check that there are no terminator instructions in the middle
                // of the block.
                if dfg[inst].opcode().is_terminator() && Some(inst) != layout.last_inst(bb) {
                    self.errors.push(VerifierError {
                        unit: self.unit.clone(),
                        object: Some(inst.into()),
                        message: format!(
                            "terminator instruction `{}` must be at the end of block {}",
                            inst.dump(dfg),
                            bb
                        ),
                    });
                }

                // Check that the last instruction in the block is a terminator.
                if Some(inst) == layout.last_inst(bb) && !dfg[inst].opcode().is_terminator() {
                    self.errors.push(VerifierError {
                        unit: self.unit.clone(),
                        object: Some(bb.into()),
                        message: format!(
                            "last instruction `{}` must be a terminator",
                            inst.dump(dfg)
                        ),
                    })
                }

                // Check the instruction itself.
                self.verify_inst(inst, dfg);
            }
        }
    }

    /// Verify the integrity of the instruction layout.
    pub fn verify_inst_layout(&mut self, layout: &InstLayout, dfg: &DataFlowGraph) {
        for inst in layout.insts() {
            self.verify_inst(inst, dfg);
        }
    }

    /// Verify the integrity of a single instruction.
    pub fn verify_inst(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        // Check that the instruction may appear in the surrounding unit.
        if !dfg[inst].opcode().valid_in().contains(self.flags) {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.into()),
                message: format!("{} may not appear in this unit", dfg[inst].opcode()),
            });
        }

        // TODO: Check types line up.
    }

    /// Finish verification and return the result.
    ///
    /// Consumes the verifier.
    pub fn finish(self) -> Result<(), VerifierErrors> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// Finish verification and panic if errors occurred.
    ///
    /// Consumes the verifier.
    pub fn finish_panic(self) {
        match self.finish() {
            Ok(()) => (),
            Err(errs) => panic!("Verification failed:\n{}", errs),
        }
    }
}

/// A verification error.
#[derive(Debug)]
pub struct VerifierError {
    /// The unit within which caused the error.
    pub unit: Option<String>,
    /// The object which caused the error.
    pub object: Option<AnyObject>,
    /// The error message.
    pub message: String,
}

impl Display for VerifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref unit) = self.unit {
            write!(f, "{}: ", unit)?;
        }
        if let Some(object) = self.object {
            write!(f, "{}: ", object)?;
        }
        write!(f, "{}", self.message)?;
        Ok(())
    }
}

/// A list of verification errors.
#[derive(Debug, Default)]
pub struct VerifierErrors(pub Vec<VerifierError>);

impl Deref for VerifierErrors {
    type Target = Vec<VerifierError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VerifierErrors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for VerifierErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for err in self.iter() {
            writeln!(f, "- {}", err)?;
        }
        Ok(())
    }
}
