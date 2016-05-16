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

	llhd_type_t int32ty = llhd_type_new_int(32);
	llhd_type_t inputs[] = { int32ty, int32ty };
	llhd_type_t Ety = llhd_type_new_comp(inputs, 2, NULL, 0);
	llhd_type_unref(int32ty);
	llhd_value_t E = llhd_entity_new(Ety, "foo");
	llhd_type_unref(Ety);
	llhd_inst_append_to(Iadd,E);
	llhd_inst_append_to(Imul,E);
	llhd_value_unref(Iadd);
	llhd_value_unref(Imul);
	llhd_value_t Isub = llhd_inst_binary_new(LLHD_BINARY_SUB, Imul, llhd_unit_get_input(E,0), NULL);
	llhd_inst_append_to(Isub,E);
	llhd_value_unref(Isub);

	assert(llhd_value_get_name(E));
	assert(strcmp(llhd_value_get_name(E), "foo") == 0);
	assert(llhd_value_get_type(E) == Ety);
	assert(llhd_value_is(E, LLHD_VALUE_UNIT));
	assert(llhd_value_is(E, LLHD_UNIT_DEF_ENTITY));
	assert(llhd_unit_is_def(E));
	assert(!llhd_unit_is_decl(E));
	assert(llhd_entity_get_num_insts(E) == 3);
	assert(llhd_entity_get_first_inst(E) == Iadd);
	assert(llhd_entity_get_last_inst(E) == Isub);
	assert(llhd_value_get_name(Iadd) == NULL);
	assert(llhd_value_get_name(Imul));
	assert(strcmp(llhd_value_get_name(Imul), "tmp") == 0);
	assert(llhd_inst_prev(Iadd) == NULL);
	assert(llhd_inst_next(Iadd) == Imul);
	assert(llhd_inst_prev(Imul) == Iadd);
	assert(llhd_inst_next(Imul) == Isub);
	assert(llhd_inst_prev(Isub) == Imul);
	assert(llhd_inst_next(Isub) == NULL);

	llhd_asm_write_unit(E,stdout);
	printf("folding constants\n");
	llhd_fold_constants(E);
	llhd_asm_write_unit(E,stdout);

	llhd_value_unref(E);

	return 0;
}
