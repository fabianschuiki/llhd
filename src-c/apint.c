// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>

llhd_apint_t
llhd_apint_not(llhd_apint_t arg) {
	return ~arg;
}

llhd_apint_t
llhd_apint_add(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs+rhs;
}

llhd_apint_t
llhd_apint_sub(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs-rhs;
}

llhd_apint_t
llhd_apint_mul(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs*rhs;
}

llhd_apint_t
llhd_apint_div(llhd_apint_t lhs, llhd_apint_t rhs, bool signd) {
	if (signd)
		return (int64_t)lhs/(int64_t)rhs;
	else
		return lhs/rhs;
}

llhd_apint_t
llhd_apint_rem(llhd_apint_t lhs, llhd_apint_t rhs, bool signd) {
	if (signd)
		return (int64_t)lhs%(int64_t)rhs;
	else
		return lhs%rhs;
}

llhd_apint_t
llhd_apint_lsl(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs << rhs;
}

llhd_apint_t
llhd_apint_lsr(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs >> rhs;
}

llhd_apint_t
llhd_apint_asr(llhd_apint_t lhs, llhd_apint_t rhs) {
	return (int64_t)lhs >> rhs;
}

llhd_apint_t
llhd_apint_and(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs&rhs;
}

llhd_apint_t
llhd_apint_or (llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs|rhs;
}

llhd_apint_t
llhd_apint_xor(llhd_apint_t lhs, llhd_apint_t rhs) {
	return lhs^rhs;
}

