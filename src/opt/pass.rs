// Copyright (c) 2017-2019 Fabian Schuiki

use crate::ir::prelude::*;
use crate::ir::ModUnitData;
use rayon::prelude::*;

/// An optimization pass.
///
/// The optimization infrastructure will always call `run_on_module()`. However,
/// implementors of the trait should override the function at their desired
/// level of granularity. The `Pass` trait provides a sane default for all
/// `run_*()` functions.
pub trait Pass {
    /// Run this pass on an entire module.
    fn run_on_module(ctx: &PassContext, module: &mut Module) -> bool {
        module
            .units
            .storage
            .par_iter_mut()
            .map(|(_, unit)| match unit {
                ModUnitData::Function(ref mut u) => {
                    Self::run_on_function(ctx, &mut FunctionBuilder::new(u))
                }
                ModUnitData::Process(ref mut u) => {
                    Self::run_on_process(ctx, &mut ProcessBuilder::new(u))
                }
                ModUnitData::Entity(ref mut u) => {
                    Self::run_on_entity(ctx, &mut EntityBuilder::new(u))
                }
                _ => false,
            })
            .reduce(|| false, |a, b| a || b)
    }

    /// Run this pass on an entire function.
    fn run_on_function(ctx: &PassContext, func: &mut FunctionBuilder) -> bool {
        let mut modified = false;
        let mut insts = vec![];
        for bb in func.func.layout.blocks() {
            for inst in func.func.layout.insts(bb) {
                insts.push(inst);
            }
        }
        for inst in insts {
            modified |= Self::run_on_inst(ctx, inst, func);
        }
        modified
    }

    /// Run this pass on an entire process.
    fn run_on_process(ctx: &PassContext, prok: &mut ProcessBuilder) -> bool {
        let mut modified = false;
        let mut insts = vec![];
        for bb in prok.prok.layout.blocks() {
            for inst in prok.prok.layout.insts(bb) {
                insts.push(inst);
            }
        }
        for inst in insts {
            modified |= Self::run_on_inst(ctx, inst, prok);
        }
        modified
    }

    /// Run this pass on an entire entity.
    fn run_on_entity(ctx: &PassContext, entity: &mut EntityBuilder) -> bool {
        let mut modified = false;
        for inst in entity.entity.layout.insts().collect::<Vec<_>>() {
            modified |= Self::run_on_inst(ctx, inst, entity);
        }
        modified
    }

    /// Run this pass on an instruction.
    #[allow(unused_variables)]
    fn run_on_inst(ctx: &PassContext, inst: Inst, unit: &mut impl UnitBuilder) -> bool {
        false
    }
}

/// Additional context and configuration for optimizations.
pub struct PassContext;
