// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdio.h>
#include <stdint.h>

#define TYPEDEF(name) typedef struct llhd_##name llhd_##name##_t
TYPEDEF(module);
TYPEDEF(value);
TYPEDEF(const);
TYPEDEF(const_logic);
TYPEDEF(const_int);
TYPEDEF(unit);
TYPEDEF(func);
TYPEDEF(proc);
TYPEDEF(arg);
TYPEDEF(entity);
TYPEDEF(basic_block);
TYPEDEF(inst);
TYPEDEF(drive_inst);
TYPEDEF(branch_inst);
TYPEDEF(compare_inst);
TYPEDEF(unary_inst);
TYPEDEF(binary_inst);
TYPEDEF(ret_inst);
TYPEDEF(type);
TYPEDEF(struct_field);
#undef TYPEDEF

typedef enum llhd_compare_mode llhd_compare_mode_t;
typedef enum llhd_unary_op llhd_unary_op_t;
typedef enum llhd_binary_op llhd_binary_op_t;


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

struct llhd_arg {
	llhd_value_t _value;
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

// basic_block:
// - append to unit
// - insert before block
// - insert after block
// - remove from parent
// - dump
struct llhd_basic_block {
	llhd_value_t _base;
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

enum llhd_compare_mode {
	LLHD_EQ,
	LLHD_NE,
	LLHD_UGT,
	LLHD_ULT,
	LLHD_UGE,
	LLHD_ULE,
	LLHD_SGT,
	LLHD_SLT,
	LLHD_SGE,
	LLHD_SLE,
};

struct llhd_compare_inst {
	llhd_inst_t _inst;
	llhd_compare_mode_t mode;
	llhd_value_t *lhs;
	llhd_value_t *rhs;
};

enum llhd_unary_op {
	LLHD_NOT,
};

struct llhd_unary_inst {
	llhd_inst_t _inst;
	llhd_unary_op_t op;
	llhd_value_t *arg;
};

enum llhd_binary_op {
	LLHD_ADD,
	LLHD_SUB,
	LLHD_MUL,
	LLHD_DIV,
	LLHD_AND,
	LLHD_OR,
	LLHD_XOR,
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

enum llhd_type_kind {
	LLHD_VOID_TYPE,
	LLHD_LABEL_TYPE,
	LLHD_INT_TYPE,
	LLHD_LOGIC_TYPE,
	LLHD_STRUCT_TYPE,
	LLHD_ARRAY_TYPE,
	LLHD_PTR_TYPE,
};

struct llhd_type {
	enum llhd_type_kind kind;
	unsigned length;
	llhd_type_t *inner[];
};

void llhd_init_value(llhd_value_t *V, const char *name, llhd_type_t *type);
void llhd_dispose_value(void*);
void llhd_destroy_value(void*);
void llhd_dump_value(void*, FILE*);
void llhd_dump_value_name(void*, FILE*);
void llhd_value_set_name(void*, const char*);
const char *llhd_value_get_name(void*);

llhd_const_int_t *llhd_make_const_int(unsigned width, const char *value);
llhd_const_logic_t *llhd_make_const_logic(unsigned width, const char *value);

llhd_proc_t *llhd_make_proc(const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry);

llhd_basic_block_t *llhd_make_basic_block(const char *name);
void llhd_insert_basic_block_before(llhd_basic_block_t *BB, llhd_basic_block_t *before);
void llhd_insert_basic_block_after(llhd_basic_block_t *BB, llhd_basic_block_t *after);

llhd_drive_inst_t *llhd_make_drive_inst(llhd_value_t *target, llhd_value_t *value);
llhd_compare_inst_t *llhd_make_compare_inst(llhd_compare_mode_t mode, llhd_value_t *lhs, llhd_value_t *rhs);
llhd_branch_inst_t *llhd_make_conditional_branch_inst(llhd_value_t *cond, llhd_basic_block_t *dst1, llhd_basic_block_t *dst0);
llhd_branch_inst_t *llhd_make_unconditional_branch_inst(llhd_basic_block_t *dst);
llhd_unary_inst_t *llhd_make_unary_inst(llhd_unary_op_t op, llhd_value_t *arg);
llhd_binary_inst_t *llhd_make_binary_inst(llhd_binary_op_t op, llhd_value_t *lhs, llhd_value_t *rhs);
llhd_ret_inst_t *llhd_make_ret_inst(llhd_value_t **values, unsigned num_values);

llhd_arg_t *llhd_make_arg(const char *name, llhd_type_t *type);

void llhd_add_inst(llhd_inst_t *I, llhd_basic_block_t *BB);

llhd_type_t *llhd_make_void_type();
llhd_type_t *llhd_make_label_type();
llhd_type_t *llhd_make_int_type(unsigned width);
llhd_type_t *llhd_make_logic_type(unsigned width);
llhd_type_t *llhd_make_struct_type(llhd_type_t **fields, unsigned num_fields);
llhd_type_t *llhd_make_array_type(llhd_type_t *element, unsigned length);
llhd_type_t *llhd_make_ptr_type(llhd_type_t *to);
llhd_type_t *llhd_copy_type(llhd_type_t *T);
void llhd_destroy_type(llhd_type_t *T);
void llhd_dump_type(llhd_type_t *T, FILE *f);
int llhd_equal_types(llhd_type_t*, llhd_type_t*);
int llhd_type_is_void(llhd_type_t*);
int llhd_type_is_label(llhd_type_t*);
int llhd_type_is_int(llhd_type_t*);
int llhd_type_is_int_width(llhd_type_t*, unsigned);
int llhd_type_is_logic(llhd_type_t*);
int llhd_type_is_logic_width(llhd_type_t*, unsigned);
int llhd_type_is_struct(llhd_type_t*);
int llhd_type_is_array(llhd_type_t*);
int llhd_type_is_ptr(llhd_type_t*);
