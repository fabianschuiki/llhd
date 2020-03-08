// Copyright (c) 2017-2019 Fabian Schuiki

//! Desequentialization

use crate::ir::prelude::*;
use crate::ir::{DataFlowGraph, ModUnitData};
use crate::opt::prelude::*;
use crate::pass::tcm::{TemporalRegion, TemporalRegionGraph};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

/// Desequentialization
///
/// This pass implements detection of state-keeping behaviour in processes and
/// the extraction of such state into explicit `reg` instructions.
pub struct Desequentialization;

impl Pass for Desequentialization {
    fn run_on_module(ctx: &PassContext, module: &mut Module) -> bool {
        module
            .units
            .storage
            .par_iter_mut()
            .map(|(_, unit)| match unit {
                ModUnitData::Process(ref mut u) => {
                    deseq_process(ctx, &mut ProcessBuilder::new(u)).is_some()
                }
                _ => false,
            })
            .reduce(|| false, |a, b| a || b)
    }
}

fn deseq_process(ctx: &PassContext, unit: &mut ProcessBuilder) -> Option<Entity> {
    info!("Deseq [{}]", unit.unit().name());

    // Find the canonicalized drive conditions.
    let layout = unit.func_layout();
    for bb in layout.blocks() {
        for inst in layout.insts(bb) {
            let data = &unit.dfg()[inst];
            if data.opcode() == Opcode::DrvCond {
                trace!(
                    "Canonicalizing condition of {}",
                    inst.dump(unit.dfg(), unit.try_cfg())
                );
                canonicalize(ctx, unit, data.args()[3]);
            }
        }
    }
    None
}

/// Canonicalize the conditions of a drive.
///
/// This function attempts to bring the drive condition into disjunctive normal
/// form (DNF), and establish equality/inequality relationships with input
/// signals where possible.
fn canonicalize(ctx: &PassContext, unit: &ProcessBuilder, cond: Value) {
    let trg = TemporalRegionGraph::new(unit.dfg(), unit.func_layout());
    canonicalize_term(ctx, unit, &trg, cond, false);
}

fn canonicalize_term(
    ctx: &PassContext,
    unit: &ProcessBuilder,
    trg: &TemporalRegionGraph,
    cond: Value,
    inv: bool,
) -> Dnf {
    let dfg = unit.dfg();
    let dnf = canonicalize_term_inner(ctx, unit, trg, cond, inv);
    let desc = if let Some(inst) = dfg.get_value_inst(cond) {
        inst.dump(dfg, unit.try_cfg()).to_string()
    } else {
        cond.dump(dfg).to_string()
    };
    trace!(
        "  {} {{ {} }} => {}",
        if inv { "neg" } else { "pos" },
        desc,
        dnf.dump(dfg),
    );
    dnf
}

fn canonicalize_term_inner(
    ctx: &PassContext,
    unit: &ProcessBuilder,
    trg: &TemporalRegionGraph,
    cond: Value,
    inv: bool,
) -> Dnf {
    let dfg = unit.dfg();

    // Don't bother with values of the wrong type.
    let ty = dfg.value_type(cond);
    if ty != crate::ty::int_ty(1) {
        return Dnf::single(Term::Invalid(cond), inv);
    }

    // Canonicalize instructions.
    if let Some(inst) = dfg.get_value_inst(cond) {
        let data = &dfg[inst];
        match data.opcode() {
            Opcode::ConstInt => {
                return Dnf::single(Term::Zero, data.get_const_int().unwrap().is_one() ^ inv);
            }
            Opcode::Not => return canonicalize_term(ctx, unit, trg, data.args()[0], !inv),
            Opcode::And | Opcode::Or => {
                let lhs = canonicalize_term(ctx, unit, trg, data.args()[0], inv);
                let rhs = canonicalize_term(ctx, unit, trg, data.args()[1], inv);
                let out = match (data.opcode(), inv) {
                    (Opcode::And, false) | (Opcode::Or, true) => Dnf::and(&lhs, &rhs),
                    (Opcode::And, true) | (Opcode::Or, false) => Dnf::or(&lhs, &rhs),
                    _ => unreachable!(),
                };
                return out;
            }
            Opcode::Xor | Opcode::Eq | Opcode::Neq => {
                let lhs_pos = canonicalize_term(ctx, unit, trg, data.args()[0], false);
                let rhs_pos = canonicalize_term(ctx, unit, trg, data.args()[1], false);
                let lhs_neg = canonicalize_term(ctx, unit, trg, data.args()[0], true);
                let rhs_neg = canonicalize_term(ctx, unit, trg, data.args()[1], true);
                let polarity = match data.opcode() {
                    Opcode::Eq => !inv,
                    _ => inv,
                };
                let out = if polarity {
                    Dnf::or(&Dnf::and(&lhs_pos, &rhs_pos), &Dnf::and(&lhs_neg, &rhs_neg))
                } else {
                    Dnf::or(&Dnf::and(&lhs_pos, &rhs_neg), &Dnf::and(&lhs_neg, &rhs_pos))
                };
                return out;
            }
            Opcode::Prb => {
                let bb = unit.func_layout().inst_block(inst).unwrap();
                return Dnf::single(Term::Signal(data.args()[0], trg[bb]), inv);
            }
            _ => (),
        }
    }
    Dnf::single(Term::Invalid(cond), inv)
}

/// An expression in disjunctive normal form.
///
/// A constant `0` is represented as `{}`. A constant `1` is represented as
/// `{{}}`.
struct Dnf(BTreeSet<BTreeMap<Term, bool>>);

impl Dnf {
    /// Create the zero expression `0`.
    pub fn zero() -> Dnf {
        Dnf(BTreeSet::new())
    }

    /// Create the identity expression `1`.
    pub fn one() -> Dnf {
        let mut set = BTreeSet::new();
        set.insert(Default::default());
        Dnf(set)
    }

    /// Create a single-term expression.
    pub fn single(term: Term, inv: bool) -> Dnf {
        match term {
            Term::Zero if inv => Self::one(),
            Term::Zero => Self::zero(),
            _ => {
                let mut set = BTreeSet::new();
                set.insert(Some((term, inv)).into_iter().collect());
                Dnf(set)
            }
        }
    }

    pub fn dump<'a>(&'a self, dfg: &'a DataFlowGraph) -> DnfDumper<'a> {
        DnfDumper(self, dfg)
    }

    /// Compute the boolean OR of two DNF expressions.
    pub fn or(lhs: &Dnf, rhs: &Dnf) -> Dnf {
        let lhs = lhs.0.iter().cloned();
        let rhs = rhs.0.iter().cloned();
        Dnf(lhs.chain(rhs).collect())
    }

    /// Compute the boolean AND of two DNF expressions.
    pub fn and(lhs: &Dnf, rhs: &Dnf) -> Dnf {
        let mut set = BTreeSet::new();
        for lhs_term in &lhs.0 {
            for rhs_term in &rhs.0 {
                if let Some(term) = Self::and_term(lhs_term, rhs_term) {
                    set.insert(term);
                }
            }
        }
        Dnf(set)
    }

    /// Compute the boolean AND between two inner terms.
    fn and_term(
        lhs: &BTreeMap<Term, bool>,
        rhs: &BTreeMap<Term, bool>,
    ) -> Option<BTreeMap<Term, bool>> {
        let mut out = BTreeMap::new();
        for (term, &inv) in lhs.iter().chain(rhs.iter()) {
            // If we insert a term whose complement is already in the AND
            // expression, the resulting expression is always false.
            if out.insert(term.clone(), inv) == Some(!inv) {
                return None;
            }
        }
        Some(out)
    }
}

struct DnfDumper<'a>(&'a Dnf, &'a DataFlowGraph);

impl std::fmt::Display for DnfDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::iter::{once, repeat};
        if (self.0).0.is_empty() {
            return write!(f, "0");
        }
        if (self.0).0.len() == 1 {
            if (self.0).0.iter().next().unwrap().is_empty() {
                return write!(f, "1");
            }
        }
        for (vs, sep) in (self.0).0.iter().zip(once("").chain(repeat(" | "))) {
            write!(f, "{}(", sep)?;
            for ((term, inv), sep) in vs.iter().zip(once("").chain(repeat(" & "))) {
                write!(f, "{}", sep)?;
                if *inv {
                    write!(f, "!")?;
                }
                match term {
                    Term::Zero => write!(f, "0")?,
                    Term::Signal(sig, tr) => write!(f, "{}@{}", sig.dump(self.1), tr)?,
                    Term::Invalid(v) => write!(f, "{}?", v.dump(self.1))?,
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Term {
    Zero,
    Signal(Value, TemporalRegion),
    Invalid(Value),
}
