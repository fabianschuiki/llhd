// Copyright (c) 2017-2021 Fabian Schuiki

//! Control Flow Simplification

use crate::{analysis::PredecessorTable, ir::prelude::*, opt::prelude::*};
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
};

/// Control Flow Simplification
///
/// This pass tries to do the following:
///
/// - Simplify phi nodes whose values dominate the node
/// - Eliminate phi nodes where the result is constant across all incoming edges
/// - Merge basic blocks with only one successor
///
pub struct ControlFlowSimplification;

impl Pass for ControlFlowSimplification {
    fn run_on_cfg(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        info!("CFS [{}]", unit.name());
        let mut modified = false;

        // Build the predecessor table and dominator tree.
        let pt = unit.predtbl();
        let dt = unit.domtree_with_predtbl(&pt);
        let bn = BlockNumbering::new(unit);

        // Collect the phi instructions. We do this by gathering the values a
        // phi node can produce, and noting which edges lead to this value, then
        // transitively do this for nested phi nodes.
        let mut phi_ways = vec![];
        for block in unit.blocks() {
            let imm_dom = match dt
                .dominators(block)
                .iter()
                .cloned()
                .filter(|&bb| bb != block)
                .max_by_key(|&bb| bn[bb])
            {
                Some(bb) => bb,
                None => continue,
            };
            for inst in unit.insts(block) {
                if !unit[inst].opcode().is_phi() {
                    continue;
                }
                let ways = prepare_phi(ctx, unit, block, inst, &pt, imm_dom);
                phi_ways.push((inst, ways));
            }
        }

        // Build the discrimination tree for each phi node and replace all
        // covered values with the discriminator, which is now control-flow
        // independent.
        for (inst, ways) in phi_ways {
            trace!("Implementing {} as multiplexer", inst.dump(&unit),);
            unit.insert_before(inst);
            let disc = build_discriminator(ctx, unit, &ways);
            for (v, _) in ways {
                unit.replace_value_within_inst(v, disc, inst);
            }
            modified |= true;
        }

        // Finally simplify phi nodes which produce the same value irrelevant of
        // the incoming edge.
        let mut elide_phis = vec![];
        for block in unit.blocks() {
            for inst in unit.insts(block) {
                if !unit[inst].opcode().is_phi() {
                    continue;
                }
                if let Some(with) = maybe_elide_phi(ctx, unit, inst) {
                    elide_phis.push((inst, with));
                }
            }
        }
        for (inst, with) in elide_phis {
            trace!("Replace {} with {}", inst.dump(&unit), with.dump(&unit),);
            let inst_value = unit.inst_result(inst);
            unit.replace_use(inst_value, with);
            unit.prune_if_unused(inst);
            modified |= true;
        }

        modified
    }
}

// Find the preconditions for the values a phi node can produce. The resulting
// list may be non-exhaustive in case of difficult phi nodes.
fn prepare_phi(
    ctx: &PassContext,
    unit: &UnitBuilder,
    block: Block,
    inst: Inst,
    pt: &PredecessorTable,
    immediate_dominator: Block,
) -> Vec<(Value, Vec<Cond>)> {
    trace!(
        "Working on {} in {} against {}",
        inst.dump(&unit),
        block.dump(&unit),
        immediate_dominator.dump(&unit)
    );

    // Try to find the transitive branch condition that leads to control flow in
    // `immediate_dominator` to reach `block` via each of the edges in the phi
    // node.
    let mut ways = vec![];
    let data = &unit[inst];
    for (&bb, &arg) in data.blocks().iter().zip(data.args().iter()) {
        trace!("  Checking from {}", bb.dump(&unit));
        let routes = justify_edge(ctx, unit, bb, block, immediate_dominator, &mut vec![], pt);
        ways.extend(routes.into_iter().map(|route| (arg, route)));
    }

    trace!("    Found {:?}", ways);
    ways
}

// Find the chain of conditions that must be true to arrive at the `to` block,
// coming from the `from` block, ultimately originating in the `target` block.
fn justify_edge(
    ctx: &PassContext,
    unit: &UnitBuilder,
    from: Block,
    to: Block,
    target: Block,
    seen: &mut Vec<Block>,
    pt: &PredecessorTable,
) -> Vec<Vec<Cond>> {
    trace!("    Justifying {} -> {}", from.dump(&unit), to.dump(&unit));

    // Investigate the terminator of the `from` block to see under what
    // condition it transfers control to `to`.
    let from_term = unit.terminator(from);
    let data = &unit[from_term];
    let cond = match data.opcode() {
        // Unconditional branches and waits are trivial, since the transfer
        // control flow in any case.
        Opcode::Br | Opcode::Wait | Opcode::WaitTime => None,

        // Conditional branches need further inspection.
        Opcode::BrCond if data.blocks()[0] == to => Some(Cond::Neg(data.args()[0])),
        Opcode::BrCond if data.blocks()[1] == to => Some(Cond::Pos(data.args()[0])),

        _ => unreachable!("weird terminator found"),
    };

    // If we have arrived at the target then we are done.
    if from == target {
        return vec![cond.into_iter().collect()];
    }

    // Gather the conditions to arrive from each of the predecessors to the from
    // block.
    let mut routes = vec![];
    seen.push(to);
    for bb in pt.pred(from) {
        if seen.contains(&bb) {
            continue;
        }
        for mut route in justify_edge(ctx, unit, bb, from, target, seen, pt) {
            if let Some(cond) = cond {
                route.push(cond);
            }
            routes.push(route);
        }
    }
    seen.pop();
    routes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cond {
    Pos(Value),
    Neg(Value),
}

fn build_discriminator(
    ctx: &PassContext,
    unit: &mut UnitBuilder,
    ways: &[(Value, Vec<Cond>)],
) -> Value {
    trace!("  Discriminating {:?}", ways);

    // Short cut the easy cases where there is nothing to discriminate.
    if ways.len() == 1 {
        return ways[0].0;
    }

    // Find the largest discriminating factor of each way.
    let mut table = HashMap::<Value, (usize, isize)>::new();
    for (_, conds) in ways {
        for &cond in conds {
            let (v, tick) = match cond {
                Cond::Pos(v) => (v, 1),
                Cond::Neg(v) => (v, -1),
            };
            let e = table.entry(v).or_insert((0, 0));
            e.0 += 1;
            e.1 += tick;
        }
    }
    let (disc, (_uses, _imbalance)) = table
        .into_iter()
        .map(|(v, (n, tick))| (v, (n, -tick.abs())))
        .max_by_key(|&(_, x)| x)
        .expect("some discriminator must be present");
    trace!("    Discriminator is {} ({})", disc, disc.dump(&unit));

    // Split the ways over the discriminator.
    let mux_conds = [Cond::Neg(disc), Cond::Pos(disc)];
    let mux_values: Vec<_> = mux_conds
        .iter()
        .map(|&cond| {
            let mux_ways: Vec<_> = ways
                .iter()
                .flat_map(|(v, conds)| -> Option<(Value, Vec<Cond>)> {
                    if conds.contains(&cond) {
                        Some((*v, conds.iter().cloned().filter(|&c| c != cond).collect()))
                    } else {
                        None
                    }
                })
                .collect();
            trace!("      {:?}: {:?}", cond, mux_ways);
            build_discriminator(ctx, unit, &mux_ways)
        })
        .collect();

    // Build the multiplexer which picks among the values.
    let arr = unit.ins().array(mux_values);
    let mux = unit.ins().mux(arr, disc);
    mux
}

/// Check if a phi node can be elided because it produces the same value no
/// matter what the incoming edge is.
fn maybe_elide_phi(_ctx: &PassContext, unit: &UnitBuilder, inst: Inst) -> Option<Value> {
    let set: HashSet<Value> = unit[inst].args().iter().cloned().collect();
    if set.len() == 1 {
        set.into_iter().next()
    } else {
        None
    }
}

/// An ordering and numbering of the basic blocks in control-flow order.
pub struct BlockNumbering {
    numbers: HashMap<Block, usize>,
    order: Vec<Block>,
}

impl BlockNumbering {
    /// Compute a block order and numbering.
    pub fn new(unit: &Unit) -> Self {
        let mut numbers = HashMap::<Block, usize>::new();
        let mut order = vec![];
        let mut done = HashSet::<Block>::new();
        let mut pending = HashSet::<Block>::new();
        let entry = unit.entry();
        pending.insert(entry);
        numbers.insert(entry, 0);

        while let Some(&block) = pending.iter().next() {
            pending.remove(&block);
            done.insert(block);
            order.push(block);
            let term = unit.terminator(block);
            if unit[term].opcode().is_terminator() {
                pending.extend(
                    unit[term]
                        .blocks()
                        .iter()
                        .cloned()
                        .filter(|bb| !done.contains(bb)),
                );
                let next_number = numbers[&block] + 1;
                for bb in unit[term].blocks().iter().cloned() {
                    numbers.entry(bb).or_insert(next_number);
                }
            }
        }

        BlockNumbering { numbers, order }
    }

    /// Get the number associated with a block.
    pub fn number(&self, block: Block) -> usize {
        self.numbers[&block]
    }

    /// Get the number associated with a block.
    pub fn get_number(&self, block: Block) -> Option<usize> {
        self.numbers.get(&block).cloned()
    }

    /// Get the control flow ordering of the blocks.
    pub fn order(&self) -> impl Iterator<Item = Block> + '_ {
        self.order.iter().cloned()
    }

    /// Get the control flow ordering of the blocks as a slice.
    pub fn order_slice(&self) -> &[Block] {
        &self.order
    }
}

impl Index<Block> for BlockNumbering {
    type Output = usize;
    fn index(&self, idx: Block) -> &usize {
        &self.numbers[&idx]
    }
}
