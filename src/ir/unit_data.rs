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

/// A function, process, or entity.
#[derive(Serialize, Deserialize)]
pub struct UnitData {
    pub kind: UnitKind,
    pub name: UnitName,
    pub sig: Signature,
    pub dfg: DataFlowGraph,
    pub cfg: ControlFlowGraph,
    pub layout: FunctionLayout,
}

impl UnitData {
    /// Create a new unit.
    pub fn new(kind: UnitKind, name: UnitName, sig: Signature) -> Self {
        match kind {
            UnitKind::Function => {
                assert!(!sig.has_outputs());
                assert!(sig.has_return_type());
            }
            UnitKind::Process | UnitKind::Entity => {
                assert!(sig.has_outputs());
                assert!(!sig.has_return_type());
            }
        }
        let mut data = Self {
            kind,
            name,
            sig,
            dfg: DataFlowGraph::new(),
            cfg: ControlFlowGraph::new(),
            layout: FunctionLayout::new(),
        };
        if kind == UnitKind::Entity {
            let bb = data.cfg.add_block();
            data.layout.append_block(bb);
            data.dfg.make_args_for_signature(&data.sig);
        }
        data.dfg.make_args_for_signature(&data.sig);
        data
    }
}

impl Index<Value> for UnitData {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.dfg.index(idx)
    }
}

impl Index<Inst> for UnitData {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.dfg.index(idx)
    }
}

impl Index<ExtUnit> for UnitData {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.dfg.index(idx)
    }
}

impl Index<Block> for UnitData {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.cfg.index(idx)
    }
}

impl IndexMut<Value> for UnitData {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Inst> for UnitData {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for UnitData {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.dfg.index_mut(idx)
    }
}

impl IndexMut<Block> for UnitData {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.cfg.index_mut(idx)
    }
}

impl Unit for UnitData {
    fn kind(&self) -> UnitKind {
        self.kind
    }

    fn get_data(&self) -> Option<&UnitData> {
        Some(self)
    }

    fn get_data_mut(&mut self) -> Option<&mut UnitData> {
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
        write!(
            f,
            "{} {} {} {{\n",
            self.kind,
            self.name,
            self.sig.dump(&self.dfg)
        )?;
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
        verifier.verify_unit(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified {}:", self.kind);
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }
}

/// Temporary object used to build a single `UnitData`.
pub struct UnitDataBuilder<'u> {
    /// The unit currently being built.
    pub func: &'u mut UnitData,
    /// The position where we are currently inserting instructions.
    pos: FunctionInsertPos,
}

impl<'u> UnitDataBuilder<'u> {
    /// Create a new unit builder.
    pub fn new(func: &'u mut UnitData) -> Self {
        Self {
            func,
            pos: FunctionInsertPos::None,
        }
    }
}

impl Index<Value> for UnitDataBuilder<'_> {
    type Output = ValueData;
    fn index(&self, idx: Value) -> &ValueData {
        self.func.index(idx)
    }
}

impl Index<Inst> for UnitDataBuilder<'_> {
    type Output = InstData;
    fn index(&self, idx: Inst) -> &InstData {
        self.func.index(idx)
    }
}

impl Index<ExtUnit> for UnitDataBuilder<'_> {
    type Output = ExtUnitData;
    fn index(&self, idx: ExtUnit) -> &ExtUnitData {
        self.func.index(idx)
    }
}

impl Index<Block> for UnitDataBuilder<'_> {
    type Output = BlockData;
    fn index(&self, idx: Block) -> &BlockData {
        self.func.index(idx)
    }
}

impl IndexMut<Value> for UnitDataBuilder<'_> {
    fn index_mut(&mut self, idx: Value) -> &mut ValueData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<Inst> for UnitDataBuilder<'_> {
    fn index_mut(&mut self, idx: Inst) -> &mut InstData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<ExtUnit> for UnitDataBuilder<'_> {
    fn index_mut(&mut self, idx: ExtUnit) -> &mut ExtUnitData {
        self.func.index_mut(idx)
    }
}

impl IndexMut<Block> for UnitDataBuilder<'_> {
    fn index_mut(&mut self, idx: Block) -> &mut BlockData {
        self.func.index_mut(idx)
    }
}

impl<'u> std::ops::Deref for UnitDataBuilder<'u> {
    type Target = UnitData;
    fn deref(&self) -> &UnitData {
        self.func
    }
}

impl<'u> std::ops::DerefMut for UnitDataBuilder<'u> {
    fn deref_mut(&mut self) -> &mut UnitData {
        self.func
    }
}

impl UnitBuilder for UnitDataBuilder<'_> {
    type Unit = UnitData;

    fn unit(&self) -> &UnitData {
        self.func
    }

    fn unit_mut(&mut self) -> &mut UnitData {
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
        self.pos = FunctionInsertPos::Append(self.unit().func_layout().entry());
    }

    fn insert_at_beginning(&mut self) {
        self.pos = FunctionInsertPos::Prepend(self.unit().func_layout().entry());
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
