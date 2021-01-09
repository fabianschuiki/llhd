// Copyright (c) 2017-2021 Fabian Schuiki

//! Var to Phi Promotion

use crate::{analysis::PredecessorTable, ir::prelude::*, opt::prelude::*};
use std::collections::{HashMap, HashSet};

/// Var to Phi Promotion
///
/// This pass tries to replace `var`, `ld`, and `st` instructions with `phi`
/// nodes as far as possible.
///
pub struct VarToPhiPromotion;

impl Pass for VarToPhiPromotion {
    fn run_on_cfg(_ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("VTPP [{}]", unit.name());
        let mut modified = false;

        // Build the predecessor table and dominator tree.
        let pt = unit.predtbl();

        // Trace variable values within each basic block, and assign potential
        // values to each of the loads.
        let dfg = &unit;
        let mut block_outs = HashMap::new();
        let mut value_table = HashMap::new();
        let mut vars = HashMap::new();
        for block in unit.blocks() {
            let mut store_table = HashMap::new();
            for inst in unit.insts(block) {
                let data = &dfg[inst];
                match data.opcode() {
                    Opcode::Var => {
                        store_table.insert(unit.inst_result(inst), data.args()[0]);
                        vars.entry(inst).or_insert_with(Vec::new);
                    }
                    Opcode::St => {
                        let var = data.args()[0];
                        let val = data.args()[1];
                        store_table.insert(var, val);
                        vars.entry(unit.value_inst(var))
                            .or_insert_with(Vec::new)
                            .push(inst);
                    }
                    Opcode::Ld => {
                        let v = match store_table.get(&data.args()[0]) {
                            Some(&v) => Var::Value(v),
                            None => Var::Incoming(data.args()[0], block),
                        };
                        value_table.insert(unit.inst_result(inst), v);
                    }
                    _ => continue,
                }
            }
            block_outs.insert(block, store_table);
        }

        trace!("Value table:");
        for (&ld, &v) in &value_table {
            let v = match v {
                Var::Incoming(var, bb) => format!("{} into {}", var.dump(&unit), bb.dump(&unit)),
                Var::Value(v) => format!("{}", v.dump(&unit)),
            };
            trace!("  ld {} = {}", ld.dump(&unit), v);
        }

        trace!("Variables leaving blocks:");
        for (&block, vars) in &block_outs {
            trace!("  Block {}:", block.dump(&unit));
            for (&var, &value) in vars {
                trace!("    st {} = {}", var.dump(&unit), value.dump(&unit));
            }
        }

        // Replace loads with the corresponding values which are live at the
        // respective locations.
        for (ld, slot) in value_table {
            trace!("Replacing {} with {:?}", ld.dump(&unit), slot);
            let inst = unit.value_inst(ld);
            let value = match slot {
                Var::Incoming(var, bb) => {
                    materialize_value(unit, &pt, var, bb, &block_outs, &mut HashSet::new())
                        .expect("cannot materialize var value")
                }
                Var::Value(v) => v,
            };
            debug!("Replacing {} with {}", inst.dump(&unit), value.dump(&unit));
            unit.replace_use(ld, value);
            unit.prune_if_unused(inst);
            modified |= true;
        }

        // Strip away all variables.
        for (var_inst, store_insts) in vars {
            for store_inst in store_insts {
                debug!("Removing {}", store_inst.dump(&unit));
                unit.delete_inst(store_inst);
            }
            debug!("Removing {}", var_inst.dump(&unit));
            unit.delete_inst(var_inst);
            modified |= true;
        }

        modified
    }
}

#[derive(Debug, Clone, Copy)]
enum Var {
    // The variable's value is determined by a block's predecessors.
    Incoming(Value, Block),
    // The variable's value is determined by an earlier value store.
    Value(Value),
}

/// Ensure that the value of a variable is available in a specified block.
fn materialize_value(
    unit: &mut UnitBuilder,
    pt: &PredecessorTable,
    var: Value,
    block: Block,
    block_outs: &HashMap<Block, HashMap<Value, Value>>,
    stack: &mut HashSet<Block>,
) -> Option<Value> {
    // Break recursion. If we arrive here there was a recursion in the CFG but
    // no store for the variable which would provide a new value. In this case
    // we simply return `None` to indicate that there is no value to be gotten
    // from this control flow path.
    if stack.contains(&block) {
        trace!("  Breaking recursion at {}", block.dump(&unit));
        return None;
    }
    trace!("  Materialize {} in {}", var.dump(&unit), block.dump(&unit));

    // Insert a recursion blocker.
    stack.insert(block);

    // Determine the value of the given variable in this block.
    let incoming_values: Vec<_> = pt
        .pred(block)
        .flat_map(|bb| {
            block_outs
                .get(&bb)
                .and_then(|vars| vars.get(&var).cloned())
                .or_else(|| materialize_value(unit, pt, var, bb, block_outs, stack))
                .map(|v| (bb, v))
        })
        .collect();

    // Check if a phi node is needed by evaluating whether we need to
    // differentiate from different distinct values.
    let distinct_values: HashSet<Value> = incoming_values.iter().map(|&(_, v)| v).collect();
    let value = if distinct_values.is_empty() {
        None
    } else if distinct_values.len() == 1 {
        distinct_values.into_iter().next()
    } else {
        trace!("  Insert phi node in {}", block.dump(&unit));
        for &(from, value) in &incoming_values {
            trace!(
                "    Incoming {} from {}",
                value.dump(&unit),
                from.dump(&unit)
            );
        }
        unit.prepend_to(block);
        let phi = unit.ins().phi(
            incoming_values.iter().map(|&(_, v)| v).collect(),
            incoming_values.iter().map(|&(bb, _)| bb).collect(),
        );
        debug!(
            "Insert {} in {}",
            unit.value_inst(phi).dump(&unit),
            block.dump(&unit)
        );
        Some(phi)
    };

    // Remove the recursion blocker.
    stack.remove(&block);
    value
}
