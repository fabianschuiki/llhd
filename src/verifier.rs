// Copyright (c) 2017-2021 Fabian Schuiki

//! Verification of IR integrity.
//!
//! This module implements verification of the intermediate representation. It
//! checks that functions, processes, and entities are well-formed, basic blocks
//! have terminators, and types line up.

use crate::{
    ir::{prelude::*, InstData, UnitFlags, ValueData},
    ty::{array_ty, int_ty, pointer_ty, signal_ty, time_ty, void_ty, Type},
};
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
    unit_name: Option<String>,
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
            self.verify_unit(unit);
        }
    }

    /// Verify the integrity of a `UnitData`.
    pub fn verify_unit(&mut self, unit: Unit) {
        self.unit_name = Some(format!("{} {}", unit.kind(), unit.name()));
        match unit.kind() {
            UnitKind::Function => {
                self.return_type = Some(unit.sig().return_type());
                self.flags = UnitFlags::FUNCTION;
            }
            UnitKind::Process => {
                self.flags = UnitFlags::PROCESS;
            }
            UnitKind::Entity => {
                self.flags = UnitFlags::ENTITY;
            }
        }

        let domtree = unit.domtree();
        if unit.first_block().is_none() {
            self.errors.push(VerifierError {
                unit: self.unit_name.clone(),
                object: None,
                message: format!("layout has no entry block"),
            });
        }
        for bb in unit.blocks() {
            // Check that the block has at least one instruction.
            if unit.first_inst(bb).is_none() {
                self.errors.push(VerifierError {
                    unit: self.unit_name.clone(),
                    object: Some(bb.to_string()),
                    message: format!("block is empty"),
                })
            }

            for inst in unit.insts(bb) {
                // Check that there are no terminator instructions in the middle
                // of the block.
                if unit[inst].opcode().is_terminator() && Some(inst) != unit.last_inst(bb) {
                    self.errors.push(VerifierError {
                        unit: self.unit_name.clone(),
                        object: Some(inst.dump(&unit).to_string()),
                        message: format!("terminator must be at the end of block {}", bb),
                    });
                }

                // Check that the last instruction in the block is a terminator.
                if Some(inst) == unit.last_inst(bb) && !unit[inst].opcode().is_terminator() {
                    self.errors.push(VerifierError {
                        unit: self.unit_name.clone(),
                        object: Some(bb.to_string()),
                        message: format!(
                            "last instruction `{}` must be a terminator",
                            inst.dump(&unit)
                        ),
                    })
                }

                // Check the instruction itself.
                self.verify_inst(inst, unit);

                // Check that the instruction dominates all its uses.
                if let Some(result) = unit.get_inst_result(inst) {
                    for &u in unit.uses(result) {
                        if !domtree.inst_dominates_inst(&unit, inst, u) {
                            self.errors.push(VerifierError {
                                unit: self.unit_name.clone(),
                                object: Some(inst.dump(&unit).to_string()),
                                message: format!("does not dominate use in `{}`", u.dump(&unit),),
                            });
                        }
                    }
                }
            }
        }

        self.unit_name = None;
        self.return_type = None;
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

    /// Verify the integrity of a single instruction.
    pub fn verify_inst(&mut self, inst: Inst, unit: Unit) {
        InstVerifier {
            verifier: self,
            unit,
        }
        .verify_inst(inst);
    }
}

/// An instruction verifier.
struct InstVerifier<'a> {
    verifier: &'a mut Verifier,
    unit: Unit<'a>,
}

impl<'a> Deref for InstVerifier<'a> {
    type Target = Verifier;
    fn deref(&self) -> &Verifier {
        self.verifier
    }
}

impl<'a> DerefMut for InstVerifier<'a> {
    fn deref_mut(&mut self) -> &mut Verifier {
        self.verifier
    }
}

impl<'a> InstVerifier<'a> {
    #[inline(always)]
    fn unit(&self) -> Unit<'a> {
        self.unit
    }

    fn is_value_defined(&self, value: Value) -> bool {
        match self.unit()[value] {
            ValueData::Invalid => false,
            ValueData::Inst { inst, .. } => self.unit.is_inst_inserted(inst),
            ValueData::Arg { .. } => true,
            ValueData::Placeholder { .. } => false,
        }
    }

    fn is_block_defined(&self, block: Block) -> bool {
        self.unit.is_block_inserted(block)
    }

    /// Verify the integrity of a single instruction.
    pub fn verify_inst(&mut self, inst: Inst) {
        let unit = self.unit;

        // Check that the instruction may appear in the surrounding unit.
        if !unit[inst].opcode().valid_in().contains(self.flags) {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&unit).to_string()),
                message: format!("{} may not appear in this unit", unit[inst].opcode()),
            });
        }

        // Check that none of the arguments are invalid, and all have a
        // definition.
        let mut args_invalid = false;
        for &value in unit[inst].args() {
            if value.is_invalid() {
                if unit[inst].opcode() == Opcode::Reg {
                    // Optional trigger condition in register uses `invalid` to
                    // indicate that the value is not use. This is ugly.
                    continue;
                }
                args_invalid = true;
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&unit).to_string()),
                    message: format!("{} uses invalid value", unit[inst].opcode()),
                });
                continue;
            }
            if !self.is_value_defined(value) {
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&unit).to_string()),
                    message: format!("value {} has no definition", value.dump(&unit)),
                });
            }
        }
        for &block in unit[inst].blocks() {
            if block.is_invalid() {
                args_invalid = true;
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&unit).to_string()),
                    message: format!("{} uses invalid block", unit[inst].opcode()),
                });
                continue;
            }
            if !self.is_block_defined(block) {
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&unit).to_string()),
                    message: format!("block {} has no definition", block.dump(&unit)),
                });
            }
        }
        if args_invalid {
            return;
        }

        // Check for instruction-specific invariants. This match block acts as
        // the source of truth for all restrictions imposed by instructions.
        match unit[inst].opcode() {
            Opcode::ConstInt => {}
            Opcode::ConstTime => {}
            Opcode::Alias => {}
            Opcode::ArrayUniform => {}
            Opcode::Array => {
                self.verify_args_match_ty(inst, unit.inst_type(inst).unwrap_array().1);
            }
            Opcode::Struct => {
                for (i, ty) in unit.inst_type(inst).unwrap_struct().iter().enumerate() {
                    self.verify_arg_matches_ty(inst, unit[inst].args()[i], ty);
                }
            }
            Opcode::Not => {
                self.assert_inst_unary(inst);
                self.verify_bitwise_compatible_ty(inst);
                self.verify_args_match_inst_ty(inst);
            }
            Opcode::Neg => {
                self.assert_inst_unary(inst);
                self.verify_arith_compatible_ty(inst);
                self.verify_args_match_inst_ty(inst);
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
                self.assert_inst_binary(inst);
                self.verify_arith_compatible_ty(inst);
                self.verify_args_match_inst_ty(inst);
            }
            Opcode::And | Opcode::Or | Opcode::Xor => {
                self.assert_inst_binary(inst);
                self.verify_bitwise_compatible_ty(inst);
                self.verify_args_match_inst_ty(inst);
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
                self.assert_inst_binary(inst);
                self.verify_bool_ty(inst);
                self.verify_arg_tys_match(inst);
            }
            Opcode::Shl | Opcode::Shr => {
                self.assert_inst_ternary(inst);
                self.verify_shift_inst(inst);
            }
            Opcode::Mux => {
                self.assert_inst_binary(inst);
                self.verify_mux_inst(inst);
            }
            Opcode::Reg => {
                self.assert_inst_reg(inst);
                self.verify_reg_inst(inst);
            }
            Opcode::InsField => {
                self.assert_inst_insext(inst);
                self.verify_ins_field_inst(inst);
            }
            Opcode::ExtField => {
                self.assert_inst_insext(inst);
                self.verify_ext_field_inst(inst);
            }
            Opcode::InsSlice => {
                self.assert_inst_insext(inst);
                self.verify_ins_slice_inst(inst);
            }
            Opcode::ExtSlice => {
                self.assert_inst_insext(inst);
                self.verify_ext_slice_inst(inst);
            }
            Opcode::Con => {
                self.assert_inst_binary(inst);
                self.verify_arg_ty_is_signal(inst, unit[inst].args()[0]);
                self.verify_arg_tys_match(inst);
            }
            Opcode::Del => {
                self.assert_inst_ternary(inst);
                self.verify_arg_ty_is_signal(inst, unit[inst].args()[0]);
                self.verify_arg_matches_ty(
                    inst,
                    unit[inst].args()[1],
                    &unit.value_type(unit[inst].args()[0]),
                );
                self.verify_arg_matches_ty(inst, unit[inst].args()[2], &time_ty());
            }
            Opcode::Call => {
                // TODO: properly check argument types match the declaration
            }
            Opcode::Inst => {
                // TODO: properly check argument types match the declaration
            }
            Opcode::Sig => {
                self.assert_inst_unary(inst);
                self.verify_sig_inst(inst);
            }
            Opcode::Prb => {
                self.assert_inst_unary(inst);
                self.verify_prb_inst(inst);
            }
            Opcode::Drv => {
                self.assert_inst_ternary(inst);
                self.verify_drv_inst(inst);
            }
            Opcode::DrvCond => {
                self.assert_inst_quaternary(inst);
                self.verify_drv_inst(inst);
            }
            Opcode::Var => {
                self.assert_inst_unary(inst);
                self.verify_var_inst(inst);
            }
            Opcode::Ld => {
                self.assert_inst_unary(inst);
                self.verify_ld_inst(inst);
            }
            Opcode::St => {
                self.assert_inst_binary(inst);
                self.verify_st_inst(inst);
            }
            Opcode::Halt => {}
            Opcode::Ret => {
                self.assert_inst_nullary(inst);
                self.verify_return_type(inst, &void_ty());
            }
            Opcode::RetValue => {
                self.assert_inst_unary(inst);
                self.verify_return_type(inst, &unit.value_type(unit[inst].args()[0]));
            }
            Opcode::Phi => {
                self.assert_inst_phi(inst);
                self.verify_args_match_inst_ty(inst);
            }
            Opcode::Br => {
                self.assert_inst_jump(inst);
            }
            Opcode::BrCond => {
                self.assert_inst_branch(inst);
                self.verify_args_match_ty(inst, &int_ty(1));
            }
            Opcode::Wait => {
                self.assert_inst_wait(inst);
            }
            Opcode::WaitTime => {
                self.assert_inst_wait(inst);
                self.verify_arg_matches_ty(inst, unit[inst].args()[0], &time_ty());
            }
        }
    }

    /// Assert that an instruction has nullary format.
    fn assert_inst_nullary(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Nullary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have nullary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has unary format.
    fn assert_inst_unary(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Unary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have unary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has binary format.
    fn assert_inst_binary(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Binary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have binary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has ternary format.
    fn assert_inst_ternary(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Ternary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have ternary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has quaternary format.
    fn assert_inst_quaternary(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Quaternary { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have quaternary format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has jump format.
    fn assert_inst_jump(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Jump { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have jump format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has phi format.
    fn assert_inst_phi(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Phi { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have phi format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has branch format.
    fn assert_inst_branch(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Branch { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have branch format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has wait format.
    fn assert_inst_wait(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Wait { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have wait format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has reg format.
    fn assert_inst_reg(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::Reg { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have reg format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Assert that an instruction has ins/ext format.
    fn assert_inst_insext(&mut self, inst: Inst) {
        match &self.unit()[inst] {
            InstData::InsExt { .. } => (),
            fmt => panic!(
                "{0:?} ({0}) should have ins/ext format, but has {1:?}",
                fmt.opcode(),
                fmt
            ),
        }
    }

    /// Verify that the types of an instruction's arguments agree.
    fn verify_arg_tys_match(&mut self, inst: Inst) {
        let ty = match self.unit()[inst].args().get(0) {
            Some(&arg) => self.unit.value_type(arg),
            None => return,
        };
        let mut mismatch = false;
        for &arg in &self.unit()[inst].args()[1..] {
            let arg_ty = self.unit.value_type(arg);
            if arg_ty != ty {
                mismatch = true;
            }
        }
        if mismatch {
            let tys: Vec<_> = self.unit()[inst]
                .args()
                .into_iter()
                .map(|&arg| self.unit.value_type(arg).to_string())
                .collect();
            let tys: String = tys.join(", ");
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("argument types must match (but are {})", tys),
            });
        }
    }

    /// Verify that the type of aninstruction's argument is a signal.
    fn verify_arg_ty_is_signal(&mut self, inst: Inst, arg: Value) {
        let ty = self.unit.value_type(arg);
        if !ty.is_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("argument {} type must be a signal (but is {})", arg, ty),
            });
        }
    }

    /// Verify that the types of an instruction's arguments match the return
    /// type of the instruction itself.
    fn verify_args_match_inst_ty(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        self.verify_args_match_ty(inst, &ty);
    }

    /// Verify that the types of an instruction's arguments match a given type.
    fn verify_args_match_ty(&mut self, inst: Inst, ty: &Type) {
        for &arg in self.unit()[inst].args() {
            self.verify_arg_matches_ty(inst, arg, ty);
        }
    }

    /// Verify that the type of an instruction's argument matches a given type.
    fn verify_arg_matches_ty(&mut self, inst: Inst, arg: Value, ty: &Type) {
        let arg_ty = self.unit.value_type(arg);
        if arg_ty != *ty {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "argument {} must be of type {} (but is {})",
                    arg, ty, arg_ty,
                ),
            });
        }
    }

    /// Verify that an instruction's return type is `i1`.
    fn verify_bool_ty(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        if ty.is_int() && ty.unwrap_int() == 1 {
            return;
        }
        self.verifier.errors.push(VerifierError {
            unit: self.verifier.unit_name.clone(),
            object: Some(inst.dump(&self.unit).to_string()),
            message: format!("return type must be i1 (but is {})", ty),
        });
    }

    /// Verify that an instruction's return type is compatible with bitwise
    /// operations.
    fn verify_bitwise_compatible_ty(&mut self, inst: Inst) {
        self.verify_arith_compatible_ty(inst);
    }

    /// Verify that an instruction's return type is compatible with arithmetic
    /// operations.
    fn verify_arith_compatible_ty(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        if ty.is_int() {
            return;
        }
        if ty.is_signal() && ty.unwrap_signal().is_int() {
            return;
        }
        self.verifier.errors.push(VerifierError {
            unit: self.verifier.unit_name.clone(),
            object: Some(inst.dump(&self.unit).to_string()),
            message: format!("return type must be iN or iN$ (but is {})", ty),
        });
    }

    /// Verify that an instruction produces a result of a given type.
    fn verify_inst_ty(&mut self, inst: Inst, ty: &Type) {
        let inst_ty = self.unit.inst_type(inst);
        if inst_ty != *ty {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("return type must be {} (but is {})", ty, inst_ty),
            });
        }
    }

    /// Verify that the types of a shift instruction line up.
    fn verify_shift_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        let base = self.unit()[inst].args()[0];
        let hidden = self.unit()[inst].args()[1];
        let amount = self.unit()[inst].args()[2];
        self.verify_arg_matches_ty(inst, base, &ty);
        let amount_ty = self.unit.value_type(amount);
        if !amount_ty.is_int() && !(amount_ty.is_signal() && amount_ty.unwrap_signal().is_int()) {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "type of shift amount must be iN or iN$ (but is {})",
                    amount_ty
                ),
            });
        }
        let base_ty = self.unit.value_type(base);
        let hidden_ty = self.unit.value_type(hidden);
        if base_ty.is_signal() != hidden_ty.is_signal()
            || base_ty.is_pointer() != hidden_ty.is_pointer()
        {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
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
        self.verifier.errors.push(VerifierError {
            unit: self.verifier.unit_name.clone(),
            object: Some(inst.dump(&self.unit).to_string()),
            message: format!(
                "shift base and hidden value types must be compatible (but are {} and {})",
                base_ty, hidden_ty
            ),
        });
    }

    /// Verify that the types of a mux instruction line up.
    fn verify_mux_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        let array = self.unit()[inst].args()[0];
        let array_ty = self.unit.value_type(array);
        if !array_ty.is_array() || array_ty.unwrap_array().1 != &ty {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "array {} element and return type {} must agree",
                    array_ty, ty
                ),
            });
        }
        let sel = self.unit()[inst].args()[1];
        let sel_ty = self.unit.value_type(sel);
        if !sel_ty.is_int() && !(sel_ty.is_signal() && sel_ty.unwrap_signal().is_int()) {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type of selector must be iN or iN$ (but is {})", sel_ty),
            });
        }
    }

    /// Verify that the types of a reg instruction line up.
    fn verify_reg_inst(&mut self, inst: Inst) {
        self.verify_arg_ty_is_signal(inst, self.unit()[inst].args()[0]);
        let ty = self
            .unit
            .value_type(self.unit()[inst].args()[0])
            .unwrap_signal()
            .clone();
        for arg in self.unit()[inst].data_args() {
            self.verify_arg_matches_ty(inst, arg, &ty);
        }
        for arg in self.unit()[inst].trigger_args() {
            if self.unit.value_type(arg).is_signal() {
                self.verify_arg_matches_ty(inst, arg, &signal_ty(int_ty(1)));
            } else {
                self.verify_arg_matches_ty(inst, arg, &int_ty(1));
            }
        }
        for arg in self.unit()[inst].gating_args() {
            if let Some(arg) = arg {
                self.verify_arg_matches_ty(inst, arg, &int_ty(1));
            }
        }
    }

    /// Determine the field type for an insf/extf instruction.
    fn find_insext_field_type(&mut self, inst: Inst, allow_deref: bool) -> Option<Type> {
        let field = self.unit()[inst].imms()[0];
        let target_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        let (target_ty, wrap): (_, &dyn Fn(Type) -> Type) = if allow_deref && target_ty.is_signal()
        {
            (target_ty.unwrap_signal(), &signal_ty)
        } else if allow_deref && target_ty.is_pointer() {
            (target_ty.unwrap_pointer(), &pointer_ty)
        } else {
            (&target_ty, &identity)
        };
        let ty = if target_ty.is_struct() {
            match target_ty.unwrap_struct().get(field) {
                Some(ty) => Some(ty.clone()),
                None => {
                    self.verifier.errors.push(VerifierError {
                        unit: self.verifier.unit_name.clone(),
                        object: Some(inst.dump(&self.unit).to_string()),
                        message: format!(
                            "field index {} out of bounds of struct type {}",
                            field, target_ty
                        ),
                    });
                    None
                }
            }
        } else if target_ty.is_array() {
            Some(target_ty.unwrap_array().1.clone())
        } else {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "target must be of struct or array type (but is {})",
                    target_ty
                ),
            });
            None
        };
        ty.map(wrap)
    }

    /// Determine the field type for an inss/exts instruction.
    fn find_insext_slice_type(&mut self, inst: Inst, allow_deref: bool) -> Option<Type> {
        let offset = self.unit()[inst].imms()[0];
        let length = self.unit()[inst].imms()[1];
        let target_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        let (target_ty, wrap): (_, &dyn Fn(Type) -> Type) = if allow_deref && target_ty.is_signal()
        {
            (target_ty.unwrap_signal(), &signal_ty)
        } else if allow_deref && target_ty.is_pointer() {
            (target_ty.unwrap_pointer(), &pointer_ty)
        } else {
            (&target_ty, &identity)
        };
        let ty = if target_ty.is_array() {
            let (array_len, elem_ty) = target_ty.unwrap_array();
            if array_len < offset + length {
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&self.unit).to_string()),
                    message: format!(
                        "access {}..{} out of array bounds 0..{}",
                        offset,
                        offset + length,
                        array_len
                    ),
                });
            }
            Some(array_ty(length, elem_ty.clone()))
        } else if target_ty.is_int() {
            let size = target_ty.unwrap_int();
            if size < offset + length {
                self.verifier.errors.push(VerifierError {
                    unit: self.verifier.unit_name.clone(),
                    object: Some(inst.dump(&self.unit).to_string()),
                    message: format!(
                        "access {}..{} out of integer bounds 0..{}",
                        offset,
                        offset + length,
                        size
                    ),
                });
            }
            Some(int_ty(length))
        } else {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("target must be of array or iN type (but is {})", target_ty),
            });
            None
        };
        ty.map(wrap)
    }

    /// Verify that the types of an insf instruction line up.
    fn verify_ins_field_inst(&mut self, inst: Inst) {
        let target_ty = self.unit.inst_type(inst);
        self.verify_arg_matches_ty(inst, self.unit()[inst].args()[0], &target_ty);
        let arg_ty = match self.find_insext_field_type(inst, false) {
            Some(ty) => ty,
            None => return,
        };
        self.verify_arg_matches_ty(inst, self.unit()[inst].args()[1], &arg_ty);
    }

    /// Verify that the types of an extf instruction line up.
    fn verify_ext_field_inst(&mut self, inst: Inst) {
        let arg_ty = match self.find_insext_field_type(inst, true) {
            Some(ty) => ty,
            None => return,
        };
        self.verify_inst_ty(inst, &arg_ty);
    }

    /// Verify that the types of an inss instruction line up.
    fn verify_ins_slice_inst(&mut self, inst: Inst) {
        let target_ty = self.unit.inst_type(inst);
        self.verify_arg_matches_ty(inst, self.unit()[inst].args()[0], &target_ty);
        let arg_ty = match self.find_insext_slice_type(inst, false) {
            Some(ty) => ty,
            None => return,
        };
        self.verify_arg_matches_ty(inst, self.unit()[inst].args()[1], &arg_ty);
    }

    /// Verify that the types of an exts instruction line up.
    fn verify_ext_slice_inst(&mut self, inst: Inst) {
        let arg_ty = match self.find_insext_slice_type(inst, true) {
            Some(ty) => ty,
            None => return,
        };
        self.verify_inst_ty(inst, &arg_ty);
    }

    /// Verify that the types of a sig instruction line up.
    fn verify_sig_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        if !ty.is_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a signal", ty),
            });
        }
        self.verify_args_match_ty(inst, ty.unwrap_signal());
    }

    /// Verify that the types of a prb instruction line up.
    fn verify_prb_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        let arg_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        if !arg_ty.is_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a signal", ty),
            });
        }
        if ty != *arg_ty.unwrap_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be signal of return type {}", arg_ty, ty),
            });
        }
    }

    /// Verify that the types of a drv instruction line up.
    fn verify_drv_inst(&mut self, inst: Inst) {
        let ty = self.unit.value_type(self.unit()[inst].args()[1]);
        let arg_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        if !arg_ty.is_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a signal", ty),
            });
        }
        if ty != *arg_ty.unwrap_signal() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "drive target type {} must be signal of driven value type {}",
                    arg_ty, ty
                ),
            });
        }
        self.verify_arg_matches_ty(inst, self.unit()[inst].args()[2], &time_ty());
        if self.unit()[inst].opcode() == Opcode::DrvCond {
            self.verify_arg_matches_ty(inst, self.unit()[inst].args()[3], &int_ty(1));
        }
    }

    /// Verify that the types of a var instruction line up.
    fn verify_var_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        if !ty.is_pointer() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a pointer", ty),
            });
        }
        self.verify_args_match_ty(inst, ty.unwrap_pointer());
    }

    /// Verify that the types of a ld instruction line up.
    fn verify_ld_inst(&mut self, inst: Inst) {
        let ty = self.unit.inst_type(inst);
        let arg_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        if !arg_ty.is_pointer() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a pointer", ty),
            });
        }
        if ty != *arg_ty.unwrap_pointer() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be pointer of return type {}", arg_ty, ty),
            });
        }
    }

    /// Verify that the types of a st instruction line up.
    fn verify_st_inst(&mut self, inst: Inst) {
        let ty = self.unit.value_type(self.unit()[inst].args()[1]);
        let arg_ty = self.unit.value_type(self.unit()[inst].args()[0]);
        if !arg_ty.is_pointer() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!("type {} must be a pointer", ty),
            });
        }
        if ty != *arg_ty.unwrap_pointer() {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "store target type {} must be pointer of stored value type {}",
                    arg_ty, ty
                ),
            });
        }
    }

    /// Verify that the return type of the enclosing function is compatible with
    /// a ret instruction.
    fn verify_return_type(&mut self, inst: Inst, ty: &Type) {
        let func_ty = self.return_type.clone().unwrap_or_else(void_ty);
        if func_ty != *ty {
            self.verifier.errors.push(VerifierError {
                unit: self.verifier.unit_name.clone(),
                object: Some(inst.dump(&self.unit).to_string()),
                message: format!(
                    "requires function to have return type {} (but has {})",
                    ty, func_ty
                ),
            });
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

fn identity(ty: Type) -> Type {
    ty
}
