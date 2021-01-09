// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of linked LLHD units.
//!
//! This module implements the `Module`, a collection of LLHD `Function`,
//! `Process`, and `Entity` objects linked together. A module acts as the root
//! node of an LLHD intermediate representation, and is the unit of information
//! ingested by the reader and emitted by the writer.

use crate::{
    impl_table_key,
    ir::{ExtUnit, Signature, Unit, UnitBuilder, UnitData, UnitName},
    table::{PrimaryTable, TableKey},
    verifier::Verifier,
};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};

/// A module.
///
/// This is the root node of an LLHD intermediate representation. Contains
/// `Function`, `Process`, and `Entity` declarations and definitions.
#[derive(Serialize, Deserialize)]
pub struct Module {
    /// The units in this module.
    pub(crate) units: PrimaryTable<UnitId, UnitData>,
    /// The order of units in the module.
    unit_order: BTreeSet<UnitId>,
    /// The declarations in this module.
    pub(crate) decls: PrimaryTable<DeclId, DeclData>,
    /// The order of declarations in the module.
    decl_order: BTreeSet<DeclId>,
    /// The local link table. Maps an external unit declared within a unit to a
    /// unit in the module.
    link_table: Option<HashMap<(UnitId, ExtUnit), LinkedUnit>>,
    /// The location of units in the input file. If the module was read from a
    /// file, this table *may* contain additional hints on the byte offsets
    /// where the units were located.
    location_hints: HashMap<UnitId, usize>,
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Self {
        Self {
            units: PrimaryTable::new(),
            unit_order: BTreeSet::new(),
            decls: PrimaryTable::new(),
            decl_order: BTreeSet::new(),
            link_table: None,
            location_hints: Default::default(),
        }
    }

    /// Dump the module in human-readable form.
    pub fn dump(&self) -> ModuleDumper {
        ModuleDumper(self)
    }

    /// Add a unit to the module.
    pub fn add_unit(&mut self, data: UnitData) -> UnitId {
        let unit = self.units.add(data);
        self.unit_order.insert(unit);
        self.link_table = None;
        unit
    }

    /// Remove a unit from the module.
    pub fn remove_unit(&mut self, unit: UnitId) {
        self.units.remove(unit);
        self.unit_order.remove(&unit);
    }

    /// Declare an external unit.
    pub fn declare(&mut self, name: UnitName, sig: Signature) -> DeclId {
        self.add_decl(DeclData {
            name,
            sig,
            loc: None,
        })
    }

    /// Declare an external unit.
    pub fn add_decl(&mut self, data: DeclData) -> DeclId {
        let decl = self.decls.add(data);
        self.decl_order.insert(decl);
        self.link_table = None;
        decl
    }

    /// Remove a declaration from the module.
    pub fn remove_decl(&mut self, decl: DeclId) {
        self.decls.remove(decl);
        self.decl_order.remove(&decl);
    }

    /// Return an iterator over the units in this module.
    pub fn units<'a>(&'a self) -> impl Iterator<Item = Unit<'a>> + 'a {
        self.unit_order.iter().map(move |&id| self.unit(id))
    }

    /// Return a mutable iterator over the units in this module.
    pub fn units_mut<'a>(&'a mut self) -> impl Iterator<Item = UnitBuilder<'a>> + 'a {
        self.units
            .storage
            .iter_mut()
            .map(|(&id, data)| UnitBuilder::new(UnitId::new(id), data))
    }

    /// Return a parallel iterator over the units in this module.
    pub fn par_units<'a>(&'a self) -> impl ParallelIterator<Item = Unit<'a>> + 'a {
        self.unit_order.par_iter().map(move |&id| self.unit(id))
    }

    /// Return a parallel mutable iterator over the units in this module.
    pub fn par_units_mut<'a>(&'a mut self) -> impl ParallelIterator<Item = UnitBuilder<'a>> + 'a {
        self.units
            .storage
            .par_iter_mut()
            .map(|(&id, data)| UnitBuilder::new(UnitId::new(id), data))
    }

    /// Return an iterator over the functions in this module.
    pub fn functions<'a>(&'a self) -> impl Iterator<Item = Unit<'a>> + 'a {
        self.units().filter(|unit| unit.is_function())
    }

    /// Return an iterator over the processes in this module.
    pub fn processes<'a>(&'a self) -> impl Iterator<Item = Unit<'a>> + 'a {
        self.units().filter(|unit| unit.is_process())
    }

    /// Return an iterator over the entities in this module.
    pub fn entities<'a>(&'a self) -> impl Iterator<Item = Unit<'a>> + 'a {
        self.units().filter(|unit| unit.is_entity())
    }

    /// Return an iterator over the external unit declarations in this module.
    pub fn decls<'a>(&'a self) -> impl Iterator<Item = DeclId> + 'a {
        self.decl_order.iter().cloned()
    }

    /// Return an unit in the module.
    pub fn unit(&self, unit: UnitId) -> Unit {
        Unit::new(unit, &self[unit])
    }

    /// Return a mutable unit in the module.
    pub fn unit_mut(&mut self, unit: UnitId) -> UnitBuilder {
        self.link_table = None;
        UnitBuilder::new(unit, &mut self[unit])
    }

    /// Return an iterator over the symbols in the module.
    pub fn symbols<'a>(&'a self) -> impl Iterator<Item = (&UnitName, LinkedUnit, &Signature)> + 'a {
        self.units()
            .map(|unit| (unit.name(), LinkedUnit::Def(unit.id()), unit.sig()))
            .chain(
                self.decls()
                    .map(move |decl| (&self[decl].name, LinkedUnit::Decl(decl), &self[decl].sig)),
            )
    }

    /// Return an iterator over the local symbols in the module.
    pub fn local_symbols<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&UnitName, LinkedUnit, &Signature)> + 'a {
        self.symbols().filter(|&(name, ..)| name.is_local())
    }

    /// Return an iterator over the global symbols in the module.
    pub fn global_symbols<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&UnitName, LinkedUnit, &Signature)> + 'a {
        self.symbols().filter(|&(name, ..)| name.is_global())
    }

    /// Check whether the module is internally linked.
    ///
    /// Adding or modifying a unit invalidates the linkage within the module.
    pub fn is_linked(&self) -> bool {
        self.link_table.is_some()
    }

    /// Locally link the module.
    pub fn link(&mut self) {
        let mut failed = false;

        // Collect a table of symbols that we can resolve against.
        let mut symbols = HashMap::new();
        for (name, unit, sig) in self.symbols() {
            if let Some((existing, _)) = symbols.insert(name, (unit, sig)) {
                if !existing.is_decl() {
                    eprintln!("unit {} declared multiple times", name);
                    failed = true;
                }
            }
        }
        if failed {
            panic!("linking failed; multiple uses of the same name");
        }

        // Resolve the external units in each unit.
        let mut linked = HashMap::new();
        for unit in self.units() {
            for (ext_unit, data) in unit.extern_units() {
                let (to, to_sig) = match symbols.get(&data.name).cloned() {
                    Some(to) => to,
                    None => {
                        eprintln!(
                            "unit {} not found; referenced in {}",
                            data.name,
                            unit.name()
                        );
                        failed = true;
                        continue;
                    }
                };
                if to_sig != &data.sig {
                    eprintln!(
                        "signature mismatch: {} has {}, but reference in {} expects {}",
                        data.name,
                        to_sig,
                        unit.name(),
                        data.sig
                    );
                    failed = true;
                    continue;
                }
                linked.insert((unit.id(), ext_unit), to);
            }
        }
        if failed {
            panic!("linking failed; unresolved references");
        }
        self.link_table = Some(linked);
    }

    /// Panic if the module is not well-formed.
    pub fn verify(&self) {
        let mut verifier = Verifier::new();
        verifier.verify_module(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified module:");
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }

    /// Lookup what an external unit links to.
    ///
    /// The module must be linked for this to work.
    pub fn lookup_ext_unit(&self, ext_unit: ExtUnit, within: UnitId) -> Option<LinkedUnit> {
        self.link_table
            .as_ref()
            .and_then(|lt| lt.get(&(within, ext_unit)))
            .cloned()
    }

    /// Add a location hint to a unit.
    ///
    /// Annotates the byte offset of a unit in the input file.
    pub fn set_location_hint(&mut self, mod_unit: UnitId, loc: usize) {
        self.location_hints.insert(mod_unit, loc);
    }

    /// Get the location hint associated with a unit.
    ///
    /// Returns the byte offset of the unit in the input file, or None if there
    /// is no hint for the value.
    pub fn location_hint(&self, mod_unit: UnitId) -> Option<usize> {
        self.location_hints.get(&mod_unit).cloned()
    }
}

impl std::ops::Index<UnitId> for Module {
    type Output = UnitData;
    fn index(&self, idx: UnitId) -> &UnitData {
        &self.units[idx]
    }
}

impl std::ops::IndexMut<UnitId> for Module {
    fn index_mut(&mut self, idx: UnitId) -> &mut UnitData {
        self.link_table = None;
        &mut self.units[idx]
    }
}

impl std::ops::Index<DeclId> for Module {
    type Output = DeclData;
    fn index(&self, idx: DeclId) -> &DeclData {
        &self.decls[idx]
    }
}

impl std::ops::IndexMut<DeclId> for Module {
    fn index_mut(&mut self, idx: DeclId) -> &mut DeclData {
        self.link_table = None;
        &mut self.decls[idx]
    }
}

/// Temporary object to dump a `Module` in human-readable form for debugging.
pub struct ModuleDumper<'a>(&'a Module);

impl std::fmt::Display for ModuleDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut newline = false;
        for unit in self.0.units() {
            if newline {
                writeln!(f, "")?;
                writeln!(f, "")?;
            }
            newline = true;
            write!(f, "{}: ", unit.id())?;
            write!(f, "{}", unit)?;
        }
        if newline && !self.0.decls().count() > 0 {
            writeln!(f, "")?;
        }
        for decl in self.0.decls() {
            if newline {
                writeln!(f, "")?;
            }
            newline = true;
            let data = &self.0[decl];
            write!(f, "declare {} {}", data.name, data.sig)?;
        }
        Ok(())
    }
}

impl_table_key! {
    /// A unit definition in a module.
    struct UnitId(u32) as "u";
    /// A unit declaration in a module.
    struct DeclId(u32) as "decl";
}

/// A unit declaration.
#[derive(Serialize, Deserialize)]
pub struct DeclData {
    /// The unit signature.
    pub sig: Signature,
    /// The unit name.
    pub name: UnitName,
    /// The location in the source file.
    pub loc: Option<usize>,
}

/// A linked unit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LinkedUnit {
    /// A unit definition.
    Def(UnitId),
    /// A unit declaration.
    Decl(DeclId),
}

impl LinkedUnit {
    /// Check whether the linked unit is a definition.
    pub fn is_def(&self) -> bool {
        match self {
            LinkedUnit::Def(..) => true,
            _ => false,
        }
    }

    /// Check whether the linked unit is a declaration.
    pub fn is_decl(&self) -> bool {
        match self {
            LinkedUnit::Decl(..) => true,
            _ => false,
        }
    }
}
