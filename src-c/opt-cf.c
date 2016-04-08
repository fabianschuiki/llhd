// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>

bool llhd_value_is_const_int(llhd_value_t);
// llhd_apint_t llhd_const_int_get_value(llhd_value_t);
// llhd_value_t llhd_const_int_new(llhd_apint_t);

llhd_apint_t llhd_apint_not(llhd_apint_t);
llhd_apint_t llhd_apint_add(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_sub(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_mul(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_div(llhd_apint_t,llhd_apint_t,bool);
llhd_apint_t llhd_apint_rem(llhd_apint_t,llhd_apint_t,bool);
llhd_apint_t llhd_apint_lsl(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_lsr(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_asr(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_and(llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_or (llhd_apint_t,llhd_apint_t);
llhd_apint_t llhd_apint_xor(llhd_apint_t,llhd_apint_t);

// -------------------------------------------------------------------------- //

static bool
fold_unary_inst_int(int op, llhd_apint_t arg, llhd_apint_t *result) {
	switch (op) {
		case LLHD_UNARY_NOT: *result = llhd_apint_not(arg); return true;
		default:
			return false;
	}
}

static bool
fold_binary_inst_int (int op, llhd_apint_t lhs, llhd_apint_t rhs, llhd_apint_t *result) {
	switch (op) {
		case LLHD_BINARY_ADD:  *result = llhd_apint_add(lhs,rhs); return true;
		case LLHD_BINARY_SUB:  *result = llhd_apint_sub(lhs,rhs); return true;
		case LLHD_BINARY_MUL:  *result = llhd_apint_mul(lhs,rhs); return true;
		case LLHD_BINARY_UDIV: *result = llhd_apint_div(lhs,rhs,false); return true;
		case LLHD_BINARY_UREM: *result = llhd_apint_rem(lhs,rhs,false); return true;
		case LLHD_BINARY_SDIV: *result = llhd_apint_div(lhs,rhs,true); return true;
		case LLHD_BINARY_SREM: *result = llhd_apint_rem(lhs,rhs,true); return true;
		case LLHD_BINARY_LSL:  *result = llhd_apint_lsl(lhs,rhs); return true;
		case LLHD_BINARY_LSR:  *result = llhd_apint_lsr(lhs,rhs); return true;
		case LLHD_BINARY_ASR:  *result = llhd_apint_asr(lhs,rhs); return true;
		case LLHD_BINARY_AND:  *result = llhd_apint_and(lhs,rhs); return true;
		case LLHD_BINARY_OR:   *result = llhd_apint_or(lhs,rhs); return true;
		case LLHD_BINARY_XOR:  *result = llhd_apint_xor(lhs,rhs); return true;
		default:
			return false;
	}
}

// static void
// fold_unary_inst (llhd_value_t I) {
// 	llhd_value_t arg = llhd_inst_unary_get_arg(I);

// 	// const int folding
// 	if (llhd_value_is(arg, LLHD_VALUE_CONST)) {
// 		if (llhd_const_is(arg, LLHD_CONST_INT)) {
// 			llhd_apint_t arg_value = llhd_const_int_get_value(arg);
// 			llhd_apint_t result;
// 			bool changed = fold_unary_inst_int(
// 				llhd_inst_unary_get_op(I),
// 				arg_value,
// 				&result
// 			);
// 			if (changed) {
// 				llhd_value_t C = llhd_const_int_new(result);
// 				llhd_value_replace_uses(I,C);
// 				llhd_value_unref(C);
// 				llhd_value_unlink(I);
// 				// llhd_value_unref(I);
// 			}
// 		}
// 	}
// }

static void
fold_binary_inst (llhd_value_t I) {
	llhd_value_t lhs = llhd_inst_binary_get_lhs(I);
	llhd_value_t rhs = llhd_inst_binary_get_rhs(I);

	// const int folding
	if (llhd_value_is(lhs, LLHD_VALUE_CONST) && llhd_value_is(rhs, LLHD_VALUE_CONST)) {
		if (llhd_const_is(lhs, LLHD_CONST_INT) && llhd_const_is(rhs, LLHD_CONST_INT)) {
			llhd_apint_t lhs_value = llhd_const_int_get_value(lhs);
			llhd_apint_t rhs_value = llhd_const_int_get_value(rhs);
			llhd_apint_t result;
			bool changed = fold_binary_inst_int(
				llhd_inst_binary_get_op(I),
				lhs_value,
				rhs_value,
				&result
			);
			if (changed) {
				llhd_value_t C = llhd_const_int_new(result);
				llhd_value_replace_uses(I,C);
				llhd_value_unref(C);
				llhd_value_unlink(I);
				// llhd_value_unref(I);
			}
		}
	}
}

static void
fold_inst (llhd_value_t I) {
	int kind = llhd_inst_get_kind(I);
	switch (kind) {
		// case LLHD_INST_UNARY:  fold_unary_inst(I); break;
		case LLHD_INST_BINARY: fold_binary_inst(I); break;
		default:
			assert(0 && "unsupported inst kind");
	}
}

static void
fold_unit(llhd_value_t U) {
	llhd_value_t I, In;
	int kind = llhd_unit_get_kind(U);
	switch (kind) {
		case LLHD_UNIT_DEF_ENTITY:
			for (I = llhd_entity_get_first_inst(U); I; I = In) {
				In = llhd_inst_next(I);
				fold_inst(I);
			}
			break;
		default:
			assert(0 && "unsupported unit kind");
	}
}

void
llhd_fold_constants (llhd_value_t value) {
	int kind = llhd_value_get_kind(value);
	switch (kind) {
		case LLHD_VALUE_INST: fold_inst(value); break;
		case LLHD_VALUE_UNIT: fold_unit(value); break;
		default:
			assert(0 && "unsupported value kind");
	}
}
