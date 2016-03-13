// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>

// Replaces every conditional branch which has a constant condition with a
// corresponding unconditional branch.
//
// Algorithm:
// 1) Iterate over every branch instruction.
// 2) If the condition is constant and its value can be evaluated, replace the
//    instruction with a new unconditional branch to the corresponding label.
void llhd_elide_const_branches(llhd_value_t proc) {
	llhd_value_t BB;
	for (BB = llhd_unit_get_first_block(proc); BB; BB = llhd_block_next(BB)) {
		llhd_value_t I, In;
		for (I = llhd_block_get_first_inst(BB); I; I = In) {
			In = llhd_inst_next(I);
			if (!llhd_inst_is(I, LLHD_INST_BRANCH))
				continue;

			llhd_value_t cond = llhd_inst_branch_get_condition(I);
			if (!cond || !llhd_value_is_const(cond))
				continue;

			llhd_value_t dst;
			if (llhd_const_is_null(cond))
				dst = llhd_inst_branch_get_dst0(I);
			else
				dst = llhd_inst_branch_get_dst1(I);

			llhd_value_t Irep = llhd_inst_branch_new_uncond(dst);
			llhd_inst_insert_before(Irep, I);
			llhd_value_unlink(I);
			llhd_value_free(I);
		}
	}
}
