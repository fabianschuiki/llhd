// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h> // remove later
#include <stdint.h> // remove later


#if defined(__GNUC__)
	#define LLHD_DEPRECATED __attribute__((deprecated))
#elif defined(_MSC_VER)
	#define LLHD_DEPRECATED __declspec(deprecated)
#elif
	#define LLHD_DEPRECATED
#endif


typedef struct llhd_module * llhd_module_t;
typedef struct llhd_type * llhd_type_t;
typedef struct llhd_value * llhd_value_t;
// typedef struct llhd_apint * llhd_apint_t;
typedef struct llhd_list * llhd_list_t;
typedef uint64_t llhd_apint_t;


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


#define LLHD_KIND(id,level,parent) ( ((id) << (level)*8) | (level) | ((parent) & ~3) )
enum llhd_value_kind {
	LLHD_MASK_VALUE          = LLHD_KIND(0xFF, 3, 0),
	LLHD_MASK_UNIT           = LLHD_KIND(0xFF, 2, LLHD_MASK_VALUE),
	LLHD_MASK_CONST          = LLHD_KIND(0xFF, 2, LLHD_MASK_VALUE),
	LLHD_MASK_INST           = LLHD_KIND(0xFF, 2, LLHD_MASK_VALUE),
	LLHD_MASK_UNARY          = LLHD_KIND(0xFF, 1, LLHD_MASK_INST),
	LLHD_MASK_BINARY         = LLHD_KIND(0xFF, 1, LLHD_MASK_INST),
	LLHD_MASK_COMPARE        = LLHD_KIND(0xFF, 1, LLHD_MASK_INST),

	/* values */
	LLHD_VALUE_UNIT          = LLHD_KIND(  1, 3, 0),
	LLHD_VALUE_CONST         = LLHD_KIND(  2, 3, 0),
	LLHD_VALUE_INST          = LLHD_KIND(  3, 3, 0),
	LLHD_VALUE_PARAM         = LLHD_KIND(  4, 3, 0),
	LLHD_VALUE_BLOCK         = LLHD_KIND(  5, 3, 0),

	/* units */
	LLHD_UNIT_DECL           = LLHD_KIND(  1, 2, LLHD_VALUE_UNIT),
	LLHD_UNIT_DEF_FUNC       = LLHD_KIND(  2, 2, LLHD_VALUE_UNIT),
	LLHD_UNIT_DEF_ENTITY     = LLHD_KIND(  3, 2, LLHD_VALUE_UNIT),
	LLHD_UNIT_DEF_PROC       = LLHD_KIND(  4, 2, LLHD_VALUE_UNIT),

	/* constants */
	LLHD_CONST_INT           = LLHD_KIND(  1, 2, LLHD_VALUE_CONST),

	/* instructions */
	LLHD_INST_BRANCH         = LLHD_KIND(  1, 2, LLHD_VALUE_INST),
	LLHD_INST_UNARY          = LLHD_KIND(  2, 2, LLHD_VALUE_INST),
	LLHD_INST_BINARY         = LLHD_KIND(  3, 2, LLHD_VALUE_INST),
	LLHD_INST_SIGNAL         = LLHD_KIND(  4, 2, LLHD_VALUE_INST),
	LLHD_INST_COMPARE        = LLHD_KIND(  5, 2, LLHD_VALUE_INST),
	LLHD_INST_DRIVE          = LLHD_KIND(  6, 2, LLHD_VALUE_INST),
	LLHD_INST_RET            = LLHD_KIND(  7, 2, LLHD_VALUE_INST),
	LLHD_INST_INST           = LLHD_KIND(  8, 2, LLHD_VALUE_INST),
	LLHD_INST_CALL           = LLHD_KIND(  9, 2, LLHD_VALUE_INST),
	LLHD_INST_EXTRACT        = LLHD_KIND( 10, 2, LLHD_VALUE_INST),
	LLHD_INST_INSERT         = LLHD_KIND( 11, 2, LLHD_VALUE_INST),
	LLHD_INST_REG            = LLHD_KIND( 12, 2, LLHD_VALUE_INST),

	/* unary instructions */
	LLHD_UNARY_NOT           = LLHD_KIND(  1, 1, LLHD_INST_UNARY),

	/* binary instructions */
	LLHD_BINARY_ADD          = LLHD_KIND(  1, 1, LLHD_INST_BINARY),
	LLHD_BINARY_SUB          = LLHD_KIND(  2, 1, LLHD_INST_BINARY),
	LLHD_BINARY_MUL          = LLHD_KIND(  3, 1, LLHD_INST_BINARY),
	LLHD_BINARY_UDIV         = LLHD_KIND(  4, 1, LLHD_INST_BINARY),
	LLHD_BINARY_UREM         = LLHD_KIND(  5, 1, LLHD_INST_BINARY),
	LLHD_BINARY_SDIV         = LLHD_KIND(  6, 1, LLHD_INST_BINARY),
	LLHD_BINARY_SREM         = LLHD_KIND(  7, 1, LLHD_INST_BINARY),
	LLHD_BINARY_LSL          = LLHD_KIND(  8, 1, LLHD_INST_BINARY),
	LLHD_BINARY_LSR          = LLHD_KIND(  9, 1, LLHD_INST_BINARY),
	LLHD_BINARY_ASR          = LLHD_KIND( 10, 1, LLHD_INST_BINARY),
	LLHD_BINARY_AND          = LLHD_KIND( 11, 1, LLHD_INST_BINARY),
	LLHD_BINARY_OR           = LLHD_KIND( 12, 1, LLHD_INST_BINARY),
	LLHD_BINARY_XOR          = LLHD_KIND( 13, 1, LLHD_INST_BINARY),

	/* compare instructions */
	LLHD_CMP_EQ              = LLHD_KIND(  1, 1, LLHD_INST_COMPARE),
	LLHD_CMP_NE              = LLHD_KIND(  2, 1, LLHD_INST_COMPARE),
	LLHD_CMP_ULT             = LLHD_KIND(  3, 1, LLHD_INST_COMPARE),
	LLHD_CMP_UGT             = LLHD_KIND(  4, 1, LLHD_INST_COMPARE),
	LLHD_CMP_ULE             = LLHD_KIND(  5, 1, LLHD_INST_COMPARE),
	LLHD_CMP_UGE             = LLHD_KIND(  6, 1, LLHD_INST_COMPARE),
	LLHD_CMP_SLT             = LLHD_KIND(  7, 1, LLHD_INST_COMPARE),
	LLHD_CMP_SGT             = LLHD_KIND(  8, 1, LLHD_INST_COMPARE),
	LLHD_CMP_SLE             = LLHD_KIND(  9, 1, LLHD_INST_COMPARE),
	LLHD_CMP_SGE             = LLHD_KIND( 10, 1, LLHD_INST_COMPARE),
};
#undef LLHD_KIND

#define LLHD_KIND_VALUE(k)   ((k) >> 24 & 0xFF)
#define LLHD_KIND_UNIT(k)    ((k) >> 16 & 0xFF)
#define LLHD_KIND_CONST(k)   ((k) >> 16 & 0xFF)
#define LLHD_KIND_INST(k)    ((k) >> 16 & 0xFF)
#define LLHD_KIND_UNARY(k)   ((k) >>  8 & 0xFF)
#define LLHD_KIND_BINARY(k)  ((k) >>  8 & 0xFF)
#define LLHD_KIND_COMPARE(k) ((k) >>  8 & 0xFF)

#define LLHD_ISA(k,a) ((k) >> ((a&3)*8) == (a) >> ((a&3)*8))
#define LLHD_AS(k,a)  (((k) & (a) & ~3) | ((a) & 3))


llhd_module_t llhd_module_new(const char*);
void llhd_module_free(llhd_module_t);
llhd_value_t llhd_module_get_first_unit(llhd_module_t);
llhd_value_t llhd_module_get_last_unit(llhd_module_t);
llhd_list_t llhd_module_get_units(llhd_module_t);
void llhd_verify_module_selfcontained(llhd_module_t);
const char *llhd_module_get_name(llhd_module_t);

llhd_list_t llhd_unit_first(llhd_list_t);
llhd_list_t llhd_unit_last(llhd_list_t);
llhd_value_t llhd_unit_next(llhd_list_t,llhd_list_t*);
llhd_value_t llhd_unit_prev(llhd_list_t,llhd_list_t*);
void llhd_unit_append_to(llhd_value_t,llhd_module_t);
void llhd_unit_prepend_to(llhd_value_t,llhd_module_t);
void llhd_unit_insert_after(llhd_value_t,llhd_value_t);
void llhd_unit_insert_before(llhd_value_t,llhd_value_t);
bool llhd_unit_is_def(llhd_value_t);
bool llhd_unit_is_decl(llhd_value_t);
llhd_value_t llhd_unit_get_first_block(llhd_value_t);
llhd_value_t llhd_unit_get_last_block(llhd_value_t);
llhd_list_t llhd_unit_get_blocks(llhd_value_t);
unsigned llhd_unit_get_num_inputs(llhd_value_t);
unsigned llhd_unit_get_num_outputs(llhd_value_t);
llhd_value_t llhd_unit_get_input(llhd_value_t,unsigned);
llhd_value_t llhd_unit_get_output(llhd_value_t,unsigned);

llhd_value_t llhd_entity_new(llhd_type_t,const char*);
llhd_value_t llhd_entity_get_first_inst(llhd_value_t);
llhd_value_t llhd_entity_get_last_inst(llhd_value_t);
unsigned llhd_entity_get_num_insts(llhd_value_t);

llhd_value_t llhd_proc_new(llhd_type_t,const char*);
llhd_value_t llhd_func_new(llhd_type_t,const char*);

llhd_value_t llhd_block_new(const char*);
llhd_list_t llhd_block_first(llhd_list_t);
llhd_list_t llhd_block_last(llhd_list_t);
llhd_value_t llhd_block_next(llhd_list_t,llhd_list_t*);
llhd_value_t llhd_block_prev(llhd_list_t,llhd_list_t*);
void llhd_block_append_to(llhd_value_t,llhd_value_t);
void llhd_block_prepend_to(llhd_value_t,llhd_value_t);
void llhd_block_insert_after(llhd_value_t,llhd_value_t);
void llhd_block_insert_before(llhd_value_t,llhd_value_t);
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
llhd_value_t llhd_inst_get_parent(llhd_value_t);
unsigned llhd_inst_get_num_params(llhd_value_t);
llhd_value_t llhd_inst_get_param(llhd_value_t,unsigned);

llhd_value_t llhd_inst_unary_new(int,llhd_value_t,const char*);
int llhd_inst_unary_get_op(llhd_value_t);
llhd_value_t llhd_inst_unary_get_arg(llhd_value_t);

llhd_value_t llhd_inst_binary_new(int,llhd_value_t,llhd_value_t,const char*);
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

llhd_value_t llhd_inst_compare_new(int,llhd_value_t,llhd_value_t,const char*);
int llhd_inst_compare_get_op(llhd_value_t);
const char *llhd_inst_compare_get_opname(llhd_value_t);
llhd_value_t llhd_inst_compare_get_lhs(llhd_value_t);
llhd_value_t llhd_inst_compare_get_rhs(llhd_value_t);

llhd_value_t llhd_inst_drive_new(llhd_value_t,llhd_value_t);
llhd_value_t llhd_inst_drive_get_sig(llhd_value_t);
llhd_value_t llhd_inst_drive_get_val(llhd_value_t);

llhd_value_t llhd_inst_ret_new();
llhd_value_t llhd_inst_ret_new_one(llhd_value_t);
llhd_value_t llhd_inst_ret_new_many(llhd_value_t*,unsigned);
unsigned llhd_inst_ret_get_num_args(llhd_value_t);
llhd_value_t llhd_inst_ret_get_arg(llhd_value_t,unsigned);

llhd_value_t llhd_inst_instance_new(llhd_value_t,llhd_value_t*,unsigned,llhd_value_t*,unsigned,const char*);
llhd_value_t llhd_inst_inst_get_comp(llhd_value_t);
unsigned llhd_inst_inst_get_num_inputs(llhd_value_t);
unsigned llhd_inst_inst_get_num_outputs(llhd_value_t);
llhd_value_t llhd_inst_inst_get_input(llhd_value_t,unsigned);
llhd_value_t llhd_inst_inst_get_output(llhd_value_t,unsigned);

llhd_value_t llhd_inst_call_new(llhd_value_t,llhd_value_t*,unsigned,const char*);
llhd_value_t llhd_inst_call_get_func(llhd_value_t);
unsigned llhd_inst_call_get_num_args(llhd_value_t);
llhd_value_t llhd_inst_call_get_arg(llhd_value_t,unsigned);

llhd_value_t llhd_inst_sig_new(llhd_type_t,const char*);

llhd_value_t llhd_inst_extract_new(llhd_value_t,unsigned,const char*);
unsigned llhd_inst_extract_get_index(llhd_value_t);
llhd_value_t llhd_inst_extract_get_target(llhd_value_t);

llhd_value_t llhd_inst_insert_new(llhd_value_t,unsigned,llhd_value_t,const char*);
unsigned llhd_inst_insert_get_index(llhd_value_t);
llhd_value_t llhd_inst_insert_get_target(llhd_value_t);
llhd_value_t llhd_inst_insert_get_value(llhd_value_t);

llhd_value_t llhd_inst_reg_new(llhd_value_t,llhd_value_t,const char*);
llhd_value_t llhd_inst_reg_get_value(llhd_value_t);
llhd_value_t llhd_inst_reg_get_strobe(llhd_value_t);

bool llhd_const_is_null(llhd_value_t);
char *llhd_const_to_string(llhd_value_t);
llhd_value_t llhd_const_int_new(unsigned,llhd_apint_t);
llhd_apint_t llhd_const_int_get_value(llhd_value_t);

llhd_value_t llhd_value_copy(llhd_value_t);
bool llhd_value_is(llhd_value_t,int);
int llhd_value_get_kind(llhd_value_t);
bool llhd_value_is_const(llhd_value_t);
void llhd_value_set_name(llhd_value_t,const char*);
const char *llhd_value_get_name(llhd_value_t);
llhd_type_t llhd_value_get_type(llhd_value_t);
bool llhd_value_has_users(llhd_value_t);
unsigned llhd_value_get_num_users(llhd_value_t);
void llhd_value_replace_uses(llhd_value_t,llhd_value_t);
void llhd_value_substitute(llhd_value_t,llhd_value_t,llhd_value_t);
void llhd_value_unlink_from_parent(llhd_value_t);
void llhd_value_unlink_uses(llhd_value_t);
void llhd_value_unlink(llhd_value_t);
void llhd_value_ref(llhd_value_t);
void llhd_value_unref(llhd_value_t);
void llhd_value_free(llhd_value_t);

llhd_type_t llhd_type_new_comp(llhd_type_t*,unsigned,llhd_type_t*,unsigned);
llhd_type_t llhd_type_new_func(llhd_type_t*,unsigned,llhd_type_t*,unsigned);
llhd_type_t llhd_type_new_int(unsigned);
llhd_type_t llhd_type_new_void();
llhd_type_t llhd_type_new_label();
llhd_type_t llhd_type_new_struct(llhd_type_t*,unsigned);
llhd_type_t llhd_type_new_array(llhd_type_t,unsigned);
bool llhd_type_is(llhd_type_t,int);
int llhd_type_get_kind(llhd_type_t);
int llhd_type_cmp(llhd_type_t,llhd_type_t);
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

void llhd_fold_constants(llhd_value_t);
void llhd_asm_write_module(llhd_module_t,FILE*);
void llhd_asm_write_unit(llhd_value_t,FILE*);
void llhd_asm_write_type(llhd_type_t,FILE*);
