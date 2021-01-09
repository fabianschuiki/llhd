// Copyright (c) 2017-2021 Fabian Schuiki

//! Desequentialization

use crate::{
    analysis::{TemporalRegion, TemporalRegionGraph},
    ir::{prelude::*, InstData},
    opt::prelude::*,
    value::IntValue,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Desequentialization
///
/// This pass implements detection of state-keeping behaviour in processes and
/// the extraction of such state into explicit `reg` instructions.
pub struct Desequentialization;

impl Pass for Desequentialization {
    fn run_on_cfg(ctx: &PassContext, unit: &mut UnitBuilder) -> bool {
        if unit.kind() == UnitKind::Process {
            match deseq_process(ctx, unit) {
                Some(entity) => {
                    *unit.data() = entity;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

fn deseq_process(ctx: &PassContext, unit: &mut UnitBuilder) -> Option<UnitData> {
    info!("Deseq [{}]", unit.name());
    let trg = unit.trg();

    // Identify the relevant temporal regions.
    if trg.regions().count() != 2 {
        trace!("Skipping (incorrect number of TRs)");
        return None;
    }
    let (tr0, tr1) = {
        let mut it = trg.regions();
        (it.next().unwrap().id, it.next().unwrap().id)
    };
    if !trg[tr0].entry {
        trace!("Skipping (TR0 is not entry)");
        return None;
    }
    if trg[tr1].entry {
        trace!("Skipping (TR1 is entry)");
        return None;
    }
    trace!("Head region {}, trigger region {}", tr0, tr1);

    // Identify the wait instruction and the signals which may trigger a state
    // change.
    let (wait_inst, sensitivity) = {
        let mut it = trg[tr0].tail_insts();
        let inst = match it.next() {
            Some(i) => i,
            None => {
                trace!("Skipping ({} has no tail inst)", tr0);
                return None;
            }
        };
        let data = &unit[inst];
        let sensitivity: BTreeSet<_> = match data.opcode() {
            Opcode::Wait => data.args().iter().cloned().collect(),
            Opcode::WaitTime => data.args().iter().skip(1).cloned().collect(),
            _ => {
                trace!("Skipping ({} tail inst is not a wait)", tr0);
                return None;
            }
        };
        (inst, sensitivity)
    };
    trace!("Wait Inst: {}", wait_inst.dump(&unit));
    trace!("Sensitivity: {:?}", sensitivity);

    // Ensure that there is is only one basic block per temporal region.
    // Lowering more complicated scenarios is possible, but is left for a future
    // extension.
    let tr0_num_bb = trg[tr0].blocks().count();
    let tr1_num_bb = trg[tr1].blocks().count();
    if tr0_num_bb != 1 || tr1_num_bb != 1 {
        trace!(
            "Skipping ({} TR0 blocks, {} TR1 blocks, instead of 1 each)",
            tr0_num_bb,
            tr1_num_bb
        );
        return None;
    }

    // Find the canonicalized drive conditions.
    let mut all_drives = HashSet::new();
    let mut conds = vec![];
    for bb in unit.blocks() {
        for inst in unit.insts(bb) {
            let data = &unit[inst];
            if data.opcode() == Opcode::DrvCond {
                trace!("Canonicalizing condition of {}", inst.dump(&unit));
                conds.push((
                    inst,
                    bb,
                    canonicalize(ctx, unit, &trg, data.args()[3], false),
                ));
                all_drives.insert(inst);
            } else if data.opcode() == Opcode::Drv {
                all_drives.insert(inst);
            }
        }
    }

    // Detect the edges and levels for each drive that trigger a state change.
    let triggers: Vec<(Inst, Block, Vec<Trigger>)> = conds
        .iter()
        .flat_map(|(inst, bb, dnf)| {
            detect_triggers(ctx, unit, tr0, tr1, dnf).map(|trig| (*inst, *bb, trig))
        })
        .collect();

    // Create a replacement entity.
    let mut entity = UnitData::new(UnitKind::Entity, unit.name().clone(), unit.sig().clone());
    let mut builder = UnitBuilder::new_anonymous(&mut entity);
    for arg in unit.unit().sig().args() {
        if let Some(name) = unit.get_name(unit.arg_value(arg)) {
            let v = builder.arg_value(arg);
            builder.set_name(v, name.to_string());
        }
    }
    let mut mig = Migrator::new(unit, &mut builder, &trg, tr0, tr1);

    // For each drive where we successfully and exhaustively identified the
    // triggers, migrate the computation of each next state into a separate
    // entity.
    let mut migrated = true;
    for (inst, bb, trigs) in triggers {
        migrated &= mig.migrate_drive(inst, bb, &trigs);
    }
    // crate::pass::ConstFolding::run_on_entity(ctx, &mut builder);
    // crate::pass::DeadCodeElim::run_on_entity(ctx, &mut builder);

    // Check if all drives were migrated.
    // This will currently fail for any unconditional drives, since we don't yet
    // handle them properly.
    all_drives
        .difference(&mig.migrated_drives)
        .for_each(|inst| {
            migrated = false;
            trace!("Skipping ({} not migrated)", inst.dump(&unit));
        });

    if migrated {
        Some(entity)
    } else {
        trace!("Process {} not migrated", unit.name());
        None
    }
}

/// Canonicalize the conditions of a drive.
///
/// This function attempts to bring the drive condition into disjunctive normal
/// form (DNF), and establish equality/inequality relationships with input
/// signals where possible.
fn canonicalize(
    ctx: &PassContext,
    unit: &UnitBuilder,
    trg: &TemporalRegionGraph,
    cond: Value,
    inv: bool,
) -> Dnf {
    let dnf = canonicalize_inner(ctx, unit, trg, cond, inv);
    let desc = if let Some(inst) = unit.get_value_inst(cond) {
        inst.dump(&unit).to_string()
    } else {
        cond.dump(&unit).to_string()
    };
    trace!(
        "  {} {{ {} }} => {}",
        if inv { "neg" } else { "pos" },
        desc,
        dnf.dump(&unit),
    );
    dnf
}

fn canonicalize_inner(
    ctx: &PassContext,
    unit: &UnitBuilder,
    trg: &TemporalRegionGraph,
    cond: Value,
    inv: bool,
) -> Dnf {
    let dfg = unit;

    // Don't bother with values of the wrong type.
    let ty = unit.value_type(cond);
    if ty != crate::ty::int_ty(1) {
        return Dnf::single(Term::Invalid(cond), inv);
    }

    // Canonicalize instructions.
    if let Some(inst) = unit.get_value_inst(cond) {
        let data = &dfg[inst];
        match data.opcode() {
            Opcode::ConstInt => {
                return Dnf::single(Term::Zero, data.get_const_int().unwrap().is_one() ^ inv);
            }
            Opcode::Not => return canonicalize(ctx, unit, trg, data.args()[0], !inv),
            Opcode::And | Opcode::Or => {
                let lhs = canonicalize(ctx, unit, trg, data.args()[0], inv);
                let rhs = canonicalize(ctx, unit, trg, data.args()[1], inv);
                let out = match (data.opcode(), inv) {
                    (Opcode::And, false) | (Opcode::Or, true) => Dnf::and(&lhs, &rhs),
                    (Opcode::And, true) | (Opcode::Or, false) => Dnf::or(&lhs, &rhs),
                    _ => unreachable!(),
                };
                return out;
            }
            Opcode::Xor | Opcode::Eq | Opcode::Neq => {
                let lhs_pos = canonicalize(ctx, unit, trg, data.args()[0], false);
                let rhs_pos = canonicalize(ctx, unit, trg, data.args()[1], false);
                let lhs_neg = canonicalize(ctx, unit, trg, data.args()[0], true);
                let rhs_neg = canonicalize(ctx, unit, trg, data.args()[1], true);
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
                let bb = unit.inst_block(inst).unwrap();
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

    pub fn dump<'a>(&'a self, unit: &Unit<'a>) -> DnfDumper<'a> {
        DnfDumper(self, *unit)
    }

    pub fn dump_term<'a>(term: &'a BTreeMap<Term, bool>, unit: &Unit<'a>) -> DnfTermDumper<'a> {
        DnfTermDumper(term, *unit)
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

struct DnfDumper<'a>(&'a Dnf, Unit<'a>);

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
            write!(f, "{}({})", sep, Dnf::dump_term(vs, &self.1))?;
        }
        Ok(())
    }
}

struct DnfTermDumper<'a>(&'a BTreeMap<Term, bool>, Unit<'a>);

impl std::fmt::Display for DnfTermDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::iter::{once, repeat};
        if (self.0).is_empty() {
            return write!(f, "1");
        }
        for ((term, inv), sep) in self.0.iter().zip(once("").chain(repeat(" & "))) {
            write!(f, "{}", sep)?;
            if *inv {
                write!(f, "!")?;
            }
            match term {
                Term::Zero => write!(f, "0")?,
                Term::Signal(sig, tr) => write!(f, "{}@{}", sig.dump(&self.1), tr)?,
                Term::Invalid(v) => write!(f, "{}?", v.dump(&self.1))?,
            }
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

/// Detect the edge and level triggers described by a DNF.
fn detect_triggers(
    ctx: &PassContext,
    unit: &UnitBuilder,
    tr0: TemporalRegion,
    tr1: TemporalRegion,
    dnf: &Dnf,
) -> Option<Vec<Trigger>> {
    trace!("Detecting triggers in {}", dnf.dump(&unit));
    let mut trigs = vec![];
    for conds in &dnf.0 {
        let trig = match detect_term_triggers(ctx, unit, tr0, tr1, conds) {
            Some(trig) => trig,
            None => return None,
        };
        trigs.push(trig);
    }
    Some(trigs)
}

fn detect_term_triggers(
    _ctx: &PassContext,
    unit: &UnitBuilder,
    tr0: TemporalRegion,
    tr1: TemporalRegion,
    conds: &BTreeMap<Term, bool>,
) -> Option<Trigger> {
    trace!("  Analyzing {}", Dnf::dump_term(conds, &unit));

    // Sort the level and edge sensitive terms.
    let mut edges = BTreeMap::new();
    let mut levels = BTreeMap::new();
    for (term, &inv) in conds {
        match *term {
            // Signals sampled before the change must be accompanied by the
            // same signal sampled after the change, but inverted.
            Term::Signal(sig, tr) if tr == tr0 => {
                if conds.get(&Term::Signal(sig, tr1)).cloned() == Some(!inv) {
                    trace!(
                        "    {} {}",
                        if inv { "rising" } else { "falling" },
                        sig.dump(&unit)
                    );
                    edges.insert(
                        sig,
                        match inv {
                            true => TriggerEdge::Rise,
                            false => TriggerEdge::Fall,
                        },
                    );
                } else {
                    trace!(
                        "    Skipping ({}@{} without corresponding {}@{})",
                        sig.dump(&unit),
                        tr0,
                        sig.dump(&unit),
                        tr1
                    );
                    return None;
                }
            }

            // Signals sampled after the change without accompanying
            // sampling before the change contribute a level sensitivity.
            Term::Signal(sig, tr) if tr == tr1 => {
                if conds.get(&Term::Signal(sig, tr0)).cloned() != Some(!inv) {
                    trace!(
                        "    {} {}",
                        if inv { "low" } else { "high" },
                        sig.dump(&unit)
                    );
                    levels.insert(
                        sig,
                        match inv {
                            false => TriggerLevel::High,
                            true => TriggerLevel::Low,
                        },
                    );
                }
            }

            _ => {
                trace!("    Skipping (invalid term)");
                return None;
            }
        }
    }

    // Discard multi-edge triggers.
    if edges.len() > 1 {
        trace!("    Skipping (multiple edge triggers)");
        return None;
    }

    // Either formulate an edge trigger with the levels as conditions, or a
    // purely level-sensitive trigger.
    if let Some((sig, edge)) = edges.into_iter().next() {
        Some(Trigger::Edge(sig, edge, levels))
    } else {
        Some(Trigger::Level(levels))
    }
}

#[derive(Debug)]
enum Trigger {
    Edge(Value, TriggerEdge, BTreeMap<Value, TriggerLevel>),
    Level(BTreeMap<Value, TriggerLevel>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TriggerEdge {
    Rise,
    Fall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TriggerLevel {
    Low,
    High,
}

/// A helper struct to migrate data flow into an entity.
struct Migrator<'a, 'b> {
    src: &'a UnitBuilder<'b>,
    dst: &'a mut UnitBuilder<'b>,
    trg: &'a TemporalRegionGraph,
    tr0: TemporalRegion,
    tr1: TemporalRegion,
    /// Cache of already-migrated instructions.
    cache: HashMap<InstData, Value>,
    /// Set of migrated drives in `src`.
    migrated_drives: HashSet<Inst>,
}

impl<'a, 'b> Migrator<'a, 'b> {
    pub fn new(
        src: &'a UnitBuilder<'b>,
        dst: &'a mut UnitBuilder<'b>,
        trg: &'a TemporalRegionGraph,
        tr0: TemporalRegion,
        tr1: TemporalRegion,
    ) -> Self {
        Self {
            src,
            dst,
            trg,
            tr0,
            tr1,
            cache: Default::default(),
            migrated_drives: Default::default(),
        }
    }

    pub fn migrate_drive(&mut self, drive: Inst, _bb: Block, trigs: &Vec<Trigger>) -> bool {
        trace!("Migrating {}", drive.dump(&self.src));
        let drive_target = self.src[drive].args()[0];
        let drive_value = self.src[drive].args()[1];

        let mig_target = match self.migrate_value(drive_target, &Default::default()) {
            Some(v) => v,
            None => return false,
        };

        let mut reg_triggers = vec![];
        for trig in trigs {
            trace!("  Migrating {:?}", trig);
            match trig {
                Trigger::Edge(sig, edge, conds) => {
                    // Setup a map of known signal values, per temporal region.
                    // Start out with just the trigger.
                    let mut ties = BTreeMap::new();
                    let (l0, l1) = match edge {
                        TriggerEdge::Rise => (TriggerLevel::Low, TriggerLevel::High),
                        TriggerEdge::Fall => (TriggerLevel::High, TriggerLevel::Low),
                    };
                    ties.insert((*sig, self.tr0), l0);
                    ties.insert((*sig, self.tr1), l1);

                    // Migrate the conditions.
                    let mut gate = None;
                    for (&sig, &level) in conds {
                        ties.insert((sig, self.tr1), level);
                        let value = self.dst.ins().prb(sig);
                        let value = match level {
                            TriggerLevel::Low => self.dst.ins().not(value),
                            TriggerLevel::High => value,
                        };
                        gate = Some(match gate {
                            Some(gate) => self.dst.ins().and(gate, value),
                            None => value,
                        });
                    }

                    // Migrate the value computation.
                    let data = match self.migrate_value(drive_value, &ties) {
                        Some(v) => v,
                        None => return false,
                    };

                    // Keep track of this trigger.
                    let trigger = self.dst.ins().prb(*sig);
                    let mode = match edge {
                        TriggerEdge::Rise => RegMode::Rise,
                        TriggerEdge::Fall => RegMode::Fall,
                    };
                    reg_triggers.push(RegTrigger {
                        data,
                        mode,
                        trigger,
                        gate,
                    });
                }
                Trigger::Level(conds) => {
                    // Migrate the conditions.
                    let mut ties = BTreeMap::new();
                    let mut trigger = None;
                    for (&sig, &level) in conds {
                        ties.insert((sig, self.tr1), level);
                        let value = self.dst.ins().prb(sig);
                        let value = match level {
                            TriggerLevel::Low => self.dst.ins().not(value),
                            TriggerLevel::High => value,
                        };
                        trigger = Some(match trigger {
                            Some(trigger) => self.dst.ins().and(trigger, value),
                            None => value,
                        });
                    }
                    let trigger = match trigger {
                        Some(c) => c,
                        None => {
                            trace!(
                                "    Skipping {} (level-sensitive with no trigger)",
                                drive.dump(&self.src)
                            );
                            return false;
                        }
                    };

                    // Migrate the value computation.
                    let data = match self.migrate_value(drive_value, &ties) {
                        Some(v) => v,
                        None => return false,
                    };

                    // Keep track of this trigger.
                    reg_triggers.push(RegTrigger {
                        data,
                        mode: RegMode::High,
                        trigger,
                        gate: None,
                    });
                }
            }
        }

        // Create the register instruction.
        self.dst.ins().reg(mig_target, reg_triggers);

        // Drive the register value onto the output.
        self.migrated_drives.insert(drive);
        true
    }

    pub fn migrate_value(
        &mut self,
        value: Value,
        ties: &BTreeMap<(Value, TemporalRegion), TriggerLevel>,
    ) -> Option<Value> {
        // Migrate arguments.
        if let Some(arg) = self.src.get_value_arg(value) {
            return Some(self.dst.arg_value(arg));
        }

        // Migrate instructions.
        if let Some(inst) = self.src.get_value_inst(value) {
            let bb = self.src.inst_block(inst)?;
            let tr = self.trg[bb];

            // Handle signal probes.
            if self.src[inst].opcode() == Opcode::Prb {
                // See if this signal is tied to a fixed value.
                let sig = self.src[inst].args()[0];
                if let Some(&level) = ties.get(&(sig, tr)) {
                    let data = InstData::ConstInt {
                        opcode: Opcode::ConstInt,
                        imm: IntValue::from_usize(
                            1,
                            match level {
                                TriggerLevel::High => 1,
                                TriggerLevel::Low => 0,
                            },
                        ),
                    };
                    return Some(self.migrate_inst_data(data, value));
                }

                // Otherwise ensure that the probe occurs *after* the trigger.
                // This is a requirement for modeling the behaviour with `reg`.
                if tr != self.tr1 {
                    trace!("    Skipping {} (probe in wrong TR)", inst.dump(&self.src));
                    return None;
                }
            }

            // Handle regular signals.
            let mut data = self.src[inst].clone();
            #[allow(deprecated)]
            for arg in data.args_mut() {
                *arg = self.migrate_value(*arg, ties)?;
            }
            return Some(self.migrate_inst_data(data, value));
        }

        // Otherwise just refuse to migrate.
        trace!(
            "    Skipping {} (cannot be migrated)",
            value.dump(&self.src)
        );
        None
    }

    fn migrate_inst_data(&mut self, data: InstData, src_value: Value) -> Value {
        if let Some(&v) = self.cache.get(&data) {
            v
        } else {
            trace!("    Migrated {}", src_value.dump(&self.src));
            let ty = self.src.value_type(src_value);
            let inst = self.dst.ins().build(data.clone(), ty);
            let value = self.dst.inst_result(inst);
            self.cache.insert(data, value);
            if let Some(name) = self.src.get_name(src_value) {
                self.dst.set_name(value, name.to_string());
            }
            value
        }
    }
}
