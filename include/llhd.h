// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h> // remove later

typedef struct llhd_module * llhd_module_t;
typedef struct llhd_type * llhd_type_t;
typedef struct llhd_value * llhd_value_t;

enum llhd_type_kind {
	LLHD_TYPE_VOID   = 1,
	LLHD_TYPE_LABEL  = 2,
	LLHD_TYPE_TIME   = 3,
	LLHD_TYPE_INT    = 4,
	LLHD_TYPE_LOGIC  = 5,
	LLHD_TYPE_STRUCT = 6,
	LLHD_TYPE_ARRAY  = 7,
	LLHD_TYPE_PTR    = 8,
	LLHD_TYPE_SIGNAL = 9,
	LLHD_TYPE_FUNC   = 10,
	LLHD_TYPE_COMP   = 11,
};

enum llhd_value_kind {
	LLHD_VALUE_UNIT  = 1,
	LLHD_VALUE_CONST = 2,
	LLHD_VALUE_INST  = 3,
};

enum llhd_unit_kind {
	LLHD_UNIT_DECL       = 1,
	LLHD_UNIT_DEF_FUNC   = 2,
	LLHD_UNIT_DEF_ENTITY = 3,
	LLHD_UNIT_DEF_PROC   = 4,
};

enum llhd_inst_kind {
	LLHD_INST_BRANCH = 1,
	LLHD_INST_UNARY  = 2,
	LLHD_INST_BINARY = 3,
};

enum llhd_unary_op {
	LLHD_UNARY_NOT   = 1,
};

enum llhd_binary_op {
	LLHD_BINARY_ADD  = 1,
	LLHD_BINARY_SUB  = 2,
	LLHD_BINARY_MUL  = 3,
	LLHD_BINARY_UDIV = 4,
	LLHD_BINARY_UREM = 5,
	LLHD_BINARY_SDIV = 6,
	LLHD_BINARY_SREM = 7,
	LLHD_BINARY_LSL  = 8,
	LLHD_BINARY_LSR  = 9,
	LLHD_BINARY_ASR  = 10,
	LLHD_BINARY_AND  = 11,
	LLHD_BINARY_OR   = 12,
	LLHD_BINARY_XOR  = 13,
};

enum llhd_const_kind {
	LLHD_CONST_INT = 1,
};

llhd_module_t llhd_module_new(const char *name);
void llhd_module_free(llhd_module_t);
llhd_value_t llhd_module_get_first_unit(llhd_module_t);
llhd_value_t llhd_module_get_last_unit(llhd_module_t);
void llhd_verify_module_selfcontained(llhd_module_t);
const char *llhd_module_get_name(llhd_module_t);

llhd_value_t llhd_unit_next(llhd_value_t);
llhd_value_t llhd_unit_prev(llhd_value_t);
void llhd_unit_append_to(llhd_value_t,llhd_module_t);
void llhd_unit_prepend_to(llhd_value_t,llhd_module_t);
void llhd_unit_insert_after(llhd_value_t,llhd_value_t);
void llhd_unit_insert_before(llhd_value_t,llhd_value_t);
bool llhd_unit_is(llhd_value_t,int);
int llhd_unit_get_kind(llhd_value_t);
bool llhd_unit_is_def(llhd_value_t);
bool llhd_unit_is_decl(llhd_value_t);
llhd_value_t llhd_unit_get_first_block(llhd_value_t);
llhd_value_t llhd_unit_get_last_block(llhd_value_t);
unsigned llhd_unit_get_num_inputs(llhd_value_t);
unsigned llhd_unit_get_num_outputs(llhd_value_t);
llhd_value_t llhd_unit_get_input(llhd_value_t,unsigned);
llhd_value_t llhd_unit_get_output(llhd_value_t,unsigned);

llhd_value_t llhd_entity_get_first_inst(llhd_value_t);
llhd_value_t llhd_entity_get_last_inst(llhd_value_t);
unsigned llhd_entity_get_num_insts(llhd_value_t);

llhd_value_t llhd_block_next(llhd_value_t);
llhd_value_t llhd_block_prev(llhd_value_t);
llhd_value_t llhd_block_get_first_inst(llhd_value_t);
llhd_value_t llhd_block_get_last_inst(llhd_value_t);
bool llhd_block_is_entry(llhd_value_t);
bool llhd_block_has_predecessors(llhd_value_t);
bool llhd_block_has_successors(llhd_value_t);
void llhd_block_get_predecessors(llhd_value_t, llhd_value_t**, unsigned*);
void llhd_block_get_successors(llhd_value_t, llhd_value_t**, unsigned*);

llhd_value_t llhd_inst_next(llhd_value_t);
llhd_value_t llhd_inst_prev(llhd_value_t);
void llhd_inst_append_to(llhd_value_t,llhd_value_t);
void llhd_inst_prepend_to(llhd_value_t,llhd_value_t);
void llhd_inst_insert_after(llhd_value_t,llhd_value_t);
void llhd_inst_insert_before(llhd_value_t,llhd_value_t);
bool llhd_inst_is(llhd_value_t,int);
int llhd_inst_get_kind(llhd_value_t);

int llhd_inst_unary_get_op(llhd_value_t);
llhd_value_t llhd_inst_unary_get_arg(llhd_value_t);

int llhd_inst_binary_get_op(llhd_value_t);
const char *llhd_inst_binary_get_opname(llhd_value_t);
llhd_value_t llhd_inst_binary_get_lhs(llhd_value_t);
llhd_value_t llhd_inst_binary_get_rhs(llhd_value_t);

llhd_value_t llhd_inst_branch_new_uncond(llhd_value_t);
llhd_value_t llhd_inst_branch_new_cond(llhd_value_t, llhd_value_t, llhd_value_t);
llhd_value_t llhd_inst_branch_get_condition(llhd_value_t);
llhd_value_t llhd_inst_branch_get_dst(llhd_value_t);
llhd_value_t llhd_inst_branch_get_dst0(llhd_value_t);
llhd_value_t llhd_inst_branch_get_dst1(llhd_value_t);

bool llhd_const_is_null(llhd_value_t);
bool llhd_const_is(llhd_value_t,int);
int llhd_const_get_kind(llhd_value_t);
uint64_t llhd_const_int_get_value(llhd_value_t);
char *llhd_const_to_string(llhd_value_t);

bool llhd_value_is(llhd_value_t,int);
int llhd_value_get_kind(llhd_value_t);
bool llhd_value_is_const(llhd_value_t);
const char *llhd_value_get_name(llhd_value_t);
llhd_type_t llhd_value_get_type(llhd_value_t);
bool llhd_value_has_users(llhd_value_t);
unsigned llhd_value_get_num_users(llhd_value_t);
void llhd_value_replace_uses(llhd_value_t,llhd_value_t);
void llhd_value_unlink_from_parent(llhd_value_t);
void llhd_value_unlink_uses(llhd_value_t);
void llhd_value_unlink(llhd_value_t);
void llhd_value_ref(llhd_value_t);
void llhd_value_unref(llhd_value_t);
void llhd_value_free(llhd_value_t);

int llhd_type_get_kind(llhd_type_t);
unsigned llhd_type_get_length(llhd_type_t);
llhd_type_t llhd_type_get_subtype(llhd_type_t);
unsigned llhd_type_get_num_fields(llhd_type_t);
llhd_type_t llhd_type_get_field(llhd_type_t,unsigned);
unsigned llhd_type_get_num_inputs(llhd_type_t);
unsigned llhd_type_get_num_outputs(llhd_type_t);
llhd_type_t llhd_type_get_input(llhd_type_t,unsigned);
llhd_type_t llhd_type_get_output(llhd_type_t,unsigned);
void llhd_type_ref(llhd_type_t);
void llhd_type_unref(llhd_type_t);

void *llhd_alloc(size_t);
void *llhd_zalloc(size_t);
void *llhd_realloc(void*,size_t);
void llhd_free(void*);
