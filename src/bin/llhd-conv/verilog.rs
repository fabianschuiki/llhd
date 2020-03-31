// Copyright (c) 2017-2020 Fabian Schuiki

//! Verilog output writer

use anyhow::{bail, Result};
use itertools::Itertools;
use llhd::ir::Unit;
use std::{collections::HashMap, io::Write};

/// Emit a module as Verilog code.
pub fn write(output: &mut impl Write, module: &llhd::ir::Module) -> Result<()> {
    debug!("Emitting Verilog code");
    let mut skipped = vec![];
    for mod_unit in module.units() {
        if let Some(entity) = module.get_entity(mod_unit) {
            if entity.name().is_global() {
                write_entity(output, entity, &mut Context::default())?;
            }
        } else {
            let name = module[mod_unit].name();
            error!("Unit {} not supported", name);
            skipped.push(name);
        }
    }
    if !skipped.is_empty() {
        bail!(
            "Units not supported in Verilog output: {}",
            skipped.iter().format(", ")
        );
    }
    Ok(())
}

#[derive(Default)]
struct Context {}

/// Emit an LLHD entity as a new Verilog module.
fn write_entity(
    output: &mut impl Write,
    entity: &llhd::ir::Entity,
    ctx: &mut Context,
) -> Result<()> {
    let name = sanitize_name(entity.name());
    debug!("Creating entity {} as `{}`", entity.name(), name);
    write!(output, "module {} (\n", name)?;
    write!(output, ");\n")?;
    write_entity_body(output, entity, ctx, Default::default())?;
    write!(output, "\nendmodule\n\n")?;
    Ok(())
}

/// Emit an LLHD entity within an existing Verilog module.
fn write_entity_body(
    output: &mut impl Write,
    entity: &llhd::ir::Entity,
    ctx: &mut Context,
    bound: HashMap<llhd::ir::Value, llhd::ir::Value>,
) -> Result<()> {
    debug!("Emitting entity {}", entity.name());
    write!(output, "\n    // Entity {}\n", entity.name())?;
    let dfg = entity.dfg();
    for inst in entity.inst_layout().insts() {
        write!(output, "    // {}\n", inst.dump(dfg, None))?;
    }
    Ok(())
}

/// Make a unit name printable in Verilog.
fn sanitize_name(name: &llhd::ir::UnitName) -> String {
    let mut out = String::new();
    if !name.is_global() {
        out.push('_');
    }
    match name {
        llhd::ir::UnitName::Global(s) | llhd::ir::UnitName::Local(s) => {
            out.extend(s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }))
        }
        llhd::ir::UnitName::Anonymous(i) => out.push_str(&i.to_string()),
    }
    out
}
