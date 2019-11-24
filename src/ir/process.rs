// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD processes.

use crate::{
    ir::{
        Block, ControlFlowGraph, DataFlowGraph, FunctionInsertPos, FunctionLayout, Inst, InstData,
        InstLayout, Signature, Unit, UnitBuilder, UnitKind, UnitName, Value,
    },
    ty::Type,
    verifier::Verifier,
};

/// A process.
#[derive(Serialize, Deserialize)]
pub struct Process {
    pub name: UnitName,
    pub sig: Signature,
    pub dfg: DataFlowGraph,
    pub cfg: ControlFlowGraph,
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
            cfg: ControlFlowGraph::new(),
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

    fn get_process(&self) -> Option<&Process> {
        Some(self)
    }

    fn get_process_mut(&mut self) -> Option<&mut Process> {
        Some(self)
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.dfg
    }

    fn try_cfg(&self) -> Option<&ControlFlowGraph> {
        Some(&self.cfg)
    }

    fn try_cfg_mut(&mut self) -> Option<&mut ControlFlowGraph> {
        Some(&mut self.cfg)
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

    fn inst_layout(&self) -> &InstLayout {
        panic!("inst_layout() called on process");
    }

    fn inst_layout_mut(&mut self) -> &mut InstLayout {
        panic!("inst_layout_mut() called on process");
    }

    fn dump_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "proc {} {} {{\n", self.name, self.sig.dump(&self.dfg))?;
        for bb in self.layout.blocks() {
            write!(f, "{}:\n", bb.dump(&self.cfg))?;
            for inst in self.layout.insts(bb) {
                write!(f, "    {}\n", inst.dump(&self.dfg, Some(&self.cfg)))?;
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
        self.pos.add_inst(inst, &mut self.prok.layout);
        inst
    }

    fn remove_inst(&mut self, inst: Inst) {
        self.prok.dfg.remove_inst(inst);
        self.pos.remove_inst(inst, &self.prok.layout);
        self.prok.layout.remove_inst(inst);
    }

    fn block(&mut self) -> Block {
        let bb = self.prok.cfg.add_block();
        self.prok.layout.append_block(bb);
        bb
    }

    fn remove_block(&mut self, bb: Block) {
        let insts: Vec<_> = self.prok.layout.insts(bb).collect();
        self.prok.layout.remove_block(bb);
        self.prok.cfg_mut().remove_block(bb);
        for inst in insts {
            if self.prok.dfg().has_result(inst) {
                let value = self.prok.dfg().inst_result(inst);
                self.prok.dfg_mut().replace_use(value, Value::invalid());
            }
            self.prok.dfg_mut().remove_inst(inst);
        }
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
