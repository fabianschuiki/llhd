// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

void llhd_asm_write_unit(llhd_value_t,FILE*);

int main() {
	llhd_value_t Na = llhd_const_int_new(123);
	llhd_value_t Nb = llhd_const_int_new(42);
	llhd_value_t Nc = llhd_const_int_new(21);
	llhd_value_t Iadd = llhd_inst_binary_new(LLHD_BINARY_ADD, Na, Nb, NULL);
	llhd_value_t Imul = llhd_inst_binary_new(LLHD_BINARY_MUL, Nc, Iadd, "tmp");
	llhd_value_unref(Na);
	llhd_value_unref(Nb);
	llhd_value_unref(Nc);

	llhd_type_t Ety = llhd_type_new_comp(NULL, 0, NULL, 0);
	llhd_value_t E = llhd_entity_new(Ety, "foo");
	llhd_inst_append_to(Iadd,E);
	llhd_inst_append_to(Imul,E);
	llhd_type_unref(Ety);
	llhd_value_unref(Iadd);
	llhd_value_unref(Imul);

	assert(llhd_value_get_name(E));
	assert(strcmp(llhd_value_get_name(E), "foo") == 0);
	assert(llhd_value_get_type(E) == Ety);
	assert(llhd_value_is(E, LLHD_VALUE_UNIT));
	assert(llhd_value_get_kind(E) == LLHD_VALUE_UNIT);
	assert(llhd_unit_is(E, LLHD_UNIT_DEF_ENTITY));
	assert(llhd_unit_get_kind(E) == LLHD_UNIT_DEF_ENTITY);
	assert(llhd_unit_is_def(E));
	assert(!llhd_unit_is_decl(E));
	assert(llhd_entity_get_num_insts(E) == 2);
	assert(llhd_entity_get_first_inst(E) == Iadd);
	assert(llhd_entity_get_last_inst(E) == Imul);
	assert(llhd_value_get_name(Iadd) == NULL);
	assert(llhd_value_get_name(Imul));
	assert(strcmp(llhd_value_get_name(Imul), "tmp") == 0);
	assert(llhd_inst_prev(Iadd) == NULL);
	assert(llhd_inst_next(Iadd) == Imul);
	assert(llhd_inst_prev(Imul) == Iadd);
	assert(llhd_inst_next(Imul) == NULL);

	llhd_asm_write_unit(E,stdout);
	printf("folding constants\n");
	llhd_fold_constants(E);
	llhd_asm_write_unit(E,stdout);

	llhd_value_unref(E);

	return 0;
}
