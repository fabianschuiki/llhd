// Copyright (c) 2016 Fabian Schuiki
#include <stddef.h>

typedef struct llhd_module * llhd_module_t;
typedef struct llhd_value * llhd_value_t;

extern const int LLHD_INST_BRANCH;

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
int llhd_unit_is_def(llhd_value_t);
int llhd_unit_is_decl(llhd_value_t);
llhd_value_t llhd_unit_get_first_block(llhd_value_t);
llhd_value_t llhd_unit_get_last_block(llhd_value_t);

llhd_value_t llhd_block_next(llhd_value_t);
llhd_value_t llhd_block_prev(llhd_value_t);
llhd_value_t llhd_block_get_first_inst(llhd_value_t);
llhd_value_t llhd_block_get_last_inst(llhd_value_t);
int llhd_block_is_entry(llhd_value_t);
int llhd_block_has_predecessors(llhd_value_t);
int llhd_block_has_successors(llhd_value_t);
void llhd_block_get_predecessors(llhd_value_t, llhd_value_t**, unsigned*);
void llhd_block_get_successors(llhd_value_t, llhd_value_t**, unsigned*);

llhd_value_t llhd_inst_next(llhd_value_t);
llhd_value_t llhd_inst_prev(llhd_value_t);
void llhd_inst_append_to(llhd_value_t,llhd_value_t);
void llhd_inst_prepend_to(llhd_value_t,llhd_value_t);
void llhd_inst_insert_after(llhd_value_t,llhd_value_t);
void llhd_inst_insert_before(llhd_value_t,llhd_value_t);
int llhd_inst_is(llhd_value_t,int);

llhd_value_t llhd_inst_branch_new_uncond(llhd_value_t);
llhd_value_t llhd_inst_branch_new_cond(llhd_value_t, llhd_value_t, llhd_value_t);
llhd_value_t llhd_inst_branch_get_condition(llhd_value_t);
llhd_value_t llhd_inst_branch_get_dst0(llhd_value_t);
llhd_value_t llhd_inst_branch_get_dst1(llhd_value_t);

int llhd_const_is_null(llhd_value_t);

int llhd_value_is_const(llhd_value_t);
const char *llhd_value_get_name(llhd_value_t);
void llhd_value_replace_uses(llhd_value_t,llhd_value_t);
void llhd_value_unlink_from_parent(llhd_value_t);
void llhd_value_unlink_uses(llhd_value_t);
void llhd_value_unlink(llhd_value_t);
void llhd_value_free(llhd_value_t);

void *llhd_alloc(size_t);
void *llhd_realloc(void*,size_t);
void llhd_free(void*);
