// Copyright (c) 2017-2019 Fabian Schuiki

//! Verification of IR integrity.
//!
//! This module implements verification of the intermediate representation. It
//! checks that functions, processes, and entities are well-formed, basic blocks
//! have terminators, and types line up.

use crate::ir::*;
use crate::ty::{int_ty, void_ty, Type};
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
    return_type: Option<Type>,
}

impl Verifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Default::default()
    }

    /// Verify the integrity of a `Module`.
    pub fn verify_module(&mut self, module: &Module) {
        for unit in module.units() {
            match &module[unit] {
                ModUnitData::Function(x) => self.verify_function(x),
                ModUnitData::Process(x) => self.verify_process(x),
                ModUnitData::Entity(x) => self.verify_entity(x),
                ModUnitData::Declare { .. } => (),
            }
        }
    }

    /// Verify the integrity of a `Function`.
    pub fn verify_function(&mut self, func: &Function) {
        self.unit = Some(format!("func {}", func.name));
        self.return_type = Some(func.sig().return_type());
        self.flags = UnitFlags::FUNCTION;
        self.verify_function_layout(&func.layout, &func.dfg);
        self.unit = None;
        self.return_type = None;
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
                    object: Some(bb.to_string()),
                    message: format!("block is empty"),
                })
            }

            for inst in layout.insts(bb) {
                // Check that there are no terminator instructions in the middle
                // of the block.
                if dfg[inst].opcode().is_terminator() && Some(inst) != layout.last_inst(bb) {
                    self.errors.push(VerifierError {
                        unit: self.unit.clone(),
                        object: Some(inst.dump(dfg).to_string()),
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
                        object: Some(bb.to_string()),
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
                object: Some(inst.dump(dfg).to_string()),
                message: format!("{} may not appear in this unit", dfg[inst].opcode()),
            });
        }

        // Check for instruction-specific invariants. This match block acts as
        // the source of truth for all restrictions imposed by instructions.
        match dfg[inst].opcode() {
            Opcode::ConstInt => {}
            Opcode::ConstTime => {}
            Opcode::Alias => {}
            Opcode::ArrayUniform => {}
            Opcode::Array => {
                self.verify_args_match_ty(inst, dfg, dfg.inst_type(inst).unwrap_array().1);
            }
            Opcode::Struct => {
                for (i, ty) in dfg.inst_type(inst).unwrap_struct().iter().enumerate() {
                    self.verify_arg_matches_ty(inst, dfg, dfg[inst].args()[i], ty);
                }
            }
            Opcode::Not => {
                self.assert_inst_unary(inst, dfg);
                self.verify_bitwise_compatible_ty(inst, dfg);
                self.verify_args_match_inst_ty(inst, dfg);
            }
            Opcode::Neg => {
                self.assert_inst_unary(inst, dfg);
                self.verify_arith_compatible_ty(inst, dfg);
                self.verify_args_match_inst_ty(inst, dfg);
            }
            Opcode::Add
            | Opcode::Sub
            | Opcode::Smul
            | Opcode::Sdiv
            | Opcode::Smod
            | Opcode::Srem
            | Opcode::Umul
            | Opcode::Udiv
            | Opcode::Umod
            | Opcode::Urem => {
                self.assert_inst_binary(inst, dfg);
                self.verify_arith_compatible_ty(inst, dfg);
                self.verify_args_match_inst_ty(inst, dfg);
            }
            Opcode::And | Opcode::Or | Opcode::Xor => {
                self.assert_inst_binary(inst, dfg);
                self.verify_bitwise_compatible_ty(inst, dfg);
                self.verify_args_match_inst_ty(inst, dfg);
            }
            Opcode::Eq
            | Opcode::Neq
            | Opcode::Slt
            | Opcode::Sgt
            | Opcode::Sle
            | Opcode::Sge
            | Opcode::Ult
            | Opcode::Ugt
            | Opcode::Ule
            | Opcode::Uge => {
                self.assert_inst_binary(inst, dfg);
                self.verify_bool_ty(inst, dfg);
                self.verify_arg_tys_match(inst, dfg);
            }
            Opcode::Shl | Opcode::Shr => {
                self.assert_inst_ternary(inst, dfg);
                self.verify_shift_inst(inst, dfg);
            }
            Opcode::Mux => {
                self.assert_inst_binary(inst, dfg);
                self.verify_mux_inst(inst, dfg);
            }
            Opcode::Sig => {
                self.assert_inst_unary(inst, dfg);
                self.verify_sig_inst(inst, dfg);
            }
            Opcode::Var => {
                self.assert_inst_unary(inst, dfg);
                self.verify_var_inst(inst, dfg);
            }
            Opcode::Halt => {}
            Opcode::Ret => {
                self.assert_inst_nullary(inst, dfg);
                self.verify_return_type(inst, dfg, &void_ty());
            }
            Opcode::RetValue => {
                self.assert_inst_unary(inst, dfg);
                self.verify_return_type(inst, dfg, &dfg.value_type(dfg[inst].args()[0]));
            }
            Opcode::Br => {
                self.assert_inst_jump(inst, dfg);
            }
            Opcode::BrCond => {
                self.assert_inst_branch(inst, dfg);
                self.verify_args_match_ty(inst, dfg, &int_ty(1));
            }
            _ => {}
            // _ => unimplemented!("verify `{}`", inst.dump(dfg)),
        }
    }

    /// Assert that an instruction has nullary format.
    fn assert_inst_nullary(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Nullary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have nullary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has unary format.
    fn assert_inst_unary(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Unary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have unary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has binary format.
    fn assert_inst_binary(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Binary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have binary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has ternary format.
    fn assert_inst_ternary(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Ternary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have ternary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has jump format.
    fn assert_inst_jump(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Jump { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have jump format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has branch format.
    fn assert_inst_branch(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        match &dfg[inst] {
            InstData::Branch { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have branch format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Verify that the types of an instruction's arguments agree.
    fn verify_arg_tys_match(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = match dfg[inst].args().get(0) {
            Some(&arg) => dfg.value_type(arg),
            None => return,
        };
        let mut mismatch = false;
        for &arg in &dfg[inst].args()[1..] {
            let arg_ty = dfg.value_type(arg);
            if arg_ty != ty {
                mismatch = true;
            }
        }
        if mismatch {
            let tys: Vec<_> = dfg[inst]
                .args()
                .into_iter()
                .map(|&arg| dfg.value_type(arg).to_string())
                .collect();
            let tys: String = tys.join(", ");
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!("argument types must match (but are {})", tys),
            });
        }
    }

    /// Verify that the types of an instruction's arguments match the return
    /// type of the instruction itself.
    fn verify_args_match_inst_ty(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        self.verify_args_match_ty(inst, dfg, &ty);
    }

    /// Verify that the types of an instruction's arguments match a given type.
    fn verify_args_match_ty(&mut self, inst: Inst, dfg: &DataFlowGraph, ty: &Type) {
        for &arg in dfg[inst].args() {
            self.verify_arg_matches_ty(inst, dfg, arg, ty);
        }
    }

    /// Verify that the type of an instruction's argument matches a given type.
    fn verify_arg_matches_ty(&mut self, inst: Inst, dfg: &DataFlowGraph, arg: Value, ty: &Type) {
        let arg_ty = dfg.value_type(arg);
        if arg_ty != *ty {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!(
                    "argument {} must be of type {} (but is {})",
                    arg, ty, arg_ty,
                ),
            });
        }
    }

    /// Verify that an instruction's return type is `i1`.
    fn verify_bool_ty(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        if ty.is_int() && ty.unwrap_int() == 1 {
            return;
        }
        self.errors.push(VerifierError {
            unit: self.unit.clone(),
            object: Some(inst.dump(dfg).to_string()),
            message: format!("return type must be i1 (but is {})", ty),
        });
    }

    /// Verify that an instruction's return type is compatible with bitwise
    /// operations.
    fn verify_bitwise_compatible_ty(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        self.verify_arith_compatible_ty(inst, dfg);
    }

    /// Verify that an instruction's return type is compatible with arithmetic
    /// operations.
    fn verify_arith_compatible_ty(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        if ty.is_int() {
            return;
        }
        if ty.is_signal() && ty.unwrap_signal().is_int() {
            return;
        }
        self.errors.push(VerifierError {
            unit: self.unit.clone(),
            object: Some(inst.dump(dfg).to_string()),
            message: format!("return type must be iN or iN$ (but is {})", ty),
        });
    }

    /// Verify that the types of a shift instruction line up.
    fn verify_shift_inst(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        let base = dfg[inst].args()[0];
        let hidden = dfg[inst].args()[1];
        let amount = dfg[inst].args()[2];
        self.verify_arg_matches_ty(inst, dfg, base, &ty);
        let amount_ty = dfg.value_type(amount);
        if !amount_ty.is_int() && !(amount_ty.is_signal() && amount_ty.unwrap_signal().is_int()) {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!(
                    "type of shift amount must be iN or iN$ (but is {})",
                    amount_ty
                ),
            });
        }
        let base_ty = dfg.value_type(base);
        let hidden_ty = dfg.value_type(hidden);
        if base_ty.is_signal() != hidden_ty.is_signal()
            || base_ty.is_pointer() != hidden_ty.is_pointer()
        {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!(
                    "shift base and hidden value types must be compatible (but are {} and {})",
                    base_ty, hidden_ty
                ),
            });
        }
        let (base_inner_ty, hidden_inner_ty) = if ty.is_signal() {
            (ty.unwrap_signal(), hidden_ty.unwrap_signal())
        } else if ty.is_pointer() {
            (ty.unwrap_pointer(), hidden_ty.unwrap_pointer())
        } else {
            (&ty, &hidden_ty)
        };
        if base_inner_ty.is_int() && hidden_inner_ty.is_int() {
            return;
        }
        if base_inner_ty.is_array()
            && hidden_inner_ty.is_array()
            && base_inner_ty.unwrap_array() == hidden_inner_ty.unwrap_array()
        {
            return;
        }
        self.errors.push(VerifierError {
            unit: self.unit.clone(),
            object: Some(inst.dump(dfg).to_string()),
            message: format!(
                "shift base and hidden value types must be compatible (but are {} and {})",
                base_ty, hidden_ty
            ),
        });
    }

    /// Verify that the types of a mux instruction line up.
    fn verify_mux_inst(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        let array = dfg[inst].args()[0];
        let array_ty = dfg.value_type(array);
        if !array_ty.is_array() || array_ty.unwrap_array().1 != &ty {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!(
                    "array {} element and return type {} must agree",
                    array_ty, ty
                ),
            });
        }
        let sel = dfg[inst].args()[1];
        let sel_ty = dfg.value_type(sel);
        if !sel_ty.is_int() && !(sel_ty.is_signal() && sel_ty.unwrap_signal().is_int()) {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!("type of selector must be iN or iN$ (but is {})", sel_ty),
            });
        }
    }

    /// Verify that the types of a sig instruction line up.
    fn verify_sig_inst(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        if !ty.is_signal() {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!("type {} must be a signal", ty),
            });
        }
        self.verify_args_match_ty(inst, dfg, ty.unwrap_signal());
    }

    /// Verify that the types of a var instruction line up.
    fn verify_var_inst(&mut self, inst: Inst, dfg: &DataFlowGraph) {
        let ty = dfg.inst_type(inst);
        if !ty.is_pointer() {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!("type {} must be a pointer", ty),
            });
        }
        self.verify_args_match_ty(inst, dfg, ty.unwrap_pointer());
    }

    /// Verify that the return type of the enclosing function is compatible with
    /// a ret instruction.
    fn verify_return_type(&mut self, inst: Inst, dfg: &DataFlowGraph, ty: &Type) {
        let func_ty = self.return_type.clone().unwrap_or_else(void_ty);
        if func_ty != *ty {
            self.errors.push(VerifierError {
                unit: self.unit.clone(),
                object: Some(inst.dump(dfg).to_string()),
                message: format!(
                    "requires function to have return type {} (but has {})",
                    ty, func_ty
                ),
            });
        }
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
    pub object: Option<String>,
    /// The error message.
    pub message: String,
}

impl Display for VerifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref unit) = self.unit {
            write!(f, "{}: ", unit)?;
        }
        if let Some(ref object) = self.object {
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
