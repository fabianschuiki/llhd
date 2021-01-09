// Copyright (c) 2017-2021 Fabian Schuiki

//! Emitting LLHD IR assembly.

use crate::ir::{prelude::*, UnitKind};
use std::{
    collections::{HashMap, HashSet},
    io::{Result, Write},
    rc::Rc,
};

/// Temporary object to emit LLHD IR assembly.
pub struct Writer<T> {
    sink: T,
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
        write!(uw.writer.sink, "{} {} (", data.kind(), data.name())?;
        let mut comma = false;
        for arg in data.sig().inputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", data.sig().arg_type(arg))?;
            uw.write_value_name(data.arg_value(arg))?;
        }
        if data.kind() == UnitKind::Function {
            write!(uw.writer.sink, ") {} {{\n", data.sig().return_type())?;
        } else {
            write!(uw.writer.sink, ") -> (")?;
            let mut comma = false;
            for arg in data.sig().outputs() {
                if comma {
                    write!(uw.writer.sink, ", ")?;
                }
                comma = true;
                write!(uw.writer.sink, "{} ", data.sig().arg_type(arg))?;
                uw.write_value_name(data.arg_value(arg))?;
            }
            write!(uw.writer.sink, ") {{\n")?;
        }
        for block in data.blocks() {
            if data.kind() != UnitKind::Entity {
                uw.write_block_name(block)?;
                write!(uw.writer.sink, ":\n")?;
            }
            for inst in data.insts(block) {
                if data[inst].opcode().is_terminator() && data.is_entity() {
                    continue;
                }
                write!(uw.writer.sink, "    ")?;
                uw.write_inst(inst)?;
                write!(uw.writer.sink, "\n")?;
            }
        }
        write!(uw.writer.sink, "}}\n")?;
        Ok(())
    }

    /// Emit assembly for a declaration.
    pub fn write_declaration(&mut self, sig: &Signature, name: &UnitName) -> Result<()> {
        write!(self.sink, "declare {} {}\n", name, sig)?;
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
    pub fn write_block_name(&mut self, block: Block) -> Result<()> {
        // If we have already picked a name for the value, use that.
        if let Some(name) = self.block_names.get(&block) {
            return write!(self.writer.sink, "{}", name);
        }

        // Check if the block has an explicit name set, or if we should just
        // generate a temporary name.
        let name = self.uniquify_name(self.unit.get_block_name(block));

        // Emit the name and associate it with the block for later reuse.
        write!(self.writer.sink, "{}", name)?;
        self.block_names.insert(block, name);
        Ok(())
    }

    /// Emit the name of a BB to be used as label in an instruction.
    pub fn write_block_value(&mut self, block: Block) -> Result<()> {
        write!(self.writer.sink, "%")?;
        self.write_block_name(block)
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
    pub fn write_inst(&mut self, inst: Inst) -> Result<()> {
        let unit = self.unit;
        if unit.has_result(inst) {
            self.write_value_name(unit.inst_result(inst))?;
            write!(self.writer.sink, " = ")?;
        }
        let data = &unit[inst];
        match data.opcode() {
            Opcode::ConstInt => write!(
                self.writer.sink,
                "{} {} {}",
                data.opcode(),
                unit.value_type(unit.inst_result(inst)),
                data.get_const_int().unwrap().value
            )?,
            Opcode::ConstTime => write!(
                self.writer.sink,
                "{} time {}",
                data.opcode(),
                data.get_const_time().unwrap()
            )?,
            Opcode::ArrayUniform => {
                write!(self.writer.sink, "[{} x ", data.imms()[0])?;
                self.write_value_use(data.args()[0], true)?;
                write!(self.writer.sink, "]")?;
            }
            Opcode::Array => {
                write!(self.writer.sink, "[")?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, first)?;
                    first = false;
                }
                write!(self.writer.sink, "]")?;
            }
            Opcode::Struct => {
                write!(self.writer.sink, "{{")?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    first = false;
                    self.write_value_use(arg, true)?;
                }
                write!(self.writer.sink, "}}")?;
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
            | Opcode::Con
            | Opcode::Del
            | Opcode::Sig
            | Opcode::Prb
            | Opcode::Drv
            | Opcode::Var
            | Opcode::Ld
            | Opcode::St
            | Opcode::RetValue => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, first)?;
                    first = false;
                }
            }
            Opcode::DrvCond => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                let args = data.args();
                self.write_value_use(args[0], true)?;
                write!(self.writer.sink, " if ")?;
                self.write_value_use(args[3], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_value_use(args[1], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_value_use(args[2], false)?;
            }
            Opcode::Shl | Opcode::Shr | Opcode::Mux => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                let mut comma = false;
                for &arg in data.args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, true)?;
                }
            }
            Opcode::Reg => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_value_use(data.args()[0], true)?;
                for t in data.triggers() {
                    write!(self.writer.sink, ", [")?;
                    self.write_value_use(t.data, false)?;
                    write!(self.writer.sink, ", {} ", t.mode)?;
                    self.write_value_use(t.trigger, false)?;
                    if let Some(gate) = t.gate {
                        write!(self.writer.sink, ", if ")?;
                        self.write_value_use(gate, false)?;
                    }
                    write!(self.writer.sink, "]")?;
                }
            }
            Opcode::InsField | Opcode::InsSlice => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, true)?;
                    first = false;
                }
                for &imm in data.imms() {
                    write!(self.writer.sink, ", {}", imm)?;
                }
            }
            Opcode::ExtField | Opcode::ExtSlice => {
                write!(
                    self.writer.sink,
                    "{} {}",
                    data.opcode(),
                    unit.inst_type(inst)
                )?;
                for &arg in data.args() {
                    write!(self.writer.sink, ", ")?;
                    self.write_value_use(arg, true)?;
                }
                for &imm in data.imms() {
                    write!(self.writer.sink, ", {}", imm)?;
                }
            }
            Opcode::Call => {
                write!(
                    self.writer.sink,
                    "{} {} {} (",
                    data.opcode(),
                    if unit.has_result(inst) {
                        unit.value_type(unit.inst_result(inst))
                    } else {
                        crate::void_ty()
                    },
                    unit[data.get_ext_unit().unwrap()].name,
                )?;
                let mut comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, true)?;
                }
                write!(self.writer.sink, ")")?;
            }
            Opcode::Inst => {
                write!(
                    self.writer.sink,
                    "{} {} (",
                    data.opcode(),
                    unit[data.get_ext_unit().unwrap()].name,
                )?;
                let mut comma = false;
                for &arg in data.input_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, true)?;
                }
                write!(self.writer.sink, ") -> (")?;
                let mut comma = false;
                for &arg in data.output_args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    self.write_value_use(arg, true)?;
                }
                write!(self.writer.sink, ")")?;
            }
            Opcode::Halt | Opcode::Ret => write!(self.writer.sink, "{}", data.opcode())?,
            Opcode::Phi => {
                write!(
                    self.writer.sink,
                    "{} {} ",
                    data.opcode(),
                    unit.value_type(unit.inst_result(inst))
                )?;
                let mut comma = false;
                for (&arg, &block) in data.args().iter().zip(data.blocks().iter()) {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
                    write!(self.writer.sink, "[")?;
                    self.write_value_use(arg, false)?;
                    write!(self.writer.sink, ", ")?;
                    self.write_block_value(block)?;
                    write!(self.writer.sink, "]")?;
                }
            }
            Opcode::Br => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_block_value(data.blocks()[0])?;
            }
            Opcode::BrCond => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_block_value(data.blocks()[0])?;
                write!(self.writer.sink, ", ")?;
                self.write_block_value(data.blocks()[1])?;
            }
            Opcode::Wait => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_block_value(data.blocks()[0])?;
                for &arg in data.args() {
                    write!(self.writer.sink, ", ")?;
                    self.write_value_use(arg, false)?;
                }
            }
            Opcode::WaitTime => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_block_value(data.blocks()[0])?;
                write!(self.writer.sink, " for ")?;
                self.write_value_use(data.args()[0], false)?;
                for &arg in &data.args()[1..] {
                    write!(self.writer.sink, ", ")?;
                    self.write_value_use(arg, false)?;
                }
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
