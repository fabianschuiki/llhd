// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

llhd_value_t llhd_const_int_new(uint64_t);
llhd_value_t llhd_inst_binary_new(int,llhd_value_t,llhd_value_t,const char*);

llhd_type_t llhd_type_new_comp(const llhd_type_t*,unsigned,const llhd_type_t*,unsigned);
llhd_value_t llhd_entity_new(llhd_type_t,const char*);

int main() {
	llhd_value_t Na = llhd_const_int_new(123);
	llhd_value_t Nb = llhd_const_int_new(42);
	llhd_value_t Iadd = llhd_inst_binary_new(LLHD_BINARY_ADD, Na, Nb, "");
	llhd_value_unref(Na);
	llhd_value_unref(Nb);

	llhd_type_t Ety = llhd_type_new_comp(NULL, 0, NULL, 0);
	llhd_value_t E = llhd_entity_new(Ety, "foo");
	llhd_inst_append_to(Iadd,E);
	llhd_type_unref(Ety);
	llhd_value_unref(Iadd);

	assert(llhd_value_get_name(E));
	assert(strcmp(llhd_value_get_name(E), "foo") == 0);
	assert(llhd_value_get_type(E) == Ety);
	assert(llhd_value_is(E, LLHD_VALUE_UNIT));
	assert(llhd_value_get_kind(E) == LLHD_VALUE_UNIT);
	// assert(llhd_unit_is(E, LLHD_UNIT_DEF_ENTITY));
	// assert(llhd_unit_get_kind(E) == LLHD_UNIT_DEF_ENTITY);
	// assert(llhd_unit_is_def(E));
	// assert(!llhd_unit_is_decl(E));
	// assert(llhd_entity_get_num_insts(E) == 1);
	// assert(llhd_entity_get_first_inst(E) == Iadd);
	// assert(llhd_entity_get_last_inst(E) == Iadd);

	llhd_value_unref(E);

	return 0;
}
