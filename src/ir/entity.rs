// Copyright (c) 2017-2020 Fabian Schuiki

//! Representation of LLHD processes.

use crate::{
    ir::{
        Block, BlockData, ControlFlowGraph, DataFlowGraph, ExtUnit, ExtUnitData, FunctionInsertPos,
        FunctionLayout, Inst, InstData, Signature, Unit, UnitBuilder, UnitKind, UnitName, Value,
        ValueData,
    },
    ty::Type,
    verifier::Verifier,
};
use std::ops::{Index, IndexMut};

/// An entity.
#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: UnitName,
    pub sig: Signature,
    pub dfg: DataFlowGraph,
    pub cfg: ControlFlowGraph,
    pub layout: FunctionLayout,
}

impl Entity {
    /// Create a new entity.
    pub fn new(name: UnitName, sig: Signature) -> Self {
        assert!(!sig.has_return_type());
        let mut ent = Self {
            name,
            sig,
            dfg: DataFlowGraph::new(),
            cfg: ControlFlowGraph::new(),
            layout: FunctionLayout::new(),
        };
        let bb = ent.cfg.add_block();
        ent.layout.append_block(bb);
        ent.dfg.make_args_for_signature(&ent.sig);
        ent
    }
}

impl Index<Value> for Entity {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.dfg.index(idx)
    }
}

impl Index<Inst> for Entity {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.dfg.index(idx)
    }
}

impl Index<ExtUnit> for Entity {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.dfg.index(idx)
    }
}

impl Index<Block> for Entity {
    type Output = BlockData;
    fn index(&self, _idx: Block) -> &BlockData {
        panic!("indexing into entity with a block")
        // self.cfg.index(idx)
    }
}

impl IndexMut<Value> for Entity {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Inst> for Entity {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for Entity {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Block> for Entity {
    fn index_mut(&mut self, _idx: Block) -> &mut BlockData {
        panic!("indexing into entity with a block")
        // self.cfg.index_mut(idx)
    }
}

impl Unit for Entity {
    fn kind(&self) -> UnitKind {
        UnitKind::Entity
    }

    fn get_entity(&self) -> Option<&Entity> {
        Some(self)
    }

    fn get_entity_mut(&mut self) -> Option<&mut Entity> {
        Some(self)
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.dfg
    }

    fn try_cfg(&self) -> Option<&ControlFlowGraph> {
        None
    }

    fn try_cfg_mut(&mut self) -> Option<&mut ControlFlowGraph> {
        None
    }

    fn sig(&self) -> &Signature {
        &self.sig
    }

    fn sig_mut(&mut self) -> &mut Signature {
        &mut self.sig
    }

    fn name(&self) -> &UnitName {
        &self.name
    }

    fn name_mut(&mut self) -> &mut UnitName {
        &mut self.name
    }

    fn func_layout(&self) -> &FunctionLayout {
        &self.layout
    }

    fn func_layout_mut(&mut self) -> &mut FunctionLayout {
        &mut self.layout
    }

    fn dump_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "entity {} {} {{\n", self.name, self.sig.dump(&self.dfg))?;
        for inst in self.layout.all_insts() {
            write!(f, "    {}\n", inst.dump(&self.dfg, None))?;
        }
        write!(f, "}}")?;
        Ok(())
    }

    fn verify(&self) {
        let mut verifier = Verifier::new();
        verifier.verify_entity(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified entity:");
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }
}

/// Temporary object used to build a single `Entity`.
pub struct EntityBuilder<'u> {
    /// The entity currently being built.
    pub entity: &'u mut Entity,
    /// The position where we are currently inserting instructions.
    pos: FunctionInsertPos,
}

impl<'u> EntityBuilder<'u> {
    /// Create a new entity builder.
    pub fn new(entity: &mut Entity) -> EntityBuilder {
        let bb = entity.layout.entry();
        EntityBuilder {
            entity,
            pos: FunctionInsertPos::Append(bb),
        }
    }
}

impl Index<Value> for EntityBuilder<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.entity.index(idx)
    }
}

impl Index<Inst> for EntityBuilder<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.entity.index(idx)
    }
}

impl Index<ExtUnit> for EntityBuilder<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.entity.index(idx)
    }
}

impl Index<Block> for EntityBuilder<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.entity.index(idx)
    }
}

impl IndexMut<Value> for EntityBuilder<'_> {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.entity.index_mut(idx)
    }
}

impl IndexMut<Inst> for EntityBuilder<'_> {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.entity.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for EntityBuilder<'_> {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.entity.index_mut(idx)
    }
}

impl IndexMut<Block> for EntityBuilder<'_> {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.entity.index_mut(idx)
    }
}

impl<'u> std::ops::Deref for EntityBuilder<'u> {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        self.entity
    }
}

impl<'u> std::ops::DerefMut for EntityBuilder<'u> {
    fn deref_mut(&mut self) -> &mut Entity {
        self.entity
    }
}

impl UnitBuilder for EntityBuilder<'_> {
    type Unit = Entity;

    fn unit(&self) -> &Entity {
        self.entity
    }

    fn unit_mut(&mut self) -> &mut Entity {
        self.entity
    }

    fn build_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.entity.dfg.add_inst(data, ty);
        self.pos.add_inst(inst, &mut self.entity.layout);
        inst
    }

    fn remove_inst(&mut self, inst: Inst) {
        self.entity.dfg.remove_inst(inst);
        self.pos.remove_inst(inst, &self.entity.layout);
        self.entity.layout.remove_inst(inst);
    }

    fn block(&mut self) -> Block {
        panic!("block() called on entity");
    }

    fn remove_block(&mut self, _: Block) {
        panic!("remove_block() called on entity");
    }

    fn insert_at_end(&mut self) {
        self.pos = FunctionInsertPos::Append(self.unit().func_layout().entry());
    }

    fn insert_at_beginning(&mut self) {
        self.pos = FunctionInsertPos::Prepend(self.unit().func_layout().entry());
    }

    fn append_to(&mut self, _: Block) {
        panic!("append_to() called on entity");
    }

    fn prepend_to(&mut self, _: Block) {
        panic!("prepend_to() called on entity");
    }

    fn insert_after(&mut self, inst: Inst) {
        self.pos = FunctionInsertPos::After(inst);
    }

    fn insert_before(&mut self, inst: Inst) {
        self.pos = FunctionInsertPos::Before(inst);
    }
}
