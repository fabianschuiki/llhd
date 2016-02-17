#pragma once

typedef struct llhd_drive_inst llhd_drive_inst_t;
typedef struct llhd_branch_inst llhd_branch_inst_t;

struct llhd_drive_inst {
	llhd_inst_t base;
	llhd_value_t *target;
	llhd_value_t *value;
}

struct llhd_branch_inst {
	llhd_inst_t base;
	llhd_basic_block_t *if0;
	llhd_basic_block_t *if1;
	llhd_value_t *cond;
}

void llhd_dispose_drive_inst(llhd_drive_inst_t *inst) {
	assert(inst);
	llhd_dispose_inst(&inst->base);
}

void llhd_dispose_branch_inst(llhd_branch_inst_t *inst) {

}
