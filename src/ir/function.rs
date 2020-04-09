// Copyright (c) 2017-2020 Fabian Schuiki

//! Representation of LLHD functions.

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

/// A function.
#[derive(Serialize, Deserialize)]
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

impl Index<Value> for Function {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.dfg.index(idx)
    }
}

impl Index<Inst> for Function {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.dfg.index(idx)
    }
}

impl Index<ExtUnit> for Function {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.dfg.index(idx)
    }
}

impl Index<Block> for Function {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.cfg.index(idx)
    }
}

impl IndexMut<Value> for Function {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Inst> for Function {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for Function {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Block> for Function {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.cfg.index_mut(idx)
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

impl Index<Value> for FunctionBuilder<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.func.index(idx)
    }
}

impl Index<Inst> for FunctionBuilder<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.func.index(idx)
    }
}

impl Index<ExtUnit> for FunctionBuilder<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.func.index(idx)
    }
}

impl Index<Block> for FunctionBuilder<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.func.index(idx)
    }
}

impl IndexMut<Value> for FunctionBuilder<'_> {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<Inst> for FunctionBuilder<'_> {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for FunctionBuilder<'_> {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<Block> for FunctionBuilder<'_> {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.func.index_mut(idx)
    }
}

impl<'u> std::ops::Deref for FunctionBuilder<'u> {
    type Target = Function;
    fn deref(&self) -> &Function {
        self.func
    }
}

impl<'u> std::ops::DerefMut for FunctionBuilder<'u> {
    fn deref_mut(&mut self) -> &mut Function {
        self.func
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
        self.pos.add_inst(inst, &mut self.func.layout);
        inst
    }

    fn remove_inst(&mut self, inst: Inst) {
        self.func.dfg.remove_inst(inst);
        self.pos.remove_inst(inst, &self.func.layout);
        self.func.layout.remove_inst(inst);
    }

    fn block(&mut self) -> Block {
        let bb = self.func.cfg.add_block();
        self.func.layout.append_block(bb);
        bb
    }

    fn remove_block(&mut self, bb: Block) {
        let insts: Vec<_> = self.func.layout.insts(bb).collect();
        self.func.dfg_mut().remove_block_use(bb);
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
