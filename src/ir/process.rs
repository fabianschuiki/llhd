// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD processes.

use crate::{
    ir::{
        Block, DataFlowGraph, FunctionInsertPos, FunctionLayout, Inst, InstData, Signature, Unit,
        UnitBuilder, UnitKind, UnitName,
    },
    table::PrimaryTable,
    ty::Type,
    verifier::Verifier,
};

/// A process.
pub struct Process {
    pub name: UnitName,
    pub sig: Signature,
    pub dfg: DataFlowGraph,
    pub bbs: PrimaryTable<Block, ()>,
    pub layout: FunctionLayout,
}

impl Process {
    /// Create a new process.
    pub fn new(name: UnitName, sig: Signature) -> Self {
        assert!(!sig.has_return_type());
        let mut prok = Self {
            name,
            sig,
            dfg: DataFlowGraph::new(),
            bbs: PrimaryTable::new(),
            layout: FunctionLayout::new(),
        };
        prok.dfg.make_args_for_signature(&prok.sig);
        prok
    }
}

impl Unit for Process {
    fn kind(&self) -> UnitKind {
        UnitKind::Process
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.dfg
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

    fn dump_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "proc {} {} {{\n", self.name, self.sig.dump(&self.dfg))?;
        for bb in self.layout.blocks() {
            write!(f, "%{}:\n", bb)?;
            for inst in self.layout.insts(bb) {
                write!(f, "    {}\n", inst.dump(&self.dfg))?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }

    fn verify(&self) {
        let mut verifier = Verifier::new();
        verifier.verify_process(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified process:");
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }
}

/// Temporary object used to build a single `Process`.
pub struct ProcessBuilder<'u> {
    /// The function currently being built.
    pub prok: &'u mut Process,
    /// The position where we are currently inserting instructions.
    pos: FunctionInsertPos,
}

impl<'u> ProcessBuilder<'u> {
    /// Create a new function builder.
    pub fn new(prok: &'u mut Process) -> Self {
        Self {
            prok,
            pos: FunctionInsertPos::None,
        }
    }
}

impl UnitBuilder for ProcessBuilder<'_> {
    type Unit = Process;

    fn unit(&self) -> &Process {
        self.prok
    }

    fn unit_mut(&mut self) -> &mut Process {
        self.prok
    }

    fn build_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.prok.dfg.add_inst(data, ty);
        match self.pos {
            FunctionInsertPos::None => panic!("no block selected to insert instruction"),
            FunctionInsertPos::Append(bb) => self.prok.layout.append_inst(inst, bb),
            FunctionInsertPos::Prepend(bb) => self.prok.layout.prepend_inst(inst, bb),
            FunctionInsertPos::After(other) => self.prok.layout.insert_inst_after(inst, other),
            FunctionInsertPos::Before(other) => self.prok.layout.insert_inst_before(inst, other),
        }
        inst
    }

    fn remove_inst(&mut self, inst: Inst) {
        self.prok.dfg.remove_inst(inst);
        self.prok.layout.remove_inst(inst);
    }

    fn block(&mut self) -> Block {
        let bb = self.prok.bbs.add(());
        self.prok.layout.append_block(bb);
        bb
    }

    fn insert_at_end(&mut self) {
        panic!("insert_at_end() called on process")
    }

    fn insert_at_beginning(&mut self) {
        panic!("insert_at_beginning() called on process")
    }

    fn append_to(&mut self, bb: Block) {
        self.pos = FunctionInsertPos::Append(bb);
    }

    fn prepend_to(&mut self, bb: Block) {
        self.pos = FunctionInsertPos::Prepend(bb);
    }

    fn insert_after(&mut self, inst: Inst) {
        self.pos = FunctionInsertPos::After(inst);
    }

    fn insert_before(&mut self, inst: Inst) {
        self.pos = FunctionInsertPos::Before(inst);
    }
}
