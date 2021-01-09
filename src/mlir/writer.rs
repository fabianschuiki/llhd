// Copyright (c) 2017-2021 Fabian Schuiki

//! Emitting LLHD IR assembly.

use crate::{
    ir::{prelude::*, UnitKind},
    Type, TypeKind,
};
use itertools::Itertools;
use num::{cast::FromPrimitive, BigInt, BigRational, One};
use std::{
    collections::{HashMap, HashSet},
    io::{Result, Write},
    rc::Rc,
};

/// Temporary object to emit LLHD IR assembly.
pub struct Writer<T> {
    sink: T,
}

struct MLIRUnitName<'a>(&'a UnitName);

impl std::fmt::Display for MLIRUnitName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            UnitName::Anonymous(id) => write!(f, "@{}", id),
            UnitName::Local(n) => write!(f, "@{}", n),
            UnitName::Global(n) => write!(f, "@{}", n),
        }
    }
}

struct MLIRUnitKind(UnitKind);

impl std::fmt::Display for MLIRUnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            UnitKind::Function => write!(f, "func"),
            UnitKind::Process => write!(f, "llhd.proc"),
            UnitKind::Entity => write!(f, "llhd.entity"),
        }
    }
}

struct MLIRType<'a>(&'a Type);

impl std::fmt::Display for MLIRType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match **self.0 {
            TypeKind::VoidType => write!(f, "()"),
            TypeKind::TimeType => write!(f, "!llhd.time"),
            TypeKind::IntType(l) => write!(f, "i{}", l),
            TypeKind::EnumType(l) => write!(f, "n{}", l),
            TypeKind::PointerType(ref ty) => write!(f, "!llhd.ptr<{}>", MLIRType(ty)),
            TypeKind::SignalType(ref ty) => write!(f, "!llhd.sig<{}>", MLIRType(ty)),
            TypeKind::ArrayType(l, ref ty) => write!(f, "!llhd.array<{}x{}>", l, MLIRType(ty)),
            TypeKind::StructType(ref tys) => {
                write!(f, "tuple<{}>", tys.iter().map(|t| MLIRType(t)).format(", "))
            }
            TypeKind::FuncType(ref args, ref ret) => write!(
                f,
                "({}) -> {}",
                args.iter().map(|t| MLIRType(t)).format(", "),
                ret
            ),
            TypeKind::EntityType(ref ins, ref outs) => write!(
                f,
                "({}) -> ({})",
                ins.iter().map(|t| MLIRType(t)).format(", "),
                outs.iter().map(|t| MLIRType(t)).format(", ")
            ),
        }
    }
}

struct MLIROpcode(Opcode);

impl std::fmt::Display for MLIROpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Opcode::ConstInt => "llhd.const",
                Opcode::ConstTime => "llhd.const",
                Opcode::Alias => "alias",
                Opcode::ArrayUniform => "llhd.array_uniform",
                Opcode::Array => "llhd.array",
                Opcode::Struct => "llhd.tuple",
                Opcode::Not => "llhd.not",
                Opcode::Neg => "llhd.neg",
                Opcode::Add => "addi",
                Opcode::Sub => "subi",
                Opcode::And => "llhd.and",
                Opcode::Or => "llhd.or",
                Opcode::Xor => "llhd.xor",
                Opcode::Smul => "muli",
                Opcode::Sdiv => "divi_signed",
                Opcode::Smod => "llhd.smod",
                Opcode::Srem => "remi_signed",
                Opcode::Umul => "muli",
                Opcode::Udiv => "divi_unsigned",
                Opcode::Umod => "remi_unsigned",
                Opcode::Urem => "remi_unsigend",
                Opcode::Eq => "llhd.eq",
                Opcode::Neq => "llhd.neq",
                Opcode::Slt => "cmpi \"slt\",",
                Opcode::Sgt => "cmpi \"sgt\",",
                Opcode::Sle => "cmpi \"sle\",",
                Opcode::Sge => "cmpi \"sge\",",
                Opcode::Ult => "cmpi \"ult\",",
                Opcode::Ugt => "cmpi \"ugt\",",
                Opcode::Ule => "cmpi \"ule\",",
                Opcode::Uge => "cmpi \"uge\",",
                Opcode::Shl => "llhd.shl",
                Opcode::Shr => "llhd.shr",
                Opcode::Mux => "llhd.dyn_extract_element",
                Opcode::Reg => "llhd.reg",
                Opcode::InsField => "llhd.insert_element",
                Opcode::InsSlice => "llhd.insert_slice",
                Opcode::ExtField => "llhd.extract_element",
                Opcode::ExtSlice => "llhd.extract_slice",
                Opcode::Con => "con",
                Opcode::Del => "del",
                Opcode::Call => "call",
                Opcode::Inst => "llhd.inst",
                Opcode::Sig => "llhd.sig",
                Opcode::Drv => "llhd.drv",
                Opcode::DrvCond => "llhd.drv",
                Opcode::Prb => "llhd.prb",
                Opcode::Var => "llhd.var",
                Opcode::Ld => "llhd.load",
                Opcode::St => "llhd.store",
                Opcode::Halt => "llhd.halt",
                Opcode::Ret => "ret",
                Opcode::RetValue => "ret",
                Opcode::Phi => "phi",
                Opcode::Br => "br",
                Opcode::BrCond => "cond_br",
                Opcode::Wait => "llhd.wait",
                Opcode::WaitTime => "llhd.wait",
            }
        )
    }
}

impl<T: Write> Writer<T> {
    /// Create a new assembly writer.
    pub fn new(sink: T) -> Self {
        Self { sink }
    }

    /// Emit assembly for a module.
    pub fn write_module(&mut self, module: &Module) -> Result<()> {
        let mut separate = false;
        for unit in module.units() {
            if separate {
                write!(self.sink, "\n")?;
            }
            separate = true;
            self.write_unit(unit)?;
        }
        for decl in module.decls() {
            if separate {
                write!(self.sink, "\n")?;
            }
            separate = false;
            let data = &module[decl];
            self.write_declaration(&data.sig, &data.name)?;
        }
        Ok(())
    }

    /// Emit assembly for a unit.
    pub fn write_unit(&mut self, data: Unit) -> Result<()> {
        let mut uw = UnitWriter::new(self, data);
        write!(
            uw.writer.sink,
            "{} {}(",
            MLIRUnitKind(data.kind()),
            MLIRUnitName(data.name())
        )?;
        let mut comma = false;
        for arg in data.sig().inputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            uw.write_value_name(data.arg_value(arg))?;
            write!(uw.writer.sink, ": {}", MLIRType(&data.sig().arg_type(arg)))?;
        }
        if data.kind() == UnitKind::Function {
            write!(
                uw.writer.sink,
                ") {} {{\n",
                MLIRType(&data.sig().return_type())
            )?;
        } else {
            write!(uw.writer.sink, ") -> (")?;
            let mut comma = false;
            for arg in data.sig().outputs() {
                if comma {
                    write!(uw.writer.sink, ", ")?;
                }
                comma = true;
                uw.write_value_name(data.arg_value(arg))?;
                write!(uw.writer.sink, ": {} ", MLIRType(&data.sig().arg_type(arg)))?;
            }
            write!(uw.writer.sink, ") {{\n")?;
        }

        let mut block_args = HashMap::<Block, Vec<Value>>::new();
        let mut terminator_args = HashMap::<(Block, Block), Vec<Value>>::new();
        if data.kind() != UnitKind::Entity {
            for block in data.blocks() {
                for inst in data.insts(block) {
                    if let Opcode::Phi = data[inst].opcode() {
                        block_args.entry(block).or_insert(Vec::new());
                        block_args
                            .get_mut(&block)
                            .unwrap()
                            .push(uw.unit.inst_result(inst));
                        for (&arg, &source_block) in
                            data[inst].args().iter().zip(data[inst].blocks().iter())
                        {
                            terminator_args
                                .entry((source_block, block))
                                .or_insert(Vec::new());
                            terminator_args
                                .get_mut(&(source_block, block))
                                .unwrap()
                                .push(arg);
                        }
                    }
                }
            }
        }

        if data.kind() != UnitKind::Entity {
            if let Some(block) = data.first_block() {
                write!(uw.writer.sink, "    ")?;
                write!(uw.writer.sink, "br ")?;
                uw.write_block_name(block, block_args.get(&block).unwrap_or(&Vec::new()))?;
                write!(uw.writer.sink, "\n")?;
            }
        }

        let mut deleted = HashSet::new();
        for block in data.blocks() {
            if data.kind() != UnitKind::Entity {
                uw.write_block_name(block, block_args.get(&block).unwrap_or(&Vec::new()))?;
                write!(uw.writer.sink, ":\n")?;
            }
            for inst in data.insts(block) {
                if data[inst].opcode().is_terminator() && data.is_entity() {
                    continue;
                }
                if data[inst].opcode() == Opcode::Phi {
                    continue;
                }
                if data.kind() == UnitKind::Process && data[inst].opcode() == Opcode::Sig {
                    let sig_val = uw.unit.inst_result(inst);
                    for &user_inst in uw.unit.uses(sig_val) {
                        if let Opcode::Shr | Opcode::Shl = data[user_inst].opcode() {
                            if data[user_inst].args()[1] != sig_val {
                                continue;
                            }
                            let amt = data[user_inst].args()[2];
                            let base = data[user_inst].args()[0];
                            for &shft_user in uw.unit.uses(uw.unit.inst_result(user_inst)) {
                                if let Opcode::ExtSlice | Opcode::ExtField =
                                    data[shft_user].opcode()
                                {
                                    if data[shft_user].imms()[0] == 0 {
                                        write!(uw.writer.sink, "    ")?;
                                        uw.write_value_name(uw.unit.inst_result(shft_user))?;
                                        if let Opcode::ExtSlice = data[shft_user].opcode() {
                                            write!(uw.writer.sink, " = llhd.dyn_extract_slice ")?;
                                        } else {
                                            write!(uw.writer.sink, " = llhd.dyn_extract_element ")?;
                                        }
                                        uw.write_value_use(base, false)?;
                                        write!(uw.writer.sink, ", ")?;
                                        uw.write_value_use(amt, false)?;
                                        write!(
                                            uw.writer.sink,
                                            " : ({}, {}) -> {}\n",
                                            MLIRType(&uw.unit.value_type(base)),
                                            MLIRType(&uw.unit.value_type(amt)),
                                            MLIRType(&uw.unit.inst_type(shft_user))
                                        )?;
                                    }
                                }
                                deleted.insert(shft_user);
                            }
                        }
                        deleted.insert(user_inst);
                    }
                    deleted.insert(inst);
                }
                if deleted.contains(&inst) {
                    continue;
                }
                write!(uw.writer.sink, "    ")?;
                uw.write_inst(block, inst, &terminator_args)?;
                write!(uw.writer.sink, "\n")?;
            }
        }
        write!(uw.writer.sink, "}}\n")?;
        Ok(())
    }

    /// Emit assembly for a declaration.
    pub fn write_declaration(&mut self, sig: &Signature, name: &UnitName) -> Result<()> {
        write!(self.sink, "declare {} {}\n", MLIRUnitName(name), sig)?;
        Ok(())
    }
}

pub struct UnitWriter<'a, T> {
    writer: &'a mut Writer<T>,
    unit: Unit<'a>,
    value_names: HashMap<Value, Rc<String>>,
    block_names: HashMap<Block, Rc<String>>,
    name_indices: HashMap<Rc<String>, usize>,
    names: HashSet<Rc<String>>,
    tmp_index: usize,
}

impl<'a, T: Write> UnitWriter<'a, T> {
    /// Create a new writer for a unit.
    pub fn new(writer: &'a mut Writer<T>, unit: Unit<'a>) -> Self {
        Self {
            writer,
            unit,
            value_names: Default::default(),
            block_names: Default::default(),
            name_indices: Default::default(),
            names: Default::default(),
            tmp_index: 0,
        }
    }

    /// Emit the name of a value.
    pub fn write_value_name(&mut self, value: Value) -> Result<()> {
        // If we have already picked a name for the value, use that.
        if let Some(name) = self.value_names.get(&value) {
            return write!(self.writer.sink, "%{}", name);
        }

        // Check if the value has an explicit name set, or if we should just
        // generate a temporary name.
        let name = self.uniquify_name(self.unit.get_name(value));

        // Emit the name and associate it with the value for later reuse.
        write!(self.writer.sink, "%{}", name)?;
        self.value_names.insert(value, name);
        Ok(())
    }

    /// Emit the name of a BB.
    pub fn write_block_name(&mut self, block: Block, block_args: &Vec<Value>) -> Result<()> {
        // If we have already picked a name for the value, use that.
        if let Some(name) = self.block_names.get(&block) {
            write!(self.writer.sink, "^{}", name)?;
            let mut first = true;
            for arg in block_args {
                if !first {
                    write!(self.writer.sink, ", ")?;
                } else {
                    write!(self.writer.sink, "(")?;
                }
                first = false;
                self.write_value_name(*arg)?;
                write!(
                    self.writer.sink,
                    ": {}",
                    MLIRType(&self.unit.value_type(*arg))
                )?;
            }
            if block_args.len() > 0 {
                write!(self.writer.sink, ")")?;
            }
            return Ok(());
        }

        // Check if the block has an explicit name set, or if we should just
        // generate a temporary name.
        let name = self.uniquify_name(self.unit.get_block_name(block));

        // Emit the name and associate it with the block for later reuse.
        write!(self.writer.sink, "^{}", name)?;
        let mut first = true;
        for arg in block_args {
            if !first {
                write!(self.writer.sink, ", ")?;
            } else {
                write!(self.writer.sink, "(")?;
            }
            first = false;
            self.write_value_name(*arg)?;
            write!(
                self.writer.sink,
                ": {}",
                MLIRType(&self.unit.value_type(*arg))
            )?;
        }
        if block_args.len() > 0 {
            write!(self.writer.sink, ")")?;
        }
        self.block_names.insert(block, name);
        Ok(())
    }

    /// Emit the name of a BB to be used as label in an instruction.
    pub fn write_block_value(&mut self, block: Block, block_args: &Vec<Value>) -> Result<()> {
        // If we have already picked a name for the value, use that.
        if let Some(name) = self.block_names.get(&block) {
            write!(self.writer.sink, "^{}", name)?;
            if block_args.len() > 0 {
                let mut first = true;
                write!(self.writer.sink, "(")?;
                for arg in block_args {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    first = false;
                    self.write_value_name(*arg)?;
                }
                write!(self.writer.sink, " : ")?;
                first = true;
                for arg in block_args {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    first = false;
                    write!(
                        self.writer.sink,
                        "{}",
                        MLIRType(&self.unit.value_type(*arg))
                    )?;
                }
                write!(self.writer.sink, ")")?;
            }
            return Ok(());
        }

        // Check if the block has an explicit name set, or if we should just
        // generate a temporary name.
        let name = self.uniquify_name(self.unit.get_block_name(block));

        // Emit the name and associate it with the block for later reuse.
        write!(self.writer.sink, "^{}", name)?;
        if block_args.len() > 0 {
            let mut first = true;
            write!(self.writer.sink, "(")?;
            for arg in block_args {
                if !first {
                    write!(self.writer.sink, ", ")?;
                }
                first = false;
                self.write_value_name(*arg)?;
            }
            write!(self.writer.sink, " : ")?;
            first = true;
            for arg in block_args {
                if !first {
                    write!(self.writer.sink, ", ")?;
                }
                first = false;
                write!(
                    self.writer.sink,
                    "{}",
                    MLIRType(&self.unit.value_type(*arg))
                )?;
            }
            write!(self.writer.sink, ")")?;
        }
        self.block_names.insert(block, name);
        Ok(())
    }

    /// Uniquify a value or block name.
    fn uniquify_name(&mut self, name: Option<&str>) -> Rc<String> {
        if let Some(requested_name) = name {
            let requested_name = escape_name(requested_name);
            let idx = self.name_indices.entry(requested_name.clone()).or_insert(0);
            loop {
                let name = if *idx == 0 {
                    requested_name.clone()
                } else {
                    Rc::new(format!("{}{}", requested_name, idx))
                };
                *idx += 1;
                if self.names.insert(name.clone()) {
                    break name;
                }
            }
        } else {
            loop {
                let name = Rc::new(format!("{}", self.tmp_index));
                self.tmp_index += 1;
                if self.names.insert(name.clone()) {
                    break name;
                }
            }
        }
    }

    /// Emit the use of a value.
    pub fn write_value_use(&mut self, value: Value, with_type: bool) -> Result<()> {
        if with_type {
            write!(self.writer.sink, "{} ", self.unit.value_type(value))?;
        }
        self.write_value_name(value)
    }

    /// Emit an instruction.
    pub fn write_inst(
        &mut self,
        curr_block: Block,
        inst: Inst,
        terminator_args: &HashMap<(Block, Block), Vec<Value>>,
    ) -> Result<()> {
        let def = Vec::new();
        let unit = self.unit;

        fn get_canonicalized_time(time: &BigRational) -> (BigRational, String) {
            let mut t = time.clone();
            let si_units = vec!["ys", "zs", "as", "fs", "ps", "ns", "us", "ms", "s"];
            for &prefix in si_units.iter().rev() {
                if t.denom() == &BigInt::one() {
                    return (t, prefix.to_string());
                }
                t = t * BigRational::from_i64(1000).unwrap();
            }
            unreachable!("too small time amount");
        }

        if unit.has_result(inst) {
            self.write_value_name(unit.inst_result(inst))?;
            write!(self.writer.sink, " = ")?;
        }
        let data = &unit[inst];
        match data.opcode() {
            Opcode::ConstInt => write!(
                self.writer.sink,
                "{} {} : {}",
                MLIROpcode(data.opcode()),
                data.get_const_int().unwrap().value,
                MLIRType(&unit.value_type(unit.inst_result(inst)))
            )?,
            Opcode::ConstTime => write!(
                self.writer.sink,
                "{} #llhd.time<{}{}, {}d, {}e> : {}",
                MLIROpcode(data.opcode()),
                get_canonicalized_time(&data.get_const_time().unwrap().time).0,
                get_canonicalized_time(&data.get_const_time().unwrap().time).1,
                data.get_const_time().unwrap().delta,
                data.get_const_time().unwrap().epsilon,
                MLIRType(&unit.value_type(unit.inst_result(inst)))
            )?,
            Opcode::ArrayUniform => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                self.write_value_use(data.args()[0], false)?;
                write!(
                    self.writer.sink,
                    " : !llhd.array<{}x{}>",
                    data.imms()[0],
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::Array => {
                write!(self.writer.sink, "llhd.array ")?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, false)?;
                    first = false;
                }
                write!(
                    self.writer.sink,
                    " : !llhd.array<{}x{}>",
                    data.args().len(),
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::Struct => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    first = false;
                    self.write_value_use(arg, false)?;
                }
                write!(self.writer.sink, " : tuple<")?;
                first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                    first = false;
                }
                write!(self.writer.sink, ">")?;
            }
            Opcode::Alias
            | Opcode::Not
            | Opcode::Neg
            | Opcode::Add
            | Opcode::Sub
            | Opcode::And
            | Opcode::Or
            | Opcode::Xor
            | Opcode::Smul
            | Opcode::Sdiv
            | Opcode::Smod
            | Opcode::Srem
            | Opcode::Umul
            | Opcode::Udiv
            | Opcode::Umod
            | Opcode::Urem
            | Opcode::Con
            | Opcode::Del
            | Opcode::Prb
            | Opcode::Var
            | Opcode::Ld
            | Opcode::St
            | Opcode::Eq
            | Opcode::Neq
            | Opcode::Slt
            | Opcode::Sgt
            | Opcode::Sle
            | Opcode::Sge
            | Opcode::Ult
            | Opcode::Ugt
            | Opcode::Ule
            | Opcode::Uge
            | Opcode::RetValue => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, false)?;
                    first = false;
                }
                write!(
                    self.writer.sink,
                    " : {}",
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::Sig => {
                let sig_name = if let Some(name) = self.value_names.get(&unit.inst_result(inst)) {
                    name.to_owned()
                } else {
                    self.uniquify_name(Some("sig"))
                };
                write!(
                    self.writer.sink,
                    "{} \"{}\" ",
                    MLIROpcode(data.opcode()),
                    sig_name
                )?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, false)?;
                    first = false;
                }
                write!(
                    self.writer.sink,
                    " : {}",
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::Drv => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let args = data.args();
                self.write_value_use(args[0], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_value_use(args[1], false)?;
                write!(self.writer.sink, " after ")?;
                self.write_value_use(args[2], false)?;
                write!(
                    self.writer.sink,
                    " : {}",
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::DrvCond => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let args = data.args();
                self.write_value_use(args[0], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_value_use(args[1], false)?;
                write!(self.writer.sink, " after ")?;
                self.write_value_use(args[2], false)?;
                write!(self.writer.sink, " if ")?;
                self.write_value_use(args[3], false)?;
                write!(
                    self.writer.sink,
                    " : {}",
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::Shl | Opcode::Shr => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let mut comma = false;
                for &arg in data.args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, false)?;
                }
                write!(
                    self.writer.sink,
                    " : ({}, {}, {}) -> {}",
                    MLIRType(&unit.value_type(data.args()[0])),
                    MLIRType(&unit.value_type(data.args()[1])),
                    MLIRType(&unit.value_type(data.args()[2])),
                    MLIRType(&unit.value_type(unit.inst_result(inst)))
                )?;
            }
            Opcode::Mux => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_value_use(data.args()[1], false)?;
                write!(
                    self.writer.sink,
                    " : ({}, {}) -> {}",
                    MLIRType(&unit.value_type(data.args()[0])),
                    MLIRType(&unit.value_type(data.args()[1])),
                    MLIRType(&unit.inst_type(inst))
                )?;
            }
            Opcode::Reg => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                self.write_value_use(data.args()[0], false)?;
                for t in data.triggers() {
                    write!(self.writer.sink, ", (")?;
                    self.write_value_use(t.data, false)?;
                    write!(self.writer.sink, ", \"{}\" ", t.mode)?;
                    self.write_value_use(t.trigger, false)?;
                    // TODO: MLIR dialect requires a time delay
                    if let Some(gate) = t.gate {
                        write!(self.writer.sink, ", if ")?;
                        self.write_value_use(gate, false)?;
                    }
                    write!(
                        self.writer.sink,
                        " : {})",
                        MLIRType(&unit.value_type(t.data))
                    )?;
                }
                write!(
                    self.writer.sink,
                    " : {}",
                    MLIRType(&unit.value_type(data.args()[0]))
                )?;
            }
            Opcode::InsField | Opcode::InsSlice => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let mut first = true;
                for &arg in data.args().iter() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, false)?;
                    first = false;
                }
                write!(
                    self.writer.sink,
                    ", {} : {}, {}",
                    data.imms()[0],
                    MLIRType(&unit.value_type(data.args()[0])),
                    MLIRType(&unit.value_type(data.args()[1]))
                )?;
            }
            Opcode::ExtField | Opcode::ExtSlice => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", {}", data.imms()[0])?;
                write!(
                    self.writer.sink,
                    " : {} -> {}",
                    MLIRType(&unit.value_type(data.args()[0])),
                    MLIRType(&unit.inst_type(inst))
                )?;
            }
            Opcode::Call => {
                write!(
                    self.writer.sink,
                    "{} {}(",
                    MLIROpcode(data.opcode()),
                    MLIRUnitName(&unit[data.get_ext_unit().unwrap()].name),
                )?;
                let mut comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, false)?;
                }
                write!(self.writer.sink, ") : (")?;
                comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                }
                let ty = unit.value_type(unit.inst_result(inst));
                let void_ty = crate::void_ty();
                write!(
                    self.writer.sink,
                    ") -> {}",
                    if unit.has_result(inst) {
                        MLIRType(&ty)
                    } else {
                        MLIRType(&void_ty)
                    }
                )?;
            }
            Opcode::Inst => {
                let inst_name = &self.uniquify_name(Some("inst"));
                write!(
                    self.writer.sink,
                    "{} \"{}\" {}(",
                    MLIROpcode(data.opcode()),
                    inst_name,
                    MLIRUnitName(&unit[data.get_ext_unit().unwrap()].name),
                )?;
                let mut comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, false)?;
                }
                write!(self.writer.sink, ") -> (")?;
                let mut comma = false;
                for &arg in data.output_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, false)?;
                }
                write!(self.writer.sink, ") : (")?;
                let mut comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                }
                write!(self.writer.sink, ") -> (")?;
                let mut comma = false;
                for &arg in data.output_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                }
                write!(self.writer.sink, ")")?;
            }
            Opcode::Halt | Opcode::Ret => {
                write!(self.writer.sink, "{}", MLIROpcode(data.opcode()))?
            }
            Opcode::Phi => {}
            Opcode::Br => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                let term_args = terminator_args
                    .get(&(curr_block, data.blocks()[0]))
                    .unwrap_or(&&def);
                self.write_block_value(data.blocks()[0], term_args)?;
            }
            Opcode::BrCond => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", ")?;
                let term_args = terminator_args
                    .get(&(curr_block, data.blocks()[1]))
                    .unwrap_or(&&def);
                self.write_block_value(data.blocks()[1], term_args)?;
                write!(self.writer.sink, ", ")?;
                let term_args = terminator_args
                    .get(&(curr_block, data.blocks()[0]))
                    .unwrap_or(&&def);
                self.write_block_value(data.blocks()[0], term_args)?;
            }
            Opcode::Wait => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                if data.args().len() > 0 {
                    write!(self.writer.sink, "(")?;
                    let mut first = true;
                    for &arg in data.args() {
                        if !first {
                            write!(self.writer.sink, ", ")?;
                        }
                        self.write_value_use(arg, false)?;
                        first = false;
                    }
                    write!(self.writer.sink, " : ")?;
                    first = true;
                    for &arg in data.args() {
                        if !first {
                            write!(self.writer.sink, ", ")?;
                        }
                        write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                        first = false;
                    }
                    write!(self.writer.sink, "), ")?;
                }
                let term_args = terminator_args
                    .get(&(curr_block, data.blocks()[0]))
                    .unwrap_or(&&def);
                self.write_block_value(data.blocks()[0], term_args)?;
            }
            Opcode::WaitTime => {
                write!(self.writer.sink, "{} ", MLIROpcode(data.opcode()))?;
                write!(self.writer.sink, " for ")?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", ")?;
                if data.args().len() > 1 {
                    write!(self.writer.sink, "(")?;
                    let mut first = true;
                    for &arg in &data.args()[1..] {
                        if !first {
                            write!(self.writer.sink, ", ")?;
                        }
                        self.write_value_use(arg, false)?;
                        first = false;
                    }
                    write!(self.writer.sink, " : ")?;
                    first = true;
                    for &arg in &data.args()[1..] {
                        if !first {
                            write!(self.writer.sink, ", ")?;
                        }
                        write!(self.writer.sink, "{}", MLIRType(&unit.value_type(arg)))?;
                        first = false;
                    }
                    write!(self.writer.sink, "), ")?;
                }
                let term_args = terminator_args
                    .get(&(curr_block, data.blocks()[0]))
                    .unwrap_or(&&def);
                self.write_block_value(data.blocks()[0], term_args)?;
            }
        }
        Ok(())
    }
}

/// Check if a character can be emitted in a name without escaping.
fn is_acceptable_name_char(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c >= '0' && c <= '9' || c == '_' || c == '.'
}

/// Escape the special characters in a name.
fn escape_name(input: &str) -> Rc<String> {
    let mut s = String::with_capacity(input.len());
    for c in input.chars() {
        if is_acceptable_name_char(c) {
            s.push(c);
        } else {
            s.push_str(&format!("\\{:x}", c as u32));
        }
    }
    Rc::new(s)
}
