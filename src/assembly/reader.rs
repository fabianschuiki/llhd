// Copyright (c) 2017-2021 Fabian Schuiki

//! Temporary representation of LLHD IR after parsing.

use crate::{
    ir::{self, Opcode, Signature, UnitBuilder, UnitName},
    ty::Type,
    value::{IntValue, TimeValue},
};
use num::{BigInt, BigRational};
use std::collections::HashMap;

#[derive(Default)]
pub struct Context<'a> {
    pub value_names: HashMap<LocalName<'a>, ir::Value>,
    pub block_names: HashMap<LocalName<'a>, ir::Block>,
}

pub enum Unit {
    Data(ir::UnitData, usize),
    Declare(ir::UnitName, ir::Signature, usize),
}

pub struct Block<'a> {
    pub name: LocalName<'a>,
    pub insts: Vec<Inst<'a>>,
}

impl<'a> Block<'a> {
    pub fn build(self, builder: &mut UnitBuilder, context: &mut Context<'a>) {
        let bb = match context.block_names.get(&self.name).cloned() {
            Some(bb) => bb,
            None => {
                let bb = builder.block();
                context.block_names.insert(self.name, bb);
                bb
            }
        };
        match self.name {
            LocalName::Anonymous(index) => builder.set_anonymous_block_hint(bb, index),
            LocalName::Named(name) => builder.set_block_name(bb, name.to_owned()),
        }
        builder.append_to(bb);
        for inst in self.insts {
            inst.build(builder, context);
        }
    }
}

pub struct Inst<'a> {
    pub opcode: Opcode,
    pub name: Option<LocalName<'a>>,
    pub data: InstData<'a>,
    pub loc: Option<usize>,
}

pub enum InstData<'a> {
    ConstInt(IntValue),
    ConstTime(TimeValue),
    Aggregate(usize, Vec<TypedValue<'a>>),
    Nullary,
    Unary(TypedValue<'a>),
    Binary(TypedValue<'a>, TypedValue<'a>),
    Ternary(TypedValue<'a>, TypedValue<'a>, TypedValue<'a>),
    Quaternary(
        TypedValue<'a>,
        TypedValue<'a>,
        TypedValue<'a>,
        TypedValue<'a>,
    ),
    Reg(
        TypedValue<'a>,
        Vec<(
            TypedValue<'a>,
            ir::RegMode,
            TypedValue<'a>,
            Option<TypedValue<'a>>,
        )>,
    ),
    Ins(TypedValue<'a>, TypedValue<'a>, [usize; 2]),
    Ext(Type, TypedValue<'a>, [usize; 2]),
    Call(Type, UnitName, Vec<TypedValue<'a>>),
    Inst(UnitName, Vec<TypedValue<'a>>, Vec<TypedValue<'a>>),
    Phi(Type, Vec<(TypedValue<'a>, Label<'a>)>),
    Branch(Option<TypedValue<'a>>, Label<'a>, Option<Label<'a>>),
    Wait(Label<'a>, Option<TypedValue<'a>>, Vec<Value<'a>>),
}

impl<'a> Inst<'a> {
    pub fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            name: None,
            data: InstData::Nullary,
            loc: None,
        }
    }

    pub fn name(self, name: LocalName<'a>) -> Self {
        let mut x = self;
        x.name = Some(name);
        x
    }

    pub fn location(self, loc: usize) -> Self {
        let mut x = self;
        x.loc = Some(loc);
        x
    }

    pub fn data(self, data: InstData<'a>) -> Self {
        let mut x = self;
        x.data = data;
        x
    }

    pub fn build(self, builder: &mut UnitBuilder, context: &mut Context<'a>) {
        let result: InstOrValue = match self.data {
            InstData::ConstInt(imm) => builder.ins().const_int(imm).into(),
            InstData::ConstTime(imm) => builder.ins().const_time(imm).into(),
            InstData::Aggregate(size, args) => {
                let args = args
                    .into_iter()
                    .map(|a| a.build(builder, context))
                    .collect();
                match self.opcode {
                    Opcode::Array => builder.ins().array(args).into(),
                    Opcode::Struct => builder.ins().strukt(args).into(),
                    Opcode::ArrayUniform => builder.ins().array_uniform(size, args[0]).into(),
                    x => unreachable!("aggregate {:?}", x),
                }
            }
            InstData::Nullary => match self.opcode {
                Opcode::Halt => builder.ins().halt().into(),
                Opcode::Ret => builder.ins().ret().into(),
                x => unreachable!("nullary {:?}", x),
            },
            InstData::Unary(arg) => {
                let arg = arg.build(builder, context);
                match self.opcode {
                    Opcode::Alias => builder.ins().alias(arg).into(),
                    Opcode::Not => builder.ins().not(arg).into(),
                    Opcode::Neg => builder.ins().neg(arg).into(),
                    Opcode::RetValue => builder.ins().ret_value(arg).into(),
                    Opcode::Sig => builder.ins().sig(arg).into(),
                    Opcode::Prb => builder.ins().prb(arg).into(),
                    Opcode::Var => builder.ins().var(arg).into(),
                    Opcode::Ld => builder.ins().ld(arg).into(),
                    x => unreachable!("unary {:?}", x),
                }
            }
            InstData::Binary(arg0, arg1) => {
                let arg0 = arg0.build(builder, context);
                let arg1 = arg1.build(builder, context);
                match self.opcode {
                    Opcode::Add => builder.ins().add(arg0, arg1).into(),
                    Opcode::Sub => builder.ins().sub(arg0, arg1).into(),
                    Opcode::And => builder.ins().and(arg0, arg1).into(),
                    Opcode::Or => builder.ins().or(arg0, arg1).into(),
                    Opcode::Xor => builder.ins().xor(arg0, arg1).into(),
                    Opcode::Smul => builder.ins().smul(arg0, arg1).into(),
                    Opcode::Sdiv => builder.ins().sdiv(arg0, arg1).into(),
                    Opcode::Smod => builder.ins().smod(arg0, arg1).into(),
                    Opcode::Srem => builder.ins().srem(arg0, arg1).into(),
                    Opcode::Umul => builder.ins().umul(arg0, arg1).into(),
                    Opcode::Udiv => builder.ins().udiv(arg0, arg1).into(),
                    Opcode::Umod => builder.ins().umod(arg0, arg1).into(),
                    Opcode::Urem => builder.ins().urem(arg0, arg1).into(),
                    Opcode::Eq => builder.ins().eq(arg0, arg1).into(),
                    Opcode::Neq => builder.ins().neq(arg0, arg1).into(),
                    Opcode::Slt => builder.ins().slt(arg0, arg1).into(),
                    Opcode::Sgt => builder.ins().sgt(arg0, arg1).into(),
                    Opcode::Sle => builder.ins().sle(arg0, arg1).into(),
                    Opcode::Sge => builder.ins().sge(arg0, arg1).into(),
                    Opcode::Ult => builder.ins().ult(arg0, arg1).into(),
                    Opcode::Ugt => builder.ins().ugt(arg0, arg1).into(),
                    Opcode::Ule => builder.ins().ule(arg0, arg1).into(),
                    Opcode::Uge => builder.ins().uge(arg0, arg1).into(),
                    Opcode::Mux => builder.ins().mux(arg0, arg1).into(),
                    Opcode::Con => builder.ins().con(arg0, arg1).into(),
                    Opcode::St => builder.ins().st(arg0, arg1).into(),
                    x => unreachable!("binary {:?}", x),
                }
            }
            InstData::Ternary(arg0, arg1, arg2) => {
                let arg0 = arg0.build(builder, context);
                let arg1 = arg1.build(builder, context);
                let arg2 = arg2.build(builder, context);
                match self.opcode {
                    Opcode::Drv => builder.ins().drv(arg0, arg1, arg2).into(),
                    Opcode::Shl => builder.ins().shl(arg0, arg1, arg2).into(),
                    Opcode::Shr => builder.ins().shr(arg0, arg1, arg2).into(),
                    Opcode::Del => builder.ins().del(arg0, arg1, arg2).into(),
                    x => unreachable!("ternary {:?}", x),
                }
            }
            InstData::Quaternary(arg0, arg1, arg2, arg3) => {
                let arg0 = arg0.build(builder, context);
                let arg1 = arg1.build(builder, context);
                let arg2 = arg2.build(builder, context);
                let arg3 = arg3.build(builder, context);
                match self.opcode {
                    Opcode::DrvCond => builder.ins().drv_cond(arg0, arg1, arg2, arg3).into(),
                    x => unreachable!("quaternary {:?}", x),
                }
            }
            InstData::Reg(init, triggers) => {
                let init = init.build(builder, context);
                let triggers = triggers
                    .into_iter()
                    .map(|(data, mode, trigger, gate)| ir::RegTrigger {
                        data: data.build(builder, context),
                        mode: mode,
                        trigger: trigger.build(builder, context),
                        gate: gate.map(|g| g.build(builder, context)),
                    })
                    .collect();
                builder.ins().reg(init, triggers).into()
            }
            InstData::Ins(target, value, imm) => {
                let target = target.build(builder, context);
                let value = value.build(builder, context);
                match self.opcode {
                    Opcode::InsField => builder.ins().ins_field(target, value, imm[0]).into(),
                    Opcode::InsSlice => builder
                        .ins()
                        .ins_slice(target, value, imm[0], imm[1])
                        .into(),
                    x => unreachable!("ins {:?}", x),
                }
            }
            InstData::Ext(ty, target, imm) => {
                let target = target.build(builder, context);
                let ins = match self.opcode {
                    Opcode::ExtField => builder.ins().ext_field(target, imm[0]),
                    Opcode::ExtSlice => builder.ins().ext_slice(target, imm[0], imm[1]),
                    x => unreachable!("ext {:?}", x),
                };
                assert_eq!(builder.value_type(ins), ty);
                ins.into()
            }
            InstData::Call(ty, unit, args) => {
                let mut sig = Signature::new();
                sig.set_return_type(ty);
                for arg in &args {
                    sig.add_input(arg.ty.clone());
                }
                let ext = builder.add_extern(unit, sig);
                let args = args
                    .into_iter()
                    .map(|v| v.build(builder, context))
                    .collect();
                builder.ins().call(ext, args).into()
            }
            InstData::Inst(unit, input_args, output_args) => {
                let mut sig = Signature::new();
                for arg in &input_args {
                    sig.add_input(arg.ty.clone());
                }
                for arg in &output_args {
                    sig.add_output(arg.ty.clone());
                }
                let ext = builder.add_extern(unit, sig);
                let input_args = input_args
                    .into_iter()
                    .map(|v| v.build(builder, context))
                    .collect();
                let output_args = output_args
                    .into_iter()
                    .map(|v| v.build(builder, context))
                    .collect();
                builder.ins().inst(ext, input_args, output_args).into()
            }
            InstData::Phi(_, edges) => {
                let mut args = vec![];
                let mut bbs = vec![];
                for (arg, bb) in edges {
                    args.push(arg.build(builder, context));
                    bbs.push(bb.build(builder, context));
                }
                builder.ins().phi(args, bbs).into()
            }
            InstData::Branch(cond, bb0, bb1) => {
                let bb0 = bb0.build(builder, context);
                match self.opcode {
                    Opcode::Br => builder.ins().br(bb0).into(),
                    Opcode::BrCond => {
                        let cond = cond.unwrap().build(builder, context);
                        let bb1 = bb1.unwrap().build(builder, context);
                        builder.ins().br_cond(cond, bb0, bb1).into()
                    }
                    x => unreachable!("branch {:?}", x),
                }
            }
            InstData::Wait(bb, time, args) => {
                let bb = bb.build(builder, context);
                let args = args
                    .into_iter()
                    .map(|a| a.build(builder, context))
                    .collect();
                match self.opcode {
                    Opcode::Wait => builder.ins().wait(bb, args).into(),
                    Opcode::WaitTime => {
                        let time = time.unwrap().build(builder, context);
                        builder.ins().wait_time(bb, time, args).into()
                    }
                    x => unreachable!("wait {:?}", x),
                }
            }
        };
        if let (Some(name), InstOrValue::Value(value)) = (self.name, result) {
            if let Some(ph) = context.value_names.insert(name, value) {
                if builder.is_placeholder(ph) {
                    builder.replace_use(ph, value);
                    builder.remove_placeholder(ph);
                } else {
                    panic!("`{}` defined multiple times", name);
                }
            }
            match name {
                LocalName::Anonymous(index) => builder.set_anonymous_hint(value, index),
                LocalName::Named(name) => builder.set_name(value, name.to_owned()),
            }
        }
        if let Some(loc) = self.loc {
            let inst = match result {
                InstOrValue::Inst(inst) => inst,
                InstOrValue::Value(value) => builder.value_inst(value),
            };
            builder.set_location_hint(inst, loc);
        }
    }
}

#[derive(Copy, Clone)]
pub enum InstOrValue {
    Inst(ir::Inst),
    Value(ir::Value),
}

impl From<ir::Inst> for InstOrValue {
    fn from(x: ir::Inst) -> InstOrValue {
        InstOrValue::Inst(x)
    }
}

impl From<ir::Value> for InstOrValue {
    fn from(x: ir::Value) -> InstOrValue {
        InstOrValue::Value(x)
    }
}

impl std::fmt::Display for InstOrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstOrValue::Inst(x) => write!(f, "{}", x),
            InstOrValue::Value(x) => write!(f, "{}", x),
        }
    }
}

/// A local name such as `%0` or `%foo`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum LocalName<'a> {
    Anonymous(u32),
    Named(&'a str),
}

impl<'a> From<&'a str> for LocalName<'a> {
    fn from(name: &'a str) -> Self {
        if name.chars().all(|c| c.is_digit(10)) {
            LocalName::Anonymous(name.parse().unwrap())
        } else {
            LocalName::Named(name)
        }
    }
}

impl std::fmt::Display for LocalName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LocalName::Anonymous(x) => write!(f, "%{}", x),
            LocalName::Named(x) => write!(f, "%{}", x),
        }
    }
}

/// A value without explicit type.
pub struct Value<'a>(pub LocalName<'a>);

impl<'a> Value<'a> {
    /// Associate a type with this value.
    pub fn ty(self, ty: Type) -> TypedValue<'a> {
        TypedValue { value: self, ty }
    }
}

impl std::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Value<'_> {
    fn build(self, _builder: &mut UnitBuilder, context: &mut Context) -> ir::Value {
        match context.value_names.get(&self.0) {
            Some(&v) => v,
            None => panic!("value {} has not been declared", self),
        }
    }
}

/// A value with explicit type.
pub struct TypedValue<'a> {
    pub value: Value<'a>,
    pub ty: Type,
}

impl<'a> TypedValue<'a> {
    fn build(self, builder: &mut UnitBuilder, context: &mut Context<'a>) -> ir::Value {
        match context.value_names.get(&self.value.0).cloned() {
            Some(v) => {
                // assert_eq!(builder.value_type(v), self.ty, "type mismatch");
                // The above will be caught by the verifier in a more gentle way
                v
            }
            None => {
                let value = builder.add_placeholder(self.ty);
                context.value_names.insert(self.value.0, value);
                value
            }
        }
    }
}

impl std::fmt::Display for TypedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.ty, self.value)
    }
}

/// A label.
pub struct Label<'a>(pub LocalName<'a>);

impl std::fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Label<'a> {
    fn build(self, builder: &mut UnitBuilder, context: &mut Context<'a>) -> ir::Block {
        match context.block_names.get(&self.0).cloned() {
            Some(bb) => bb,
            None => {
                let bb = builder.block();
                context.block_names.insert(self.0, bb);
                bb
            }
        }
    }
}

pub fn parse_time_triple(
    time: &str,
    delta: Option<&str>,
    epsilon: Option<&str>,
) -> (BigRational, usize, usize) {
    // Strip away the unit suffices.
    let time = &time[..time.len() - 1];
    let delta = delta.map(|delta| &delta[..delta.len() - 1]);
    let epsilon = epsilon.map(|epsilon| &epsilon[..epsilon.len() - 1]);

    // Determine the SI prefix for the time.
    let scale = match &time[time.len() - 1..] {
        "a" => Some(-18),
        "f" => Some(-15),
        "p" => Some(-12),
        "n" => Some(-9),
        "u" => Some(-6),
        "m" => Some(-3),
        "k" => Some(3),
        "M" => Some(6),
        "G" => Some(9),
        "T" => Some(12),
        "P" => Some(15),
        "E" => Some(18),
        _ => None,
    };
    let time = if scale.is_some() {
        &time[..time.len() - 1]
    } else {
        time
    };
    let scale = scale.unwrap_or(0);

    // Split the time into integer and fractional parts.
    let mut split = time.split('.');
    let int = split.next().unwrap();
    let frac = split.next();

    // Concatenate the integer and fraction part into one number.
    let mut numer = int.to_owned();
    if let Some(ref frac) = frac {
        numer.push_str(frac);
    }
    let mut denom = String::from("1");

    // Calculate the exponent the numerator needs to be multiplied with
    // to arrive at the correct value. If it is negative, i.e. the order
    // of magnitude needs to be reduced, append that amount of zeros to
    // the denominator. If it is positive, i.e. the order of magnitude
    // needs to be increased, append that amount of zeros to the
    // numerator.
    let zeros = scale - frac.map(|s| s.len() as isize).unwrap_or(0);
    if zeros < 0 {
        denom.extend(std::iter::repeat('0').take(-zeros as usize))
    } else if zeros > 0 {
        numer.extend(std::iter::repeat('0').take(zeros as usize))
    }

    // Convert the values to BigInt and combine them into a rational
    // number.
    let numer = BigInt::parse_bytes(numer.as_bytes(), 10).unwrap();
    let denom = BigInt::parse_bytes(denom.as_bytes(), 10).unwrap();
    let v = BigRational::new(numer, denom);

    // Parse the delta and epsilon times.
    let delta = delta.map(|x| x.parse().unwrap()).unwrap_or(0);
    let epsilon = epsilon.map(|x| x.parse().unwrap()).unwrap_or(0);

    (v, delta, epsilon)
}

pub use super::grammar::{ModuleParser, TimeValueParser, TypeParser};
