// Copyright (c) 2016 Fabian Schuiki
typedef struct llhd_module * llhd_module_t;
typedef struct llhd_value * llhd_value_t;

llhd_module_t llhd_new_module(const char *name);
void llhd_free_module(llhd_module_t);
llhd_value_t llhd_module_get_first_unit(llhd_module_t);
llhd_value_t llhd_module_get_last_unit(llhd_module_t);
void llhd_verify_module_selfcontained(llhd_module_t);

llhd_value_t llhd_unit_next(llhd_value_t);
llhd_value_t llhd_unit_prev(llhd_value_t);
void llhd_unit_append_to(llhd_value_t,llhd_module_t);
void llhd_unit_prepend_to(llhd_value_t,llhd_module_t);
void llhd_unit_insert_after(llhd_value_t,llhd_value_t);
void llhd_unit_insert_before(llhd_value_t,llhd_value_t);
int llhd_unit_is_def(llhd_value_t);
int llhd_unit_is_decl(llhd_value_t);

const char *llhd_value_get_name(llhd_value_t);
void llhd_value_replace_uses(llhd_value_t,llhd_value_t);
void llhd_value_unlink(llhd_value_t);
void llhd_value_free(llhd_value_t);
