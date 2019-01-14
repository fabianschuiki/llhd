// Copyright (c) 2017 Fabian Schuiki

use crate::argument::*;
use crate::inst::*;
use crate::module::ModuleContext;
use std;
use std::collections::HashMap;
use crate::ty::*;
use crate::unit::*;
use crate::util::IndirectMapIter;
use crate::value::*;

/// An entity. Describes through its instructions the data dependencies in order
/// to react to changes in input signals. Implements *data flow* and *timed
/// execution*.
pub struct Entity {
    id: EntityRef,
    global: bool,
    name: String,
    ty: Type,
    ins: Vec<Argument>,
    outs: Vec<Argument>,
    insts: HashMap<InstRef, Inst>,
    inst_seq: Vec<InstRef>,
}

impl Entity {
    /// Create a new entity with the given name and type signature. Anonymous
    /// arguments are created for each input and output in the type signature.
    /// Use the `inputs_mut` and `outputs_mut` functions get a hold of these
    /// arguments and assign names and additional data to them.
    pub fn new(name: impl Into<String>, ty: Type) -> Entity {
        let (ins, outs) = {
            let (in_tys, out_tys) = ty.as_entity();
            let to_arg = |t: &Type| Argument::new(t.clone());
            (
                in_tys.iter().map(&to_arg).collect(),
                out_tys.iter().map(&to_arg).collect(),
            )
        };
        Entity {
            id: EntityRef::new(ValueId::alloc()),
            global: true,
            name: name.into(),
            ty: ty,
            ins: ins,
            outs: outs,
            insts: HashMap::new(),
            inst_seq: Vec::new(),
        }
    }

    /// Obtain a reference to this entity.
    pub fn as_ref(&self) -> EntityRef {
        self.id
    }

    /// Get the name of the entity.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a graph reference to one of the inputs of the entity.
    pub fn input(&self, idx: usize) -> ArgumentRef {
        self.ins[idx].as_ref()
    }

    /// Get a reference to the input arguments of the entity.
    pub fn inputs(&self) -> &[Argument] {
        &self.ins
    }

    /// Get a mutable reference to the input arguments of the entity.
    pub fn inputs_mut(&mut self) -> &mut [Argument] {
        &mut self.ins
    }

    /// Get a graph reference to one of the outputs of the entity.
    pub fn output(&self, idx: usize) -> ArgumentRef {
        self.outs[idx].as_ref()
    }

    /// Get a reference to the output arguments of the entity.
    pub fn outputs(&self) -> &[Argument] {
        &self.outs
    }

    /// Get a mutable reference to the output arguments of the entity.
    pub fn outputs_mut(&mut self) -> &mut [Argument] {
        &mut self.outs
    }

    /// Add an instruction to the body.
    pub fn add_inst(&mut self, inst: Inst, pos: InstPosition) -> InstRef {
        let ir = inst.as_ref();
        self.insts.insert(ir, inst);
        self.insert_inst(ir, pos);
        ir
    }

    /// Move an instruction around within the body.
    pub fn move_inst(&mut self, inst: InstRef, pos: InstPosition) {
        self.detach_inst(inst);
        self.insert_inst(inst, pos);
    }

    /// Remove an instruction from the body.
    pub fn remove_inst(&mut self, inst: InstRef) {
        self.detach_inst(inst);
        self.insts.remove(&inst);
    }

    /// Insert an instruction into the basic block dictated by the requested
    /// position. The instruction must be detached, i.e. either it is a fresh
    /// instruction or `detach_inst` has been called before.
    fn insert_inst(&mut self, inst: InstRef, pos: InstPosition) {
        let index = match pos {
            InstPosition::Begin => 0,
            InstPosition::End => self.inst_seq.len(),
            InstPosition::Before(i) => self.inst_pos(i),
            InstPosition::After(i) => self.inst_pos(i) + 1,
            InstPosition::BlockBegin(_) | InstPosition::BlockEnd(_) => {
                panic!("entity has no blocks")
            }
        };
        self.inst_seq.insert(index, inst);
    }

    /// Detach an instruction from its current basic block.
    fn detach_inst(&mut self, inst: InstRef) {
        let pos = self.inst_pos(inst);
        self.inst_seq.remove(pos);
    }

    /// Determine the index at which a certain position is located. Panics if
    /// the instruction is not part of the entity.
    fn inst_pos(&self, inst: InstRef) -> usize {
        self.inst_seq
            .iter()
            .position(|&i| i == inst)
            .expect("entity does not contain inst")
    }

    /// Get a reference to an instruction in the body. Panics if the instruction
    /// does not exist.
    pub fn inst(&self, inst: InstRef) -> &Inst {
        self.insts.get(&inst).unwrap()
    }

    /// Get a mutable reference to an instruction in the body. Panics if the
    /// instruction does not exist.
    pub fn inst_mut(&mut self, inst: InstRef) -> &mut Inst {
        self.insts.get_mut(&inst).unwrap()
    }

    /// Obtain an iterator over the instructions in this entity.
    pub fn insts(&self) -> IndirectMapIter<std::slice::Iter<InstRef>, Inst> {
        IndirectMapIter::new(self.inst_seq.iter(), &self.insts)
    }

    /// Obtain an iterator over references to the instructions in this entity.
    pub fn inst_refs(&self) -> std::slice::Iter<InstRef> {
        self.inst_seq.iter()
    }
}

impl Value for Entity {
    fn id(&self) -> ValueId {
        self.id.into()
    }

    fn ty(&self) -> Type {
        self.ty.clone()
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn is_global(&self) -> bool {
        self.global
    }
}

pub struct EntityContext<'tctx> {
    module: &'tctx ModuleContext<'tctx>,
    entity: &'tctx Entity,
}

impl<'tctx> EntityContext<'tctx> {
    pub fn new(module: &'tctx ModuleContext, entity: &'tctx Entity) -> EntityContext<'tctx> {
        EntityContext {
            module: module,
            entity: entity,
        }
    }
}

impl<'tctx> Context for EntityContext<'tctx> {
    fn parent(&self) -> Option<&Context> {
        Some(self.module.as_context())
    }

    fn try_value(&self, value: &ValueRef) -> Option<&Value> {
        match *value {
            ValueRef::Inst(id) => Some(self.inst(id)),
            ValueRef::Block(_) => panic!("entity has no blocks"),
            ValueRef::Argument(id) => Some(self.argument(id)),
            _ => None,
        }
    }
}

impl<'tctx> UnitContext for EntityContext<'tctx> {
    fn inst(&self, inst: InstRef) -> &Inst {
        self.entity.inst(inst)
    }

    fn argument(&self, argument: ArgumentRef) -> &Argument {
        self.entity
            .ins
            .iter()
            .chain(self.entity.outs.iter())
            .find(|x| argument == x.as_ref())
            .unwrap()
    }
}
