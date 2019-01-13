// Copyright (c) 2017 Fabian Schuiki
#![allow(unused_variables)]

//! Modules in LLHD encapsulate a design hierarchy and its data dependency and
//! control flow graphs.

use crate::entity::Entity;
use crate::function::Function;
use crate::process::Process;
use std;
use std::collections::HashMap;
use crate::value::{Context, EntityRef, FunctionRef, ProcessRef, Value, ValueRef};

pub struct Module {
    funcs: HashMap<FunctionRef, Function>,
    procs: HashMap<ProcessRef, Process>,
    entities: HashMap<EntityRef, Entity>,
    values: Vec<ValueRef>,
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Module {
        Module {
            funcs: HashMap::new(),
            procs: HashMap::new(),
            entities: HashMap::new(),
            values: Vec::new(),
        }
    }

    /// Add a function to the module.
    pub fn add_function(&mut self, func: Function) -> FunctionRef {
        let r = func.as_ref();
        self.funcs.insert(r, func);
        self.values.push(r.into());
        r
    }

    /// Add a process to the module.
    pub fn add_process(&mut self, prok: Process) -> ProcessRef {
        let r = prok.as_ref();
        self.procs.insert(r, prok);
        self.values.push(r.into());
        r
    }

    /// Add a entity to the module.
    pub fn add_entity(&mut self, entity: Entity) -> EntityRef {
        let r = entity.as_ref();
        self.entities.insert(r, entity);
        self.values.push(r.into());
        r
    }

    /// Get a reference to a function in the module.
    pub fn function(&self, func: FunctionRef) -> &Function {
        self.funcs.get(&func).unwrap()
    }

    /// Get a mutable reference to a function in the module.
    pub fn function_mut(&mut self, func: FunctionRef) -> &mut Function {
        self.funcs.get_mut(&func).unwrap()
    }

    /// Get a reference to a process in the module.
    pub fn process(&self, prok: ProcessRef) -> &Process {
        self.procs.get(&prok).unwrap()
    }

    /// Get a mutable reference to a process in the module.
    pub fn process_mut(&mut self, prok: ProcessRef) -> &mut Process {
        self.procs.get_mut(&prok).unwrap()
    }

    /// Get a reference to an entity in the module.
    pub fn entity(&self, entity: EntityRef) -> &Entity {
        self.entities.get(&entity).unwrap()
    }

    /// Get a mutable reference to an entity in the module.
    pub fn entity_mut(&mut self, entity: EntityRef) -> &mut Entity {
        self.entities.get_mut(&entity).unwrap()
    }

    /// Obtain an iterator over the values in the module. This includes globals,
    /// functions, processes, and entities.
    pub fn values(&self) -> std::slice::Iter<ValueRef> {
        self.values.iter()
    }
}

pub struct ModuleContext<'tctx> {
    module: &'tctx Module,
}

impl<'tctx> ModuleContext<'tctx> {
    pub fn new(module: &Module) -> ModuleContext {
        ModuleContext { module: module }
    }

    pub fn function(&self, func: FunctionRef) -> &Function {
        self.module.function(func)
    }

    pub fn process(&self, prok: ProcessRef) -> &Process {
        self.module.process(prok)
    }

    pub fn entity(&self, entity: EntityRef) -> &Entity {
        self.module.entity(entity)
    }
}

impl<'tctx> Context for ModuleContext<'tctx> {
    fn try_value(&self, value: &ValueRef) -> Option<&Value> {
        match *value {
            ValueRef::Function(id) => Some(self.function(id)),
            ValueRef::Process(id) => Some(self.process(id)),
            ValueRef::Entity(id) => Some(self.entity(id)),
            _ => None,
        }
    }
}
