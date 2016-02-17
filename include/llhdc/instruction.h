#pragma once

typedef struct llhd_drive_inst llhd_drive_inst_t;
typedef struct llhd_branch_inst llhd_branch_inst_t;

struct llhd_value {
	char *name;
}

struct llhd_unit {
	llhd_value_t base;
	llhd_module_t *parent;
}

struct llhd_func {
	llhd_unit_t base;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
}

struct llhd_proc {
	llhd_unit_t base;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
}

struct llhd_entity {
	llhd_unit_t base;
	llhd_inst_t *inst_head;
	llhd_inst_t *inst_tail;
}

struct llhd_basic_block {
	llhd_value_t base;
	llhd_unit_t *parent;
	llhd_basic_block_t *prev;
	llhd_basic_block_t *next;
}

struct llhd_inst {
	llhd_value_t base;
	llhd_basic_block_t *parent;
	llhd_inst_t *prev;
	llhd_inst_t *next;
}

struct llhd_drive_inst {
	llhd_inst_t base;
	llhd_value_t *target;
	llhd_value_t *value;
}

struct llhd_branch_inst {
	llhd_inst_t base;
	llhd_basic_block_t *dst0;
	llhd_basic_block_t *dst1;
	llhd_value_t *cond;
}

void llhd_dispose_drive_inst(llhd_drive_inst_t *inst) {
	assert(inst);
	llhd_dispose_inst(&inst->base);
}

void llhd_dispose_branch_inst(llhd_branch_inst_t *inst) {

}
