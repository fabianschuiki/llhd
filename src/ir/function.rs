// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of LLHD functions.

use crate::{
    ir::{
        Block, ControlFlowGraph, DataFlowGraph, FunctionInsertPos, FunctionLayout, Inst, InstData,
        InstLayout, Signature, Unit, UnitBuilder, UnitKind, UnitName, Value,
    },
    ty::Type,
    verifier::Verifier,
};

/// A function.
pub struct Function {
    pub name: UnitName,
    pub sig: Signature,
    pub dfg: DataFlowGraph,
    pub cfg: ControlFlowGraph,
    pub layout: FunctionLayout,
}

impl Function {
    /// Create a new function.
    pub fn new(name: UnitName, sig: Signature) -> Self {
        assert!(!sig.has_outputs());
        assert!(sig.has_return_type());
        let mut func = Self {
            name,
            sig,
            dfg: DataFlowGraph::new(),
            cfg: ControlFlowGraph::new(),
            layout: FunctionLayout::new(),
        };
        func.dfg.make_args_for_signature(&func.sig);
        func
    }
}

impl Unit for Function {
    fn kind(&self) -> UnitKind {
        UnitKind::Function
    }

    fn get_function(&self) -> Option<&Function> {
        Some(self)
    }

    fn get_function_mut(&mut self) -> Option<&mut Function> {
        Some(self)
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.dfg
    }

    fn cfg(&self) -> &ControlFlowGraph {
        &self.cfg
    }

    fn cfg_mut(&mut self) -> &mut ControlFlowGraph {
        &mut self.cfg
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
        panic!("inst_layout() called on function");
    }

    fn inst_layout_mut(&mut self) -> &mut InstLayout {
        panic!("inst_layout_mut() called on function");
    }

    fn dump_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "func {} {} {{\n", self.name, self.sig.dump(&self.dfg))?;
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
        verifier.verify_function(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified function:");
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }
}

/// Temporary object used to build a single `Function`.
pub struct FunctionBuilder<'u> {
    /// The function currently being built.
    pub func: &'u mut Function,
    /// The position where we are currently inserting instructions.
    pos: FunctionInsertPos,
}

impl<'u> FunctionBuilder<'u> {
    /// Create a new function builder.
    pub fn new(func: &'u mut Function) -> Self {
        Self {
            func,
            pos: FunctionInsertPos::None,
        }
    }
}

impl UnitBuilder for FunctionBuilder<'_> {
    type Unit = Function;

    fn unit(&self) -> &Function {
        self.func
    }

    fn unit_mut(&mut self) -> &mut Function {
        self.func
    }

    fn build_inst(&mut self, data: InstData, ty: Type) -> Inst {
        let inst = self.func.dfg.add_inst(data, ty);
        match self.pos {
            FunctionInsertPos::None => panic!("no block selected to insert instruction"),
            FunctionInsertPos::Append(bb) => self.func.layout.append_inst(inst, bb),
            FunctionInsertPos::Prepend(bb) => self.func.layout.prepend_inst(inst, bb),
            FunctionInsertPos::After(other) => self.func.layout.insert_inst_after(inst, other),
            FunctionInsertPos::Before(other) => self.func.layout.insert_inst_before(inst, other),
        }
        inst
    }

    fn remove_inst(&mut self, inst: Inst) {
        self.func.dfg.remove_inst(inst);
        self.func.layout.remove_inst(inst);
    }

    fn block(&mut self) -> Block {
        let bb = self.func.cfg.add_block();
        self.func.layout.append_block(bb);
        bb
    }

    fn remove_block(&mut self, bb: Block) {
        let insts: Vec<_> = self.func.layout.insts(bb).collect();
        self.func.layout.remove_block(bb);
        self.func.cfg_mut().remove_block(bb);
        for inst in insts {
            if self.func.dfg().has_result(inst) {
                let value = self.func.dfg().inst_result(inst);
                self.func.dfg_mut().replace_use(value, Value::invalid());
            }
            self.func.dfg_mut().remove_inst(inst);
        }
    }

    fn insert_at_end(&mut self) {
        panic!("insert_at_end() called on function")
    }

    fn insert_at_beginning(&mut self) {
        panic!("insert_at_beginning() called on function")
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
