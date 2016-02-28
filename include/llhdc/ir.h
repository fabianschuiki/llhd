// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include "llhdc/common.h"

#define LLHD_STRUCT(name) typedef struct llhd_##name llhd_##name##_t
#define LLHD_ENUM(name) typedef enum llhd_##name llhd_##name##_t

LLHD_STRUCT(module);
// LLHD_STRUCT(value);
// LLHD_STRUCT(const);
// LLHD_STRUCT(const_logic);
// LLHD_STRUCT(const_int);
// LLHD_STRUCT(const_time);
// LLHD_STRUCT(unit);
// LLHD_STRUCT(func);
// LLHD_STRUCT(proc);
// LLHD_STRUCT(arg);
// LLHD_STRUCT(entity);
// LLHD_STRUCT(basic_block);
LLHD_STRUCT(inst);
LLHD_STRUCT(drive_inst);
LLHD_STRUCT(branch_inst);
LLHD_STRUCT(compare_inst);
LLHD_STRUCT(unary_inst);
LLHD_STRUCT(binary_inst);
LLHD_STRUCT(ret_inst);
LLHD_STRUCT(wait_inst);
LLHD_STRUCT(signal_inst);
LLHD_STRUCT(instance_inst);
LLHD_STRUCT(type);
LLHD_STRUCT(struct_field);

// LLHD_ENUM(compare_mode);
// LLHD_ENUM(unary_op);
// LLHD_ENUM(binary_op);
LLHD_ENUM(type_kind);

#undef LLHD_STRUCT
#undef LLHD_ENUM



/* -------------------------------------------------------------------------- */
/*                                                                            */
/*   TYPES                                                                    */
/*                                                                            */
/* -------------------------------------------------------------------------- */

typedef struct llhd_type llhd_type_t;
typedef enum llhd_type_kind llhd_type_kind_t;

enum llhd_type_kind {
	LLHD_VOID_TYPE,
	LLHD_LABEL_TYPE,
	LLHD_TIME_TYPE,
	LLHD_INT_TYPE,
	LLHD_LOGIC_TYPE,
	LLHD_STRUCT_TYPE,
	LLHD_ARRAY_TYPE,
	LLHD_PTR_TYPE,
};

LLHD_API llhd_type_t *llhd_type_make_void();
LLHD_API llhd_type_t *llhd_type_make_label();
LLHD_API llhd_type_t *llhd_type_make_time();
LLHD_API llhd_type_t *llhd_type_make_int(unsigned width);
LLHD_API llhd_type_t *llhd_type_make_logic(unsigned width);
LLHD_API llhd_type_t *llhd_type_make_struct(llhd_type_t **fields, unsigned num_fields);
LLHD_API llhd_type_t *llhd_type_make_array(llhd_type_t *element, unsigned length);
LLHD_API llhd_type_t *llhd_type_make_ptr(llhd_type_t *to);
LLHD_API llhd_type_t *llhd_type_copy(llhd_type_t *T);
LLHD_API void llhd_type_destroy(llhd_type_t *T);

LLHD_API int llhd_type_is(llhd_type_t*, llhd_type_kind_t);
LLHD_API int llhd_type_is_void(llhd_type_t*);
LLHD_API int llhd_type_is_label(llhd_type_t*);
LLHD_API int llhd_type_is_time(llhd_type_t*);
LLHD_API int llhd_type_is_int(llhd_type_t*);
LLHD_API int llhd_type_is_int_width(llhd_type_t*, unsigned);
LLHD_API int llhd_type_is_logic(llhd_type_t*);
LLHD_API int llhd_type_is_logic_width(llhd_type_t*, unsigned);
LLHD_API int llhd_type_is_struct(llhd_type_t*);
LLHD_API int llhd_type_is_array(llhd_type_t*);
LLHD_API int llhd_type_is_ptr(llhd_type_t*);

LLHD_API int llhd_type_equal(llhd_type_t*, llhd_type_t*);
LLHD_API void llhd_type_dump(llhd_type_t*, FILE*);
LLHD_API llhd_type_kind_t llhd_type_get_kind(llhd_type_t*);

// scalar accessors
LLHD_API unsigned llhd_type_scalar_get_width(llhd_type_t*);

// struct accessors
LLHD_API unsigned llhd_type_struct_get_num_fields(llhd_type_t*);
LLHD_API llhd_type_t **llhd_type_struct_get_fields(llhd_type_t*);
LLHD_API llhd_type_t *llhd_type_struct_get_field(llhd_type_t*, unsigned);

// array accessors
LLHD_API unsigned llhd_type_array_get_length(llhd_type_t*);
LLHD_API llhd_type_t *llhd_type_array_get_subtype(llhd_type_t*);

// ptr accessors
LLHD_API llhd_type_t *llhd_type_ptr_get_subtype(llhd_type_t*);



/* -------------------------------------------------------------------------- */
/*                                                                            */
/*   VALUES                                                                   */
/*                                                                            */
/* -------------------------------------------------------------------------- */

typedef struct llhd_value llhd_value_t;
typedef enum llhd_value_kind llhd_value_kind_t;

enum llhd_value_kind {
	LLHD_CONST_VALUE,
	LLHD_ENTITY_VALUE,
	LLHD_FUNC_VALUE,
	LLHD_PROC_VALUE,
	LLHD_BASIC_BLOCK_VALUE,
	LLHD_INST_VALUE,
};

LLHD_API llhd_value_t *llhd_value_copy(llhd_value_t*);
LLHD_API void llhd_value_destroy(void*);

LLHD_API int llhd_value_is(llhd_value_t*, llhd_value_kind_t);
LLHD_API int llhd_value_is_const(llhd_value_t*);
LLHD_API int llhd_value_is_entity(llhd_value_t*);
LLHD_API int llhd_value_is_func(llhd_value_t*);
LLHD_API int llhd_value_is_proc(llhd_value_t*);
LLHD_API int llhd_value_is_basic_block(llhd_value_t*);
LLHD_API int llhd_value_is_inst(llhd_value_t*);

LLHD_API int llhd_value_equal(llhd_value_t*, llhd_value_t*);
LLHD_API void llhd_value_dump(void*, FILE*);
LLHD_API void llhd_value_dump_name(void*, FILE*);
LLHD_API llhd_value_kind_t llhd_value_get_kind(llhd_value_t*);

LLHD_API void llhd_value_set_name(void*, const char*);
LLHD_API const char *llhd_value_get_name(void*);
LLHD_API llhd_type_t *llhd_value_get_type(llhd_value_t*);



/* -------------------------------------------------------------------------- */
/*                                                                            */
/*   CONSTANTS                                                                */
/*                                                                            */
/* -------------------------------------------------------------------------- */

typedef struct llhd_const       llhd_const_t;
typedef struct llhd_const_int   llhd_const_int_t;
typedef struct llhd_const_logic llhd_const_logic_t;
typedef struct llhd_const_time  llhd_const_time_t;

LLHD_API llhd_const_int_t   *llhd_make_const_int(unsigned width, const char *value);
LLHD_API llhd_const_logic_t *llhd_make_const_logic(unsigned width, const char *value);
LLHD_API llhd_const_time_t  *llhd_make_const_time(const char *value);



/* -------------------------------------------------------------------------- */
/*                                                                            */
/*   UNIT                                                                     */
/*                                                                            */
/* -------------------------------------------------------------------------- */

typedef struct llhd_unit        llhd_unit_t;
typedef struct llhd_arg         llhd_arg_t;
typedef struct llhd_func        llhd_func_t;
typedef struct llhd_proc        llhd_proc_t;
typedef struct llhd_entity      llhd_entity_t;
typedef struct llhd_basic_block llhd_basic_block_t;


LLHD_API llhd_module_t *llhd_unit_get_parent(llhd_unit_t*);
LLHD_API llhd_unit_t   *llhd_unit_get_next(llhd_unit_t*);
LLHD_API llhd_unit_t   *llhd_unit_get_prev(llhd_unit_t*);
LLHD_API void           llhd_unit_remove_from_parent(llhd_unit_t*);

LLHD_API void                llhd_unit_append_basic_block(llhd_unit_t*, llhd_basic_block_t*);
LLHD_API llhd_basic_block_t *llhd_unit_get_first_basic_block(llhd_unit_t*);
LLHD_API llhd_basic_block_t *llhd_unit_get_last_basic_block(llhd_unit_t*);


LLHD_API llhd_arg_t *llhd_make_arg(const char *name, llhd_type_t *type);
LLHD_API llhd_proc_t *llhd_make_proc(const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry);
LLHD_API llhd_func_t *llhd_make_func(const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry);
LLHD_API llhd_entity_t *llhd_make_entity(const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry);


LLHD_API llhd_basic_block_t *llhd_make_basic_block(const char *name);

LLHD_API llhd_unit_t *llhd_basic_block_get_parent(llhd_basic_block_t*);
LLHD_API llhd_basic_block_t *llhd_basic_block_get_next(llhd_basic_block_t*);
LLHD_API llhd_basic_block_t *llhd_basic_block_get_prev(llhd_basic_block_t*);
LLHD_API void llhd_basic_block_insert_before(llhd_basic_block_t *BB, llhd_basic_block_t *before);
LLHD_API void llhd_basic_block_insert_after(llhd_basic_block_t *BB, llhd_basic_block_t *after);

LLHD_API void llhd_basic_block_append_inst(llhd_basic_block_t*, llhd_inst_t*);
LLHD_API llhd_inst_t *llhd_basic_block_get_first_inst(llhd_basic_block_t*);
LLHD_API llhd_inst_t *llhd_basic_block_get_last_inst(llhd_basic_block_t*);



/* -------------------------------------------------------------------------- */
/*                                                                            */
/*   INSTRUCTIONS                                                             */
/*                                                                            */
/* -------------------------------------------------------------------------- */

typedef enum llhd_compare_mode llhd_compare_mode_t;
typedef enum llhd_unary_op     llhd_unary_op_t;
typedef enum llhd_binary_op    llhd_binary_op_t;

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

enum llhd_unary_op {
	LLHD_NOT,
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

llhd_drive_inst_t *llhd_make_drive_inst(llhd_value_t *target, llhd_value_t *value);
llhd_compare_inst_t *llhd_make_compare_inst(llhd_compare_mode_t mode, llhd_value_t *lhs, llhd_value_t *rhs);
llhd_branch_inst_t *llhd_make_conditional_branch_inst(llhd_value_t *cond, llhd_basic_block_t *dst1, llhd_basic_block_t *dst0);
llhd_branch_inst_t *llhd_make_unconditional_branch_inst(llhd_basic_block_t *dst);
llhd_unary_inst_t *llhd_make_unary_inst(llhd_unary_op_t op, llhd_value_t *arg);
llhd_binary_inst_t *llhd_make_binary_inst(llhd_binary_op_t op, llhd_value_t *lhs, llhd_value_t *rhs);
llhd_ret_inst_t *llhd_make_ret_inst(llhd_value_t **values, unsigned num_values);
llhd_wait_inst_t *llhd_make_wait_inst(llhd_value_t *duration);
llhd_signal_inst_t *llhd_make_signal_inst(llhd_type_t *type);
llhd_instance_inst_t *llhd_make_instance_inst(llhd_value_t *value, llhd_value_t **in, unsigned num_in, llhd_value_t **out, unsigned num_out);
