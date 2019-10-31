// Copyright (c) 2017-2019 Fabian Schuiki

//! Global Common Subexpression Elimination

use crate::ir::prelude::*;
use crate::opt::prelude::*;

/// Global Common Subexpression Elimination
///
/// This pass implements global common subexpression elimination. It tries to
/// eliminate redundant instructions.
pub struct GlobalCommonSubexprElim;

impl Pass for GlobalCommonSubexprElim {
    fn run_on_inst(_ctx: &PassContext, _inst: Inst, _unit: &mut impl UnitBuilder) -> bool {
        // std::thread::sleep(std::time::Duration::from_millis(1));
        // debug!("Running on inst {}", inst.dump(unit.dfg(), unit.try_cfg()));
        false
    }
}
