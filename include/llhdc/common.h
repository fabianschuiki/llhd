#pragma once
#include <stdio.h>

#define TYPEDEF(name) typedef struct llhd_##name llhd_##name##_t
TYPEDEF(module);
TYPEDEF(value);
TYPEDEF(unit);
TYPEDEF(func);
TYPEDEF(proc);
TYPEDEF(arg);
TYPEDEF(entity);
TYPEDEF(basic_block);
TYPEDEF(inst);
TYPEDEF(drive_inst);
TYPEDEF(branch_inst);
TYPEDEF(type);
#undef TYPEDEF


struct llhd_value {
	struct llhd_value_intf *_intf;
	char *name;
	llhd_type_t *type;
};

typedef void (*llhd_value_intf_dispose_fn)(void*);
typedef void (*llhd_value_intf_dump_fn)(void*, FILE*);
struct llhd_value_intf {
	llhd_value_intf_dispose_fn dispose;
	llhd_value_intf_dump_fn dump;
};

struct llhd_arg {
	llhd_value_t _base;
	llhd_unit_t *parent;
};

struct llhd_func {
	llhd_value_t base;
	llhd_module_t *parent;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
};

struct llhd_proc {
	llhd_value_t _base;
	llhd_module_t *parent;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
	unsigned num_in;
	llhd_arg_t **in;
	unsigned num_out;
	llhd_arg_t **out;
};

struct llhd_entity {
	llhd_value_t base;
	llhd_module_t *parent;
	llhd_inst_t *inst_head;
	llhd_inst_t *inst_tail;
};

struct llhd_basic_block {
	llhd_value_t _base;
	llhd_unit_t *parent;
	llhd_basic_block_t *prev;
	llhd_basic_block_t *next;
	llhd_inst_t *inst_head;
	llhd_inst_t *inst_tail;
};

struct llhd_inst {
	llhd_value_t base;
	llhd_basic_block_t *parent;
	llhd_inst_t *prev;
	llhd_inst_t *next;
};

struct llhd_drive_inst {
	llhd_inst_t base;
	llhd_value_t *target;
	llhd_value_t *value;
};

struct llhd_branch_inst {
	llhd_inst_t base;
	llhd_basic_block_t *dst0;
	llhd_basic_block_t *dst1;
	llhd_value_t *cond;
};

void llhd_init_value(llhd_value_t *V, const char *name, llhd_type_t *type);
void llhd_dispose_value(void*);
void llhd_destroy_value(void*);
void llhd_dump_value(void*, FILE*);

llhd_proc_t *llhd_make_proc(const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry);
llhd_basic_block_t *llhd_make_basic_block(const char *name);
llhd_drive_inst_t *llhd_make_drive_inst();
llhd_arg_t *llhd_make_arg(const char *name, llhd_type_t *type);

void llhd_add_inst(llhd_inst_t *I, llhd_basic_block_t *BB);
