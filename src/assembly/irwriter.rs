// Copyright (c) 2017-2019 Fabian Schuiki

//! Emitting LLHD IR assembly.

use crate::ir::{prelude::*, ModUnitData};
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
        let mut was_declaration = false;
        for mod_unit in module.units() {
            let is_declaration = module.is_declaration(mod_unit);
            if separate && (!was_declaration || !is_declaration) {
                write!(self.sink, "\n")?;
            }
            separate = true;
            was_declaration = is_declaration;
            match &module[mod_unit] {
                ModUnitData::Function(x) => self.write_function(x)?,
                ModUnitData::Process(x) => self.write_process(x)?,
                ModUnitData::Entity(x) => self.write_entity(x)?,
                ModUnitData::Declare { sig, name } => self.write_declaration(sig, name)?,
            }
        }
        Ok(())
    }

    /// Emit assembly for a function.
    pub fn write_function(&mut self, func: &Function) -> Result<()> {
        let mut uw = UnitWriter::new(self, func);
        write!(uw.writer.sink, "func {} (", func.name())?;
        let mut comma = false;
        for arg in func.sig().inputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", func.sig().arg_type(arg))?;
            uw.write_value_name(func.dfg().arg_value(arg))?;
        }
        write!(uw.writer.sink, ") {} {{\n", func.sig().return_type())?;
        for block in func.layout.blocks() {
            uw.write_block_name(block)?;
            write!(uw.writer.sink, ":\n")?;
            for inst in func.layout.insts(block) {
                write!(uw.writer.sink, "    ")?;
                uw.write_inst(inst)?;
                write!(uw.writer.sink, "\n")?;
            }
        }
        write!(uw.writer.sink, "}}\n")?;
        Ok(())
    }

    /// Emit assembly for a process.
    pub fn write_process(&mut self, prok: &Process) -> Result<()> {
        let mut uw = UnitWriter::new(self, prok);
        write!(uw.writer.sink, "proc {} (", prok.name())?;
        let mut comma = false;
        for arg in prok.sig().inputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", prok.sig().arg_type(arg))?;
            uw.write_value_name(prok.dfg().arg_value(arg))?;
        }
        write!(uw.writer.sink, ") -> (")?;
        let mut comma = false;
        for arg in prok.sig().outputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", prok.sig().arg_type(arg))?;
            uw.write_value_name(prok.dfg().arg_value(arg))?;
        }
        write!(uw.writer.sink, ") {{\n")?;
        for block in prok.layout.blocks() {
            uw.write_block_name(block)?;
            write!(uw.writer.sink, ":\n")?;
            for inst in prok.layout.insts(block) {
                write!(uw.writer.sink, "    ")?;
                uw.write_inst(inst)?;
                write!(uw.writer.sink, "\n")?;
            }
        }
        write!(uw.writer.sink, "}}\n")?;
        Ok(())
    }

    /// Emit assembly for a entity.
    pub fn write_entity(&mut self, ent: &Entity) -> Result<()> {
        let mut uw = UnitWriter::new(self, ent);
        write!(uw.writer.sink, "entity {} (", ent.name())?;
        let mut comma = false;
        for arg in ent.sig().inputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", ent.sig().arg_type(arg))?;
            uw.write_value_name(ent.dfg().arg_value(arg))?;
        }
        write!(uw.writer.sink, ") -> (")?;
        let mut comma = false;
        for arg in ent.sig().outputs() {
            if comma {
                write!(uw.writer.sink, ", ")?;
            }
            comma = true;
            write!(uw.writer.sink, "{} ", ent.sig().arg_type(arg))?;
            uw.write_value_name(ent.dfg().arg_value(arg))?;
        }
        write!(uw.writer.sink, ") {{\n")?;
        for inst in ent.layout.insts() {
            write!(uw.writer.sink, "    ")?;
            uw.write_inst(inst)?;
            write!(uw.writer.sink, "\n")?;
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

pub struct UnitWriter<'a, T, U> {
    writer: &'a mut Writer<T>,
    unit: &'a U,
    value_names: HashMap<Value, Rc<String>>,
    block_names: HashMap<Block, Rc<String>>,
    name_indices: HashMap<Rc<String>, usize>,
    names: HashSet<Rc<String>>,
    tmp_index: usize,
}

impl<'a, T: Write, U: Unit> UnitWriter<'a, T, U> {
    /// Create a new writer for a unit.
    pub fn new(writer: &'a mut Writer<T>, unit: &'a U) -> Self {
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
        let name = self.uniquify_name(self.unit.dfg().get_name(value).map(AsRef::as_ref));

        // Emit the name and associate it with the value for later reuse.
        write!(self.writer.sink, "%{}", name)?;
        self.value_names.insert(value, name);
        Ok(())
    }

    /// Emit the name of a BB.
    pub fn write_block_name(&mut self, block: Block) -> Result<()> {
        // If we have already picked a name for the value, use that.
        if let Some(name) = self.block_names.get(&block) {
            return write!(self.writer.sink, "%{}", name);
        }

        // Check if the block has an explicit name set, or if we should just
        // generate a temporary name.
        let name = self.uniquify_name(None);

        // Emit the name and associate it with the block for later reuse.
        write!(self.writer.sink, "%{}", name)?;
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
                if !self.names.contains(&name) {
                    break name;
                }
            }
        } else {
            loop {
                let name = Rc::new(format!("{}", self.tmp_index));
                self.tmp_index += 1;
                if !self.names.contains(&name) {
                    break name;
                }
            }
        }
    }

    /// Emit the use of a value.
    pub fn write_value_use(&mut self, value: Value, with_type: bool) -> Result<()> {
        if with_type {
            write!(self.writer.sink, "{} ", self.unit.dfg().value_type(value))?;
        }
        self.write_value_name(value)
    }

    /// Emit an instruction.
    pub fn write_inst(&mut self, inst: Inst) -> Result<()> {
        let dfg = self.unit.dfg();
        if dfg.has_result(inst) {
            self.write_value_name(dfg.inst_result(inst))?;
            write!(self.writer.sink, " = ")?;
        }
        let data = &dfg[inst];
        match data.opcode() {
            Opcode::ConstInt => write!(
                self.writer.sink,
                "{} {} {}",
                data.opcode(),
                dfg.value_type(dfg.inst_result(inst)),
                data.get_const_int().unwrap()
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
                let iter = data
                    .data_args()
                    .iter()
                    .cloned()
                    .zip(data.mode_args().iter().cloned())
                    .zip(data.trigger_args().iter().cloned());
                for ((value, mode), trigger) in iter {
                    write!(self.writer.sink, ", ")?;
                    self.write_value_use(value, false)?;
                    write!(self.writer.sink, " {} ", mode)?;
                    self.write_value_use(trigger, false)?;
                }
            }
            Opcode::InsField | Opcode::InsSlice | Opcode::ExtField | Opcode::ExtSlice => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                let mut first = true;
                for &arg in data.args() {
                    if !first {
                        write!(self.writer.sink, ", ")?;
                    }
                    self.write_value_use(arg, first)?;
                    first = false;
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
                    dfg.value_type(dfg.inst_result(inst)),
                    dfg[data.get_ext_unit().unwrap()].name,
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
                    dfg[data.get_ext_unit().unwrap()].name,
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
            Opcode::Br => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_block_name(data.blocks()[0])?;
            }
            Opcode::BrCond => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_value_use(data.args()[0], false)?;
                write!(self.writer.sink, ", ")?;
                self.write_block_name(data.blocks()[0])?;
                write!(self.writer.sink, ", ")?;
                self.write_block_name(data.blocks()[1])?;
            }
            Opcode::Wait | Opcode::WaitTime => {
                write!(self.writer.sink, "{} ", data.opcode())?;
                self.write_block_name(data.blocks()[0])?;
                let mut comma = false;
                for &arg in data.args() {
                    if comma {
                        write!(self.writer.sink, ", ")?;
                    }
                    comma = true;
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
