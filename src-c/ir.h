// Copyright (c) 2016 Fabian Schuiki
#pragma once

struct llhd_module {
	char *name;
	llhd_unit_t *unit_head;
	llhd_unit_t *unit_tail;
};

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

struct llhd_const {
	llhd_value_t _value;
};

struct llhd_const_logic {
	llhd_const_t _const;
	char *value;
};

struct llhd_const_int {
	llhd_const_t _const;
	char *value;
};

struct llhd_const_time {
	llhd_const_t _const;
	char *value;
};

struct llhd_arg {
	llhd_value_t _value;
	llhd_unit_t *parent;
};

struct llhd_unit {
	llhd_value_t _value;
	llhd_module_t *parent;
	llhd_unit_t *prev;
	llhd_unit_t *next;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
	unsigned bb_num;
};

struct llhd_func {
	llhd_unit_t _unit;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
};

struct llhd_proc {
	llhd_unit_t _unit;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
	unsigned num_in;
	llhd_arg_t **in;
	unsigned num_out;
	llhd_arg_t **out;
};

struct llhd_entity {
	llhd_unit_t _unit;
	llhd_basic_block_t *bb_head;
	llhd_basic_block_t *bb_tail;
	unsigned num_in;
	llhd_arg_t **in;
	unsigned num_out;
	llhd_arg_t **out;
};

// basic_block:
// - append to unit
// - insert before block
// - insert after block
// - remove from parent
// - dump
struct llhd_basic_block {
	llhd_value_t _value;
	llhd_unit_t *parent;
	llhd_basic_block_t *prev;
	llhd_basic_block_t *next;
	llhd_inst_t *inst_head;
	llhd_inst_t *inst_tail;
};

// inst:
// - append to basic block
// - insert after inst
// - insert before inst
// - remove from parent
// - dump
struct llhd_inst {
	llhd_value_t _value;
	llhd_basic_block_t *parent;
	llhd_inst_t *prev;
	llhd_inst_t *next;
};

struct llhd_drive_inst {
	llhd_inst_t _inst;
	llhd_value_t *target;
	llhd_value_t *value;
};

struct llhd_branch_inst {
	llhd_inst_t _inst;
	llhd_value_t *cond; // NULL if uncond
	llhd_basic_block_t *dst1; // dst if uncond
	llhd_basic_block_t *dst0; // NULL if uncond
};

struct llhd_compare_inst {
	llhd_inst_t _inst;
	llhd_compare_mode_t mode;
	llhd_value_t *lhs;
	llhd_value_t *rhs;
};

struct llhd_unary_inst {
	llhd_inst_t _inst;
	llhd_unary_op_t op;
	llhd_value_t *arg;
};

struct llhd_binary_inst {
	llhd_inst_t _inst;
	llhd_binary_op_t op;
	llhd_value_t *lhs;
	llhd_value_t *rhs;
};

struct llhd_ret_inst {
	llhd_inst_t _inst;
	unsigned num_values;
	llhd_value_t *values[];
};

struct llhd_wait_inst {
	llhd_inst_t _inst;
	llhd_value_t *duration; // time or label
};

struct llhd_signal_inst {
	llhd_inst_t _inst;
};

struct llhd_instance_inst {
	llhd_inst_t _inst;
	llhd_value_t *value;
	unsigned num_in;
	llhd_value_t **in;
	unsigned num_out;
	llhd_value_t **out;
};

struct llhd_type {
	llhd_type_kind_t kind;
	unsigned length;
	llhd_type_t *inner[];
};

void llhd_value_init(llhd_value_t *V, const char *name, llhd_type_t *type);
void llhd_value_dispose(void*);
