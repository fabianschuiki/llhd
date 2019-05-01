// Copyright (c) 2017 Fabian Schuiki

use crate::argument::*;
use crate::block::*;
use crate::inst::*;
use crate::seq_body::*;
use crate::ty::*;
use crate::unit::*;
use crate::value::*;

/// A process. Sequentially executes instructions to react to changes in input
/// signals. Implements *control flow* and *timed execution*.
pub struct Process {
    id: ProcessRef,
    global: bool,
    name: String,
    ty: Type,
    ins: Vec<Argument>,
    outs: Vec<Argument>,
    body: SeqBody,
}

impl Process {
    /// Create a new process with the given name and type signature. Anonymous
    /// arguments are created for each input and output in the type signature.
    /// Use the `inputs_mut` and `outputs_mut` functions get a hold of these
    /// arguments and assign names and additional data to them.
    pub fn new(name: impl Into<String>, ty: Type) -> Process {
        let (ins, outs) = {
            let (in_tys, out_tys) = ty.unwrap_entity();
            let to_arg = |t: &Type| Argument::new(t.clone());
            (
                in_tys.iter().map(&to_arg).collect(),
                out_tys.iter().map(&to_arg).collect(),
            )
        };
        Process {
            id: ProcessRef::new(ValueId::alloc()),
            global: true,
            name: name.into(),
            ty: ty,
            ins: ins,
            outs: outs,
            body: SeqBody::new(),
        }
    }

    /// Obtain a reference to this process.
    pub fn as_ref(&self) -> ProcessRef {
        self.id
    }

    /// Get the name of the process.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a graph reference to one of the inputs of the entity.
    pub fn input(&self, idx: usize) -> ArgumentRef {
        self.ins[idx].as_ref()
    }

    /// Get a reference to the input arguments of the process.
    pub fn inputs(&self) -> &[Argument] {
        &self.ins
    }

    /// Get a mutable reference to the input arguments of the process.
    pub fn inputs_mut(&mut self) -> &mut [Argument] {
        &mut self.ins
    }

    /// Get a graph reference to one of the outputs of the entity.
    pub fn output(&self, idx: usize) -> ArgumentRef {
        self.outs[idx].as_ref()
    }

    /// Get a reference to the output arguments of the process.
    pub fn outputs(&self) -> &[Argument] {
        &self.outs
    }

    /// Get a mutable reference to the output arguments of the process.
    pub fn outputs_mut(&mut self) -> &mut [Argument] {
        &mut self.outs
    }

    /// Get a reference to the sequential body of the process.
    pub fn body(&self) -> &SeqBody {
        &self.body
    }

    /// Get a mutable reference to the sequential body of the process.
    pub fn body_mut(&mut self) -> &mut SeqBody {
        &mut self.body
    }
}

impl Value for Process {
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

pub struct ProcessContext<'tctx> {
    process: &'tctx Process,
}

impl<'tctx> ProcessContext<'tctx> {
    pub fn new(process: &'tctx Process) -> ProcessContext<'tctx> {
        ProcessContext { process: process }
    }
}

impl<'tctx> Context for ProcessContext<'tctx> {
    fn try_value(&self, value: &ValueRef) -> Option<&Value> {
        match *value {
            ValueRef::Inst(id) => Some(self.inst(id)),
            ValueRef::Block(id) => Some(self.block(id)),
            ValueRef::Argument(id) => Some(self.argument(id)),
            _ => None,
        }
    }
}

impl<'tctx> UnitContext for ProcessContext<'tctx> {
    fn inst(&self, inst: InstRef) -> &Inst {
        self.process.body.inst(inst)
    }

    fn argument(&self, argument: ArgumentRef) -> &Argument {
        self.process
            .ins
            .iter()
            .chain(self.process.outs.iter())
            .find(|x| argument == x.as_ref())
            .unwrap()
    }
}

impl<'tctx> SequentialContext for ProcessContext<'tctx> {
    fn block(&self, block: BlockRef) -> &Block {
        self.process.body.block(block)
    }
}
