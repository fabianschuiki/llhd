// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdint.h>
#include <assert.h>
#include <stdio.h>


static void
const_fold(llhd_value_t V) {
	if (llhd_value_is(V, LLHD_INST_BINARY)) {
		llhd_value_t lhs = llhd_inst_binary_get_lhs(V);
		llhd_value_t rhs = llhd_inst_binary_get_rhs(V);
		if (llhd_value_is(lhs, LLHD_CONST_INT) && llhd_value_is(rhs, LLHD_CONST_INT)) {
			unsigned bits = llhd_type_get_length(llhd_value_get_type(lhs));
			uint64_t lhs_value = llhd_const_int_get_value(lhs);
			uint64_t rhs_value = llhd_const_int_get_value(rhs);
			uint64_t result;
			switch (llhd_inst_binary_get_op(V)) {
				case LLHD_BINARY_ADD: result = lhs_value + rhs_value; break;
				case LLHD_BINARY_SUB: result = lhs_value - rhs_value; break;
				default: return;
			}
			llhd_value_t C = llhd_const_int_new(bits, result);
			llhd_value_replace_uses(V,C);
			llhd_value_unref(C);
		}
	}
}

int main() {
	llhd_value_t Na = llhd_const_int_new(32, 123);
	llhd_value_t Nb = llhd_const_int_new(32, 42);
	assert(Na);
	assert(Nb);
	llhd_value_t Iadd = llhd_inst_binary_new(LLHD_BINARY_ADD, Na, Nb, "");
	llhd_value_t Iadd2 = llhd_inst_binary_new(LLHD_BINARY_ADD, Na, Iadd, "");
	assert(Iadd);
	assert(Iadd2);

	assert(llhd_value_is(Na, LLHD_VALUE_CONST));
	assert(llhd_value_is(Nb, LLHD_VALUE_CONST));
	assert(llhd_value_is(Na, LLHD_CONST_INT));
	assert(llhd_value_is(Nb, LLHD_CONST_INT));

	assert(llhd_value_is(Iadd, LLHD_VALUE_INST));
	assert(llhd_value_is(Iadd, LLHD_INST_BINARY));
	assert(llhd_value_is(Iadd, LLHD_BINARY_ADD));

	assert(llhd_value_has_users(Na));
	assert(llhd_value_has_users(Nb));
	assert(llhd_value_has_users(Iadd));
	assert(llhd_value_get_num_users(Na) == 2);
	assert(llhd_value_get_num_users(Nb) == 1);
	assert(llhd_value_get_num_users(Iadd) == 1);
	assert(llhd_value_get_num_users(Iadd2) == 0);

	llhd_value_unref(Iadd);
	const_fold(Iadd);

	llhd_value_t Iadd2_2 = llhd_inst_binary_get_rhs(Iadd2);
	assert(llhd_value_is(Iadd2_2, LLHD_VALUE_CONST));
	assert(llhd_value_is(Iadd2_2, LLHD_CONST_INT));
	assert(llhd_const_int_get_value(Iadd2_2) == 165);

	llhd_value_unref(Na);
	llhd_value_unref(Nb);
	llhd_value_unref(Iadd2);

	return 0;
}
