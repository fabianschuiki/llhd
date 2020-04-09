// Copyright (c) 2017-2020 Fabian Schuiki

//! Representation of LLHD processes.

use crate::{
    ir::{
        Block, BlockData, ControlFlowGraph, DataFlowGraph, ExtUnit, ExtUnitData, FunctionInsertPos,
        FunctionLayout, Inst, InstData, InstLayout, Signature, Unit, UnitBuilder, UnitKind,
        UnitName, Value, ValueData,
    },
    ty::Type,
    verifier::Verifier,
};
use std::ops::{Index, IndexMut};

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

impl Index<Value> for Process {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.dfg.index(idx)
    }
}

impl Index<Inst> for Process {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.dfg.index(idx)
    }
}

impl Index<ExtUnit> for Process {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.dfg.index(idx)
    }
}

impl Index<Block> for Process {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.cfg.index(idx)
    }
}

impl IndexMut<Value> for Process {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Inst> for Process {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for Process {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Block> for Process {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.cfg.index_mut(idx)
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

impl Index<Value> for ProcessBuilder<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.prok.index(idx)
    }
}

impl Index<Inst> for ProcessBuilder<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.prok.index(idx)
    }
}

impl Index<ExtUnit> for ProcessBuilder<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.prok.index(idx)
    }
}

impl Index<Block> for ProcessBuilder<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.prok.index(idx)
    }
}

impl IndexMut<Value> for ProcessBuilder<'_> {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.prok.index_mut(idx)
    }
}

impl IndexMut<Inst> for ProcessBuilder<'_> {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.prok.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for ProcessBuilder<'_> {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.prok.index_mut(idx)
    }
}

impl IndexMut<Block> for ProcessBuilder<'_> {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.prok.index_mut(idx)
    }
}

impl<'u> std::ops::Deref for ProcessBuilder<'u> {
    type Target = Process;
    fn deref(&self) -> &Process {
        self.prok
    }
}

impl<'u> std::ops::DerefMut for ProcessBuilder<'u> {
    fn deref_mut(&mut self) -> &mut Process {
        self.prok
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
        self.prok.dfg_mut().remove_block_use(bb);
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
