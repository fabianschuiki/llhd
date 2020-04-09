// Copyright (c) 2017-2020 Fabian Schuiki

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
                ModUnitData::Function(ref mut u) =>
                {
                    #[allow(deprecated)]
                    Self::run_on_function(ctx, &mut FunctionBuilder::new(u))
                }
                ModUnitData::Process(ref mut u) =>
                {
                    #[allow(deprecated)]
                    Self::run_on_process(ctx, &mut ProcessBuilder::new(u))
                }
                ModUnitData::Entity(ref mut u) =>
                {
                    #[allow(deprecated)]
                    Self::run_on_entity(ctx, &mut EntityBuilder::new(u))
                }
                ModUnitData::Data(ref mut u) => {
                    Self::run_on_unit(ctx, &mut UnitDataBuilder::new(u))
                }
                _ => false,
            })
            .reduce(|| false, |a, b| a || b)
    }

    /// Run this pass on an entire function.
    #[allow(deprecated)]
    fn run_on_function(ctx: &PassContext, func: &mut FunctionBuilder) -> bool {
        Self::run_on_cfg(ctx, func)
    }

    /// Run this pass on an entire process.
    #[allow(deprecated)]
    fn run_on_process(ctx: &PassContext, prok: &mut ProcessBuilder) -> bool {
        Self::run_on_cfg(ctx, prok)
    }

    /// Run this pass on an entire entity.
    #[allow(deprecated)]
    fn run_on_entity(ctx: &PassContext, entity: &mut EntityBuilder) -> bool {
        Self::run_on_cfg(ctx, entity)
    }

    /// Run this pass on an entire unit.
    fn run_on_unit(ctx: &PassContext, data: &mut UnitDataBuilder) -> bool {
        Self::run_on_cfg(ctx, data)
    }

    /// Run this pass on an entire function or process.
    fn run_on_cfg(ctx: &PassContext, unit: &mut impl UnitBuilder) -> bool {
        let mut modified = false;
        let insts: Vec<_> = unit.func_layout().all_insts().collect();
        for inst in insts {
            modified |= Self::run_on_inst(ctx, inst, unit);
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
