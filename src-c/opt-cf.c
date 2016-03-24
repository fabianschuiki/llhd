// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>

typedef struct llhd_apint * llhd_apint_t;

bool llhd_value_is_const_int(llhd_value_t);
llhd_apint_t llhd_const_int_get_value(llhd_value_t);
llhd_value_t llhd_const_int_new(llhd_apint_t);

llhd_apint_t llhd_apint_not(const llhd_apint_t);
llhd_apint_t llhd_apint_add(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_sub(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_mul(const llhd_apint_t, const llhd_apint_t, bool);
llhd_apint_t llhd_apint_div(const llhd_apint_t, const llhd_apint_t, bool);
llhd_apint_t llhd_apint_rem(const llhd_apint_t, const llhd_apint_t, bool);
llhd_apint_t llhd_apint_lsl(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_lsr(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_asr(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_and(const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_or (const llhd_apint_t, const llhd_apint_t);
llhd_apint_t llhd_apint_xor(const llhd_apint_t, const llhd_apint_t);

// -------------------------------------------------------------------------- //

static llhd_apint_t
fold_unary_inst_int (int op, llhd_apint_t arg) {
	switch (op) {
		case LLHD_UNARY_NOT: return llhd_apint_not(arg); break;
		default:
			return NULL;
	}
}

static llhd_apint_t
fold_binary_inst_int (int op, llhd_apint_t lhs, llhd_apint_t rhs) {
	switch (op) {
		case LLHD_BINARY_ADD:  return llhd_apint_add(lhs,rhs); break;
		case LLHD_BINARY_SUB:  return llhd_apint_sub(lhs,rhs); break;
		case LLHD_BINARY_UMUL: return llhd_apint_mul(lhs,rhs,false); break;
		case LLHD_BINARY_UDIV: return llhd_apint_div(lhs,rhs,false); break;
		case LLHD_BINARY_UREM: return llhd_apint_rem(lhs,rhs,false); break;
		case LLHD_BINARY_SMUL: return llhd_apint_mul(lhs,rhs,true); break;
		case LLHD_BINARY_SDIV: return llhd_apint_div(lhs,rhs,true); break;
		case LLHD_BINARY_SREM: return llhd_apint_rem(lhs,rhs,true); break;
		case LLHD_BINARY_LSL:  return llhd_apint_lsl(lhs,rhs); break;
		case LLHD_BINARY_LSR:  return llhd_apint_lsr(lhs,rhs); break;
		case LLHD_BINARY_ASR:  return llhd_apint_asr(lhs,rhs); break;
		case LLHD_BINARY_AND:  return llhd_apint_and(lhs,rhs); break;
		case LLHD_BINARY_OR:   return llhd_apint_or(lhs,rhs); break;
		case LLHD_BINARY_XOR:  return llhd_apint_xor(lhs,rhs); break;
		default:
			return NULL;
	}
}

static void
fold_unary_inst (llhd_value_t I) {
	llhd_value_t arg = llhd_inst_unary_get_arg(I);

	// const int folding
	if (llhd_value_is_const_int(arg)) {
		llhd_apint_t arg_value = llhd_const_int_get_value(arg);
		llhd_apint_t result = fold_unary_inst_int(
			llhd_inst_unary_get_op(I),
			arg_value
		);
		if (result) {
			llhd_value_t C = llhd_const_int_new(result);
			llhd_value_replace_uses(I,C);
			llhd_value_unlink(I);
			llhd_value_free(I);
		}
	}
}

static void
fold_binary_inst (llhd_value_t I) {
	llhd_value_t lhs = llhd_inst_binary_get_lhs(I);
	llhd_value_t rhs = llhd_inst_binary_get_rhs(I);

	// const int folding
	if (llhd_value_is_const_int(lhs) && llhd_value_is_const_int(rhs)) {
		llhd_apint_t lhs_value = llhd_const_int_get_value(lhs);
		llhd_apint_t rhs_value = llhd_const_int_get_value(rhs);
		llhd_apint_t result = fold_binary_inst_int(
			llhd_inst_binary_get_op(I),
			lhs_value,
			rhs_value
		);
		if (result) {
			llhd_value_t C = llhd_const_int_new(result);
			llhd_value_replace_uses(I,C);
			llhd_value_unlink(I);
			llhd_value_free(I);
		}
	}
}

static void
fold_inst (llhd_value_t I) {
	int kind = llhd_inst_get_kind(I);
	switch (kind) {
		case LLHD_INST_UNARY:  fold_unary_inst(I); break;
		case LLHD_INST_BINARY: fold_binary_inst(I); break;
		default:
			assert(0 && "unsupported inst kind");
	}
}

void
llhd_fold_constants (llhd_value_t value) {
	int kind = llhd_value_get_kind(value);
	switch (kind) {
		case LLHD_VALUE_INST: fold_inst(value); break;
		default:
			assert(0 && "unsupported value kind");
	}
}
