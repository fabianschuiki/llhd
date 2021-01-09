// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of LLHD instructions.
//!
//! This module implements the various instructions of the intermediate
//! representation.

use crate::{
    ir::{Block, ExtUnit, Inst, Unit, UnitBuilder, Value},
    ty::{array_ty, int_ty, pointer_ty, signal_ty, struct_ty, void_ty, Type},
    value::{IntValue, TimeValue},
};
use bitflags::bitflags;
use std::borrow::Cow;

/// A temporary object used to construct a single instruction.
pub struct InstBuilder<'a, 'b> {
    builder: &'b mut UnitBuilder<'a>,
    name: Option<String>,
}

impl<'a, 'b> InstBuilder<'a, 'b> {
    /// Create a new instruction builder that inserts into `builder`.
    pub fn new(builder: &'b mut UnitBuilder<'a>) -> Self {
        Self {
            builder,
            name: None,
        }
    }

    /// Assign a name to the instruction being built.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl<'a, 'b> InstBuilder<'a, 'b> {
    /// Construct the zero value for a type.
    ///
    /// This is a convenience function that creates the appropriate instruction
    /// sequence to generate the given zero value. Note that arrays and structs
    /// emit multiple instructions.
    pub fn const_zero(&mut self, ty: &Type) -> Value {
        use crate::ty::TypeKind::*;
        match ty.as_ref() {
            TimeType => self.const_time(TimeValue::zero()),
            IntType(w) => self.const_int(IntValue::zero(*w)),
            ArrayType(l, ty) => {
                let name = self.name.take();
                let elem = self.const_zero(ty);
                self.name = name;
                self.array_uniform(*l, elem)
            }
            StructType(tys) => {
                let name = self.name.take();
                let elems = tys.iter().map(|ty| self.const_zero(ty)).collect();
                self.name = name;
                self.strukt(elems)
            }
            _ => panic!("no zero value for {}", ty),
        }
    }

    /// Construct the given value for int type.
    ///
    /// This is a convenience function that creates the appropriate instruction
    /// sequence to generate the given value for int type.
    pub fn const_int(&mut self, value: impl Into<IntValue>) -> Value {
        let value = value.into();
        let ty = value.ty();
        let data = InstData::ConstInt {
            opcode: Opcode::ConstInt,
            imm: value,
        };
        let inst = self.build(data, ty);
        self.inst_result(inst)
    }

    /// Construct the given value for time type.
    ///
    /// This is a convenience function that creates the appropriate instruction
    /// sequence to generate the given value for int type.
    pub fn const_time(&mut self, value: impl Into<TimeValue>) -> Value {
        let value = value.into();
        let ty = value.ty();
        let data = InstData::ConstTime {
            opcode: Opcode::ConstTime,
            imm: value,
        };
        let inst = self.build(data, ty);
        self.inst_result(inst)
    }

    /// Creates alias instruction to assign a new name to a value.
    pub fn alias(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_unary(Opcode::Alias, ty, x);
        self.inst_result(inst)
    }

    /// Creates array instruction to generate array of a given size
    /// with all elements of given value.
    pub fn array_uniform(&mut self, imm: usize, x: Value) -> Value {
        let ty = array_ty(imm, self.value_type(x));
        let inst = self.build(
            InstData::Array {
                opcode: Opcode::ArrayUniform,
                imms: [imm],
                args: [x],
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates array instruction to generate array from a vector of similar type of Values.
    pub fn array(&mut self, args: Vec<Value>) -> Value {
        assert!(!args.is_empty());
        let ty = array_ty(args.len(), self.value_type(args[0]));
        let inst = self.build(
            InstData::Aggregate {
                opcode: Opcode::Array,
                args: args,
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates struct instruction to generate struct from a vector of different types of Values.
    pub fn strukt(&mut self, args: Vec<Value>) -> Value {
        let ty = struct_ty(
            args.iter()
                .cloned()
                .map(|arg| self.value_type(arg))
                .collect(),
        );
        let inst = self.build(
            InstData::Aggregate {
                opcode: Opcode::Struct,
                args: args,
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates not instruction to generate inverse of a given Value.
    pub fn not(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_unary(Opcode::Not, ty, x);
        self.inst_result(inst)
    }

    /// Creates neg instruction to compute two's compliment of the given Value.
    pub fn neg(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_unary(Opcode::Neg, ty, x);
        self.inst_result(inst)
    }

    /// Creates add instruction to sum two given Value's.
    pub fn add(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Add, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates sub instruction to substract the two given Value's.
    pub fn sub(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Sub, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates and instruction to compute bitwise AND of the given two Value's.
    pub fn and(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::And, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates or instruction to compute bitwise OR of the given two Value's.
    pub fn or(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Or, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates xor instruction to compute bitwise XOR of the given two Value's.
    pub fn xor(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Xor, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates smul instruction to compute signed binary multiplication.
    pub fn smul(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Smul, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates sdiv instruction to compute signed binary division
    pub fn sdiv(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Sdiv, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates smod instruction to compute signed binary modulus of a given Value
    /// when divided by the other
    pub fn smod(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Smod, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates srem instruction to compute signed binary reminder of the given Value
    /// when divided by the other
    pub fn srem(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Srem, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates smul instruction to compute unsigned binary multiplication
    pub fn umul(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Umul, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates sdiv instruction to compute unsigned binary division of a given Value
    /// with the other
    pub fn udiv(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Udiv, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates umod instruction to compute unsigned binary modulus of a given Value
    /// when divided by the other
    pub fn umod(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Umod, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates urem instruction to compute unsigned binary reminder of the given Value
    /// when divided by the other
    pub fn urem(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_binary(Opcode::Urem, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates eq instruction to check for equality of the given Value's
    pub fn eq(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Eq, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates neq instruction to check for unequality of the given Value's
    pub fn neq(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Neq, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates slt instruction to check if a given Value, as signed, is less than the other
    pub fn slt(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Slt, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates sgt instruction to check if a given Value, as signed, is greater than the other
    pub fn sgt(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Sgt, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates sle instruction to check if a given Value, as signed, is less than or equal
    /// to the other
    pub fn sle(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Sle, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates sge instruction to check if a given Value, as signed, is greater than or
    /// equal to the other
    pub fn sge(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Sge, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates ult instruction to check if a given Value, as unsigned, is less than the other
    pub fn ult(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Ult, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates ugt instruction to check if a given Value, as unsigned, is greater than the other
    pub fn ugt(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Ugt, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates ule instruction to check if a given Value, as unsigned, is less than or equal
    /// to the other
    pub fn ule(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Ule, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates uge instruction to check if a given Value, as unsigned, is greater than or
    /// equal the other
    pub fn uge(&mut self, x: Value, y: Value) -> Value {
        let inst = self.build_binary(Opcode::Uge, int_ty(1), x, y);
        self.inst_result(inst)
    }

    /// Creates shl instruction to shift a given Value to the left by the given amount
    /// from a hidden Value
    pub fn shl(&mut self, x: Value, y: Value, z: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_ternary(Opcode::Shl, ty, x, y, z);
        self.inst_result(inst)
    }

    /// Creates shr instruction to shift a given Value to the right by the given amount
    /// from a hidden Value
    pub fn shr(&mut self, x: Value, y: Value, z: Value) -> Value {
        let ty = self.value_type(x);
        let inst = self.build_ternary(Opcode::Shr, ty, x, y, z);
        self.inst_result(inst)
    }

    /// Creates mux instruction to choose a Value from a given array of Values
    /// based on a given selector Value
    pub fn mux(&mut self, x: Value, y: Value) -> Value {
        let ty = self.value_type(x);
        assert!(ty.is_array(), "argument to `mux` must be of array type");
        let ty = ty.unwrap_array().1.clone();
        let inst = self.build_binary(Opcode::Mux, ty, x, y);
        self.inst_result(inst)
    }

    /// Creates reg instruction to provide a storage element which drives it output onto a signal
    pub fn reg(&mut self, x: Value, data: Vec<RegTrigger>) -> Inst {
        let mut args = vec![x];
        let mut modes = vec![];
        args.extend(data.iter().map(|x| x.data));
        args.extend(data.iter().map(|x| x.trigger));
        args.extend(data.iter().map(|x| x.gate.unwrap_or(Value::invalid())));
        modes.extend(data.iter().map(|x| x.mode));
        assert_eq!(args.len(), modes.len() * 3 + 1);
        self.build(
            InstData::Reg {
                opcode: Opcode::Reg,
                args,
                modes,
            },
            void_ty(),
        )
    }

    /// Creates insf instruction to insert a field or element or bit
    /// in a given array or struct or integer, respectively, at an index
    pub fn ins_field(&mut self, x: Value, y: Value, imm: usize) -> Value {
        let ty = self.value_type(x);
        let inst = self.build(
            InstData::InsExt {
                opcode: Opcode::InsField,
                args: [x, y],
                imms: [imm, 0],
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates inss instruction to insert a slice of array element or
    /// integer bits in a given array or integer, respectively, in a range of indexes
    pub fn ins_slice(&mut self, x: Value, y: Value, imm0: usize, imm1: usize) -> Value {
        let ty = self.value_type(x);
        let inst = self.build(
            InstData::InsExt {
                opcode: Opcode::InsSlice,
                args: [x, y],
                imms: [imm0, imm1],
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates extf instruction to extract a field or element or bit
    /// from a given array or struct or integer, respectively, at an index
    pub fn ext_field(&mut self, x: Value, imm: usize) -> Value {
        let ty = with_unpacked_sigptr(self.value_type(x), |ty| {
            if ty.is_struct() {
                let fields = ty.unwrap_struct();
                assert!(imm < fields.len(), "field index in `extf` out of range");
                fields[imm].clone()
            } else if ty.is_array() {
                ty.unwrap_array().1.clone()
            } else {
                panic!("argument to `extf` must be of struct or array type");
            }
        });
        let inst = self.build(
            InstData::InsExt {
                opcode: Opcode::ExtField,
                args: [x, Value::invalid()],
                imms: [imm, 0],
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates exts instruction to extract a slice of array element or
    /// integer bits from a given array or integer, respectively, in a range of indexes
    pub fn ext_slice(&mut self, x: Value, imm0: usize, imm1: usize) -> Value {
        let ty = with_unpacked_sigptr(self.value_type(x), |ty| {
            if ty.is_array() {
                array_ty(imm1, ty.unwrap_array().1.clone())
            } else if ty.is_int() {
                int_ty(imm1)
            } else {
                panic!("argument to `exts` must be of array or integer type");
            }
        });
        let inst = self.build(
            InstData::InsExt {
                opcode: Opcode::ExtSlice,
                args: [x, Value::invalid()],
                imms: [imm0, imm1],
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates con instruction to connect two signals
    pub fn con(&mut self, x: Value, y: Value) -> Inst {
        self.build_binary(Opcode::Con, void_ty(), x, y)
    }

    /// Creates del instruction to delay a signal source by a delay
    pub fn del(&mut self, target: Value, source: Value, delay: Value) -> Inst {
        self.build_ternary(Opcode::Del, void_ty(), target, source, delay)
    }

    /// Creates call instruction to transfer control to a function and yield its return value
    pub fn call(&mut self, unit: ExtUnit, args: Vec<Value>) -> Value {
        let ty = self.builder.extern_sig(unit).return_type();
        let inst = self.build(
            InstData::Call {
                opcode: Opcode::Call,
                unit,
                ins: args.len() as u16,
                args,
            },
            ty,
        );
        self.inst_result(inst)
    }

    /// Creates inst instruction to instantiates a proces or entity within the current one
    pub fn inst(&mut self, unit: ExtUnit, mut inputs: Vec<Value>, outputs: Vec<Value>) -> Inst {
        let ins = inputs.len() as u16;
        inputs.extend(outputs);
        let data = InstData::Call {
            opcode: Opcode::Inst,
            unit,
            ins,
            args: inputs,
        };
        self.build(data, void_ty())
    }

    /// Creates sig instruction to allocate a signal
    pub fn sig(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        let ty = if ty.is_signal() { ty } else { signal_ty(ty) };
        let inst = self.build_unary(Opcode::Sig, ty, x);
        self.inst_result(inst)
    }

    /// Creates prb instruction to probe the current value of a signal
    pub fn prb(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        assert!(ty.is_signal(), "argument to `prb` must be of signal type");
        let ty = ty.unwrap_signal().clone();
        let inst = self.build_unary(Opcode::Prb, ty, x);
        self.inst_result(inst)
    }

    /// Creates drv instruction to shedule signal value to change after a delay
    pub fn drv(&mut self, signal: Value, value: Value, delay: Value) -> Inst {
        self.build_ternary(Opcode::Drv, void_ty(), signal, value, delay)
    }

    /// Creates drv_cond instruction to shedule signal value to change after a delay
    /// if given condition is satisfied
    pub fn drv_cond(&mut self, signal: Value, value: Value, delay: Value, cond: Value) -> Inst {
        self.build_quaternary(Opcode::DrvCond, void_ty(), signal, value, delay, cond)
    }

    /// Creates var instruction to allocate memory on stack with the initial value
    /// and returns a pointer to that location
    pub fn var(&mut self, x: Value) -> Value {
        let ty = pointer_ty(self.value_type(x));
        let inst = self.build_unary(Opcode::Var, ty, x);
        self.inst_result(inst)
    }

    /// Creates ld instruction to load a value from a memory location pointer
    pub fn ld(&mut self, x: Value) -> Value {
        let ty = self.value_type(x);
        assert!(ty.is_pointer(), "argument to `ld` must be of pointer type");
        let ty = ty.unwrap_pointer().clone();
        let inst = self.build_unary(Opcode::Ld, ty, x);
        self.inst_result(inst)
    }

    /// Creates st instruction to store a value to a memory location pointer
    pub fn st(&mut self, x: Value, y: Value) -> Inst {
        self.build_binary(Opcode::St, void_ty(), x, y)
    }

    /// Creates halt instruction to terminate an execution of a process
    pub fn halt(&mut self) -> Inst {
        self.build_nullary(Opcode::Halt)
    }

    /// Creates ret instruction to return from a void function
    pub fn ret(&mut self) -> Inst {
        self.build_nullary(Opcode::Ret)
    }

    /// Creates ret instruction to return from a void function and returns a value
    pub fn ret_value(&mut self, x: Value) -> Inst {
        self.build_unary(Opcode::RetValue, void_ty(), x)
    }

    /// Creates phi instruction to implement phi node in SSA graph representing the function
    /// or process
    pub fn phi(&mut self, args: Vec<Value>, bbs: Vec<Block>) -> Value {
        assert!(args.len() > 0);
        assert_eq!(args.len(), bbs.len());
        let ty = self.value_type(args[0]);
        let data = InstData::Phi {
            opcode: Opcode::Phi,
            args,
            bbs,
        };
        let inst = self.build(data, ty);
        self.inst_result(inst)
    }

    /// Creates br instruction to transfer control to another basic block
    pub fn br(&mut self, bb: Block) -> Inst {
        let data = InstData::Jump {
            opcode: Opcode::Br,
            bbs: [bb],
        };
        self.build(data, void_ty())
    }

    /// Creates br instruction to transfer control to another basic block, between two blocks,
    /// based on the given condition
    pub fn br_cond(&mut self, x: Value, bb0: Block, bb1: Block) -> Inst {
        let data = InstData::Branch {
            opcode: Opcode::BrCond,
            args: [x],
            bbs: [bb0, bb1],
        };
        self.build(data, void_ty())
    }

    /// Creates wait instruction to suspend execution of a process until
    /// any of the observed signals change
    pub fn wait(&mut self, bb: Block, args: Vec<Value>) -> Inst {
        let data = InstData::Wait {
            opcode: Opcode::Wait,
            bbs: [bb],
            args: args,
        };
        self.build(data, void_ty())
    }

    /// Creates wait instruction to suspend execution of a process until
    /// any of the observed signals change or a fixed time interval has passed
    pub fn wait_time(&mut self, bb: Block, time: Value, mut args: Vec<Value>) -> Inst {
        args.insert(0, time);
        let data = InstData::Wait {
            opcode: Opcode::WaitTime,
            bbs: [bb],
            args: args,
        };
        self.build(data, void_ty())
    }
}

/// Convenience functions to construct the different instruction formats.
impl<'a, 'b> InstBuilder<'a, 'b> {
    /// `opcode`
    fn build_nullary(&mut self, opcode: Opcode) -> Inst {
        let data = InstData::Nullary { opcode };
        self.build(data, void_ty())
    }

    /// `a = opcode type x`
    fn build_unary(&mut self, opcode: Opcode, ty: Type, x: Value) -> Inst {
        let data = InstData::Unary { opcode, args: [x] };
        self.build(data, ty)
    }

    /// `a = opcode type x, y`
    fn build_binary(&mut self, opcode: Opcode, ty: Type, x: Value, y: Value) -> Inst {
        let data = InstData::Binary {
            opcode,
            args: [x, y],
        };
        self.build(data, ty)
    }

    /// `a = opcode type x, y, z`
    fn build_ternary(&mut self, opcode: Opcode, ty: Type, x: Value, y: Value, z: Value) -> Inst {
        let data = InstData::Ternary {
            opcode,
            args: [x, y, z],
        };
        self.build(data, ty)
    }

    /// `a = opcode type x, y, z, w`
    fn build_quaternary(
        &mut self,
        opcode: Opcode,
        ty: Type,
        x: Value,
        y: Value,
        z: Value,
        w: Value,
    ) -> Inst {
        let data = InstData::Quaternary {
            opcode,
            args: [x, y, z, w],
        };
        self.build(data, ty)
    }
}

/// Fundamental convenience forwards to the wrapped builder.
impl<'a, 'b> InstBuilder<'a, 'b> {
    /// Convenience forward to `UnitBuilder`.
    pub(crate) fn build(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.builder.build_inst(data, ty);
        if let Some(name) = self.name.take() {
            if let Some(value) = self.builder.get_inst_result(inst) {
                self.builder.set_name(value, name);
            }
        }
        inst
    }

    /// Convenience forward to `Unit`.
    fn value_type(&self, value: Value) -> Type {
        self.builder.value_type(value)
    }

    /// Convenience forward to `Unit`.
    fn inst_result(&self, inst: Inst) -> Value {
        self.builder.inst_result(inst)
    }

    /// Assign another value's name plus a suffix to the instruction being
    /// built.
    ///
    /// If `value` has a name, the instruction's name will be
    /// `<value>.<suffix>`. Otherwise it will just be `<suffix>`.
    pub fn suffix<'c>(mut self, value: Value, suffix: impl Into<Cow<'c, str>>) -> Self {
        let suffix = suffix.into(); // moooh
        self.name = if let Some(name) = self.builder.get_name(value) {
            Some(format!("{}.{}", name, suffix))
        } else {
            Some(suffix.into_owned())
        };
        self
    }
}

/// An instruction format.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstData {
    /// `a = const iN imm`
    ConstInt { opcode: Opcode, imm: IntValue },
    /// `a = const time imm`
    ConstTime { opcode: Opcode, imm: TimeValue },
    /// `opcode imm, type x`
    Array {
        opcode: Opcode,
        imms: [usize; 1],
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
    /// `opcode type x, y, z, w`
    Quaternary { opcode: Opcode, args: [Value; 4] },
    /// `opcode bb`
    Jump { opcode: Opcode, bbs: [Block; 1] },
    /// `opcode type [x, bb],*`
    Phi {
        opcode: Opcode,
        args: Vec<Value>,
        bbs: Vec<Block>,
    },
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
    /// `a = reg type x (, data mode trigger)*`
    Reg {
        opcode: Opcode,
        args: Vec<Value>,
        modes: Vec<RegMode>,
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
            InstData::Quaternary { opcode, .. } => opcode,
            InstData::Phi { opcode, .. } => opcode,
            InstData::Jump { opcode, .. } => opcode,
            InstData::Branch { opcode, .. } => opcode,
            InstData::Wait { opcode, .. } => opcode,
            InstData::Call { opcode, .. } => opcode,
            InstData::InsExt { opcode, .. } => opcode,
            InstData::Reg { opcode, .. } => opcode,
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
            InstData::Quaternary { args, .. } => args,
            InstData::Phi { args, .. } => args,
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
            InstData::Reg { args, .. } => args,
        }
    }

    /// Mutable access to the arguments of an instruction.
    #[deprecated = "do not use directly"]
    pub(crate) fn args_mut(&mut self) -> &mut [Value] {
        match self {
            InstData::ConstInt { .. } => &mut [],
            InstData::ConstTime { .. } => &mut [],
            InstData::Array { args, .. } => args,
            InstData::Aggregate { args, .. } => args,
            InstData::Nullary { .. } => &mut [],
            InstData::Unary { args, .. } => args,
            InstData::Binary { args, .. } => args,
            InstData::Ternary { args, .. } => args,
            InstData::Quaternary { args, .. } => args,
            InstData::Phi { args, .. } => args,
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
            InstData::Reg { args, .. } => args,
        }
    }

    /// Get the immediates of an instruction.
    pub fn imms(&self) -> &[usize] {
        match self {
            InstData::ConstInt { .. } => &[],
            InstData::ConstTime { .. } => &[],
            InstData::Array { imms, .. } => imms,
            InstData::Aggregate { .. } => &[],
            InstData::Nullary { .. } => &[],
            InstData::Unary { .. } => &[],
            InstData::Binary { .. } => &[],
            InstData::Ternary { .. } => &[],
            InstData::Quaternary { .. } => &[],
            InstData::Phi { .. } => &[],
            InstData::Jump { .. } => &[],
            InstData::Branch { .. } => &[],
            InstData::Wait { .. } => &[],
            InstData::Call { .. } => &[],
            InstData::InsExt {
                opcode: Opcode::InsField,
                imms,
                ..
            }
            | InstData::InsExt {
                opcode: Opcode::ExtField,
                imms,
                ..
            } => &imms[0..1],
            InstData::InsExt { imms, .. } => imms,
            InstData::Reg { .. } => &[],
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

    /// Get the data arguments of a register instruction.
    pub fn data_args(&self) -> impl Iterator<Item = Value> + '_ {
        match self {
            InstData::Reg { args, modes, .. } => &args[1..1 + modes.len()],
            _ => &[],
        }
        .iter()
        .cloned()
    }

    /// Get the trigger arguments of a register instruction.
    pub fn trigger_args(&self) -> impl Iterator<Item = Value> + '_ {
        match self {
            InstData::Reg { args, modes, .. } => &args[1 + modes.len()..1 + 2 * modes.len()],
            _ => &[],
        }
        .iter()
        .cloned()
    }

    /// Get the gating arguments of a register instruction.
    pub fn gating_args(&self) -> impl Iterator<Item = Option<Value>> + '_ {
        match self {
            InstData::Reg { args, modes, .. } => &args[1 + 2 * modes.len()..],
            _ => &[],
        }
        .iter()
        .map(|&v| if v == Value::invalid() { None } else { Some(v) })
    }

    /// Get the modes of a register instruction.
    pub fn mode_args(&self) -> impl Iterator<Item = RegMode> + '_ {
        match self {
            InstData::Reg { modes, .. } => modes.as_slice(),
            _ => &[],
        }
        .iter()
        .cloned()
    }

    /// Get the register triggers.
    pub fn triggers(&self) -> impl Iterator<Item = RegTrigger> + '_ {
        self.data_args()
            .zip(self.mode_args())
            .zip(self.trigger_args())
            .zip(self.gating_args())
            .map(|(((data, mode), trigger), gate)| RegTrigger {
                data,
                mode,
                trigger,
                gate,
            })
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
            InstData::Quaternary { .. } => &[],
            InstData::Phi { bbs, .. } => bbs,
            InstData::Jump { bbs, .. } => bbs,
            InstData::Branch { bbs, .. } => bbs,
            InstData::Wait { bbs, .. } => bbs,
            InstData::Call { .. } => &[],
            InstData::InsExt { .. } => &[],
            InstData::Reg { .. } => &[],
        }
    }

    /// Mutable access to the BBs of an instruction.
    #[deprecated = "do not use directly"]
    pub(crate) fn blocks_mut(&mut self) -> &mut [Block] {
        match self {
            InstData::ConstInt { .. } => &mut [],
            InstData::ConstTime { .. } => &mut [],
            InstData::Array { .. } => &mut [],
            InstData::Aggregate { .. } => &mut [],
            InstData::Nullary { .. } => &mut [],
            InstData::Unary { .. } => &mut [],
            InstData::Binary { .. } => &mut [],
            InstData::Ternary { .. } => &mut [],
            InstData::Quaternary { .. } => &mut [],
            InstData::Phi { bbs, .. } => bbs,
            InstData::Jump { bbs, .. } => bbs,
            InstData::Branch { bbs, .. } => bbs,
            InstData::Wait { bbs, .. } => bbs,
            InstData::Call { .. } => &mut [],
            InstData::InsExt { .. } => &mut [],
            InstData::Reg { .. } => &mut [],
        }
    }

    /// Replace all uses of a value with another.
    #[deprecated = "use DataFlowGraph::replace_value_within_inst instead"]
    pub(crate) fn replace_value(&mut self, from: Value, to: Value) -> usize {
        let mut count = 0;
        #[allow(deprecated)]
        for arg in self.args_mut() {
            if *arg == from {
                *arg = to;
                count += 1;
            }
        }
        count
    }

    /// Replace all uses of a block with another.
    #[deprecated = "use DataFlowGraph::replace_block_within_inst instead"]
    pub(crate) fn replace_block(&mut self, from: Block, to: Block) -> usize {
        let mut count = 0;
        #[allow(deprecated)]
        for bb in self.blocks_mut() {
            if *bb == from {
                *bb = to;
                count += 1;
            }
        }
        count
    }

    /// Remove all uses of a block.
    #[deprecated = "use DataFlowGraph::remove_block_from_inst instead"]
    pub(crate) fn remove_block(&mut self, block: Block) -> usize {
        match self {
            InstData::Phi { bbs, args, .. } => {
                let mut count = 0;
                for i in 0..bbs.len() {
                    if i >= bbs.len() {
                        break;
                    }
                    if bbs[i] == block {
                        bbs.swap_remove(i);
                        args.swap_remove(i);
                        count += 1;
                    }
                }
                count
            }
            #[allow(deprecated)]
            _ => self.replace_block(block, Block::invalid()),
        }
    }

    /// Return the const int constructed by this instruction.
    pub fn get_const_int(&self) -> Option<&IntValue> {
        match self {
            InstData::ConstInt { imm, .. } => Some(imm),
            _ => None,
        }
    }

    /// Return the const time constructed by this instruction.
    pub fn get_const_time(&self) -> Option<&TimeValue> {
        match self {
            InstData::ConstTime { imm, .. } => Some(imm),
            _ => None,
        }
    }

    /// Return the external unit being called or instantiated by this
    /// instruction.
    pub fn get_ext_unit(&self) -> Option<ExtUnit> {
        match self {
            InstData::Call { unit, .. } => Some(*unit),
            _ => None,
        }
    }
}

impl Default for InstData {
    fn default() -> InstData {
        InstData::Nullary {
            opcode: Opcode::Ret,
        }
    }
}

bitflags! {
    /// A set of flags identifying a unit.
    #[derive(Default, Serialize, Deserialize)]
    pub struct UnitFlags: u8 {
        /// UnitFlag for a FUNCTION
        const FUNCTION = 0b001;
        /// UnitFlag for a PROCESS
        const PROCESS = 0b010;
        /// UnitFlag for a ENTITY
        const ENTITY = 0b100;
        /// UnitFlag for ALL
        const ALL = 0b111;
    }
}

/// The trigger modes for register data acquisition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegMode {
    /// The register is transparent if the trigger is low.
    Low,
    /// The register is transparent if the trigger is high.
    High,
    /// The register stores data on the rising edge of the trigger.
    Rise,
    /// The register stores data on the falling edge of the trigger.
    Fall,
    /// The register stores data on any edge of the trigger.
    Both,
}

impl std::fmt::Display for RegMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RegMode::Low => write!(f, "low"),
            RegMode::High => write!(f, "high"),
            RegMode::Rise => write!(f, "rise"),
            RegMode::Fall => write!(f, "fall"),
            RegMode::Both => write!(f, "both"),
        }
    }
}

/// The trigger for register data acquisition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegTrigger {
    /// The value to be stored.
    pub data: Value,
    /// The trigger mode.
    pub mode: RegMode,
    /// The trigger signal.
    pub trigger: Value,
    /// The gating condition.
    pub gate: Option<Value>,
}

/// An instruction opcode.
///
/// This enum represents the actual instruction, whereas `InstData` covers the
/// format and arguments of the instruction.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Con,
    Del,

    Call,
    Inst,

    Sig,
    Prb,
    Drv,
    DrvCond,

    Var,
    Ld,
    St,

    Halt,
    Ret,
    RetValue,
    Phi,
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
                Opcode::Con => "con",
                Opcode::Del => "del",
                Opcode::Call => "call",
                Opcode::Inst => "inst",
                Opcode::Sig => "sig",
                Opcode::Drv => "drv",
                Opcode::DrvCond => "drv",
                Opcode::Prb => "prb",
                Opcode::Var => "var",
                Opcode::Ld => "ld",
                Opcode::St => "st",
                Opcode::Halt => "halt",
                Opcode::Ret => "ret",
                Opcode::RetValue => "ret",
                Opcode::Phi => "phi",
                Opcode::Br => "br",
                Opcode::BrCond => "br",
                Opcode::Wait => "wait",
                Opcode::WaitTime => "wait",
            }
        )
    }
}

impl Opcode {
    /// Return a set of flags where this instruction is valid.
    pub fn valid_in(self) -> UnitFlags {
        match self {
            Opcode::Halt => UnitFlags::PROCESS | UnitFlags::ENTITY,
            Opcode::Wait => UnitFlags::PROCESS,
            Opcode::WaitTime => UnitFlags::PROCESS,
            Opcode::Ret => UnitFlags::FUNCTION,
            Opcode::RetValue => UnitFlags::FUNCTION,
            Opcode::Phi => UnitFlags::FUNCTION | UnitFlags::PROCESS,
            Opcode::Br => UnitFlags::FUNCTION | UnitFlags::PROCESS,
            Opcode::BrCond => UnitFlags::FUNCTION | UnitFlags::PROCESS,
            Opcode::Con => UnitFlags::ENTITY,
            Opcode::Del => UnitFlags::ENTITY,
            Opcode::Reg => UnitFlags::ENTITY,
            Opcode::Inst => UnitFlags::ENTITY,
            _ => UnitFlags::ALL,
        }
    }

    /// Check if this instruction can appear in a `Function`.
    pub fn valid_in_function(self) -> bool {
        self.valid_in().contains(UnitFlags::FUNCTION)
    }

    /// Check if this instruction can appear in a `Process`.
    pub fn valid_in_process(self) -> bool {
        self.valid_in().contains(UnitFlags::PROCESS)
    }

    /// Check if this instruction can appear in a `Entity`.
    pub fn valid_in_entity(self) -> bool {
        self.valid_in().contains(UnitFlags::ENTITY)
    }

    /// Check if this instruction is a constant.
    pub fn is_const(self) -> bool {
        match self {
            Opcode::ConstInt => true,
            Opcode::ConstTime => true,
            _ => false,
        }
    }

    /// Check if this instruction is a phi node.
    pub fn is_phi(self) -> bool {
        match self {
            Opcode::Phi => true,
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

    /// Check if this is a temporal instruction.
    pub fn is_temporal(self) -> bool {
        match self {
            Opcode::Halt | Opcode::Wait | Opcode::WaitTime => true,
            _ => false,
        }
    }
}

impl Inst {
    /// Dump the instruction in human readable form
    pub fn dump<'a>(self, unit: &Unit<'a>) -> InstDumper<'a> {
        InstDumper(self, *unit)
    }
}

/// Temporary object to dump an `Inst` in human-readable form for debugging.
pub struct InstDumper<'a>(Inst, Unit<'a>);

impl std::fmt::Display for InstDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let inst = self.0;
        let unit = self.1;
        let data = &unit[inst];
        if unit.has_result(inst) {
            let result = unit.inst_result(inst);
            write!(
                f,
                "{} = {} {}",
                result.dump(&unit),
                data.opcode(),
                unit.value_type(result)
            )?;
        } else {
            write!(f, "{}", data.opcode())?;
        }
        if let InstData::Call { unit: ext_unit, .. } = *data {
            write!(f, " {}", unit[ext_unit].name)?;
            write!(f, " (")?;
            let mut comma = false;
            for arg in data.input_args() {
                if comma {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg.dump(&unit))?;
                comma = true;
            }
            write!(f, ")")?;
            if data.opcode() == Opcode::Inst {
                write!(f, " -> (")?;
                let mut comma = false;
                for arg in data.output_args() {
                    if comma {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.dump(&unit))?;
                    comma = true;
                }
                write!(f, ")")?;
            }
        } else if let InstData::Reg { .. } = *data {
            write!(f, " {}", data.args()[0])?;
            for arg in data.data_args() {
                write!(f, ", {}", arg.dump(&unit))?;
            }
            for arg in data.mode_args() {
                write!(f, ", {}", arg)?;
            }
            for arg in data.trigger_args() {
                write!(f, ", {}", arg.dump(&unit))?;
            }
        } else if let InstData::Phi { .. } = *data {
            let mut comma = false;
            write!(f, " ")?;
            for (arg, block) in data.args().iter().zip(data.blocks().iter()) {
                if comma {
                    write!(f, ", ")?;
                }
                write!(f, "[{}, {}]", arg.dump(&unit), block.dump(&unit))?;
                comma = true;
            }
        } else {
            let mut comma = false;
            for arg in data.args() {
                if comma {
                    write!(f, ",")?;
                }
                write!(f, " {}", arg.dump(&unit))?;
                comma = true;
            }
            for block in data.blocks() {
                if comma {
                    write!(f, ",")?;
                }
                write!(f, " {}", block.dump(&unit))?;
                comma = true;
            }
            for imm in data.imms() {
                if comma {
                    write!(f, ",")?;
                }
                write!(f, " {}", imm)?;
                comma = true;
            }
            match data {
                InstData::ConstInt { imm, .. } => write!(f, " {}", imm.value)?,
                InstData::ConstTime { imm, .. } => write!(f, " {}", imm)?,
                InstData::Array { imms, .. } => write!(f, ", {}", imms[0])?,
                _ => (),
            }
        }
        Ok(())
    }
}

fn with_unpacked_sigptr(ty: Type, f: impl FnOnce(Type) -> Type) -> Type {
    if ty.is_pointer() {
        pointer_ty(f(ty.unwrap_pointer().clone()))
    } else if ty.is_signal() {
        signal_ty(f(ty.unwrap_signal().clone()))
    } else {
        f(ty)
    }
}
