// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include "util.h"
#include "value.h"
#include <stdlib.h>
#include <assert.h>

// Algorithm:
// 1) Iterate over all basic blocks. For each drive instruction, calculate the
//    conditions along the control path that lead to the instruction, and gather
//    them in an array.
// 2) Sort the array by the driven signal.
// 3) Iterate over the array, merging subsequent entries for the same signal by
//    combining the conditions with an OR and minimizing the resulting
//    expression.
// 4) If the condition evaluates to 1, remove the signal from the array and
//    skip to the next.
// 5) For every drive instruction of the signal, factor out the minimized
//    condition found above. In the combinatorial process, create a new branch
//    based on the resulting condition, insert the instruction tree to calculate
//    the driven value. Add the corresponding inputs and outputs to the process.
//    In the replacement entity, insert the instructions to calculate the
//    minimized storage condition, and add a register instruction based on that
//    storage condition and the driven value established above.
//    Remove the original drive instructions from their parent.
// 6) Instantiate the combinatorial process in the replacement entity.
// 7) Optimize the combinatorial process and the replacement entity.
// 8) Replace all uses of the original process with the replacement entity,
//    which has an identical interface.

struct record {
	llhd_value_t sig;
	llhd_value_t inst;
	llhd_value_t cond;
};

static int
compare_records(const void *pa, const void *pb) {
	const struct record *a = pa, *b = pb;
	if (a->sig < b->sig) return -1;
	if (a->sig > b->sig) return 1;
	return 0;
}

static void
dump_records(struct record *records, unsigned num_records) {
	unsigned i;
	for (i = 0; i < num_records; ++i) {
		printf("{ sig = %p, inst = %p }\n", records->sig, records->inst);
		++records;
	}
}

static llhd_value_t
get_block_condition(llhd_value_t BB) {
	struct llhd_list *pos;
	llhd_value_t result = NULL;

	/*
	 * 1) get predecessors
	 * 2) figure out if true or false branch
	 * 3) get branch condition, negate if false branch
	 */
	// printf("calculating block condition of %s %p\n", llhd_value_get_name(BB), BB);

	for (pos = BB->users.next; pos != &BB->users; pos = pos->next) {
		struct llhd_value_use *use;
		llhd_value_t cond, precond, I;

		use = llhd_container_of(pos, use, link);
		cond = llhd_inst_branch_get_condition(use->user);
		precond = get_block_condition(llhd_inst_get_parent(use->user));
		// printf("- user %p (arg %d)\n", use->user, use->arg);

		if (cond) {
			// printf("  merging branch condition with predecessor condition\n");
			if (use->arg == 2) {
				cond = llhd_inst_unary_new(LLHD_UNARY_NOT, cond, NULL);
			} else {
				llhd_value_ref(cond);
			}
			I = llhd_inst_binary_new(LLHD_BINARY_AND, cond, precond, NULL);
			llhd_value_unref(cond);
			llhd_value_unref(precond);
			cond = I;
		} else {
			// printf("  unconditional branch, reusing predecessor condition\n");
			cond = precond;
		}

		if (result) {
			I = llhd_inst_binary_new(LLHD_BINARY_OR, result, cond, NULL);
			llhd_value_unref(result);
			llhd_value_unref(cond);
			result = I;
		} else {
			result = cond;
		}
	}

	return result ? result : llhd_const_int_new(1);
}

static void
dump_bool_ops(llhd_value_t V, FILE *out) {
	int kind, kind2, op;
	size_t x = (size_t)V;

	kind = llhd_value_get_kind(V);
	if (kind == LLHD_VALUE_INST) {
		kind2 = llhd_inst_get_kind(V);
		if (kind2 == LLHD_INST_UNARY) {
			op = llhd_inst_unary_get_op(V);
			if (op == LLHD_UNARY_NOT) {
				fputc('~', out);
				dump_bool_ops(llhd_inst_unary_get_arg(V), out);
			} else {
				goto bail;
			}
		} else if (kind2 == LLHD_INST_BINARY) {
			op = llhd_inst_binary_get_op(V);
			if (op == LLHD_BINARY_AND) {
				fputs("(", out);
				dump_bool_ops(llhd_inst_binary_get_lhs(V), out);
				fputs(" && ", out);
				dump_bool_ops(llhd_inst_binary_get_rhs(V), out);
				fputs(")", out);
			} else if (op == LLHD_BINARY_OR) {
				fputs("(", out);
				dump_bool_ops(llhd_inst_binary_get_lhs(V), out);
				fputs(" || ", out);
				dump_bool_ops(llhd_inst_binary_get_rhs(V), out);
				fputs(")", out);
			} else {
				goto bail;
			}
		} else {
			goto bail;
		}
	} else if (kind == LLHD_VALUE_CONST) {
		char *str = llhd_const_to_string(V);
		fputs(str, out);
		llhd_free(str);
	} else {
		goto bail;
	}
	return;
bail:
	x = (x ^ (x >> 32)) & 0xFFFFFFFF;
	x = (x ^ (x >> 16)) & 0xFFFF;
	fprintf(out, "$%zx", x);
}

static llhd_value_t
simplify(llhd_value_t V) {
	int kind;

	if (!llhd_value_is(V, LLHD_VALUE_INST))
		goto bail;
	kind = llhd_inst_get_kind(V);

	if (kind == LLHD_INST_UNARY) {
		int op = llhd_inst_unary_get_op(V);
		llhd_value_t arg;

		arg = simplify(llhd_inst_unary_get_arg(V));

		if (arg != llhd_inst_unary_get_arg(V)) {
			llhd_value_t I = llhd_inst_unary_new(op, arg, llhd_value_get_name(V));
			llhd_value_unref(arg);
			return I;
		}

	} else if (kind == LLHD_INST_BINARY) {
		int op = llhd_inst_binary_get_op(V);
		llhd_value_t lhs, rhs, not_lhs = NULL, not_rhs = NULL, k = NULL, nk = NULL;

		lhs = simplify(llhd_inst_binary_get_lhs(V));
		rhs = simplify(llhd_inst_binary_get_rhs(V));

		if (llhd_value_is(lhs, LLHD_VALUE_CONST)) {
			k = lhs;
			nk = rhs;
		} else if (llhd_value_is(rhs, LLHD_VALUE_CONST)) {
			k = rhs;
			nk = lhs;
		}

		if (llhd_value_is(lhs, LLHD_VALUE_INST) && llhd_inst_is(lhs, LLHD_INST_UNARY) && llhd_inst_unary_get_op(lhs) == LLHD_UNARY_NOT) {
			not_lhs = llhd_inst_unary_get_arg(lhs);
		}
		if (llhd_value_is(rhs, LLHD_VALUE_INST) && llhd_inst_is(rhs, LLHD_INST_UNARY) && llhd_inst_unary_get_op(rhs) == LLHD_UNARY_NOT) {
			not_rhs = llhd_inst_unary_get_arg(rhs);
		}

		if (op == LLHD_BINARY_AND) {
			if (k) {
				if (llhd_const_int_get_value(k) == 0) {
					llhd_value_unref(nk);
					return k;
				} else if (llhd_const_int_get_value(k) == 1) {
					llhd_value_unref(k);
					return nk;
				}
			}

			if (lhs == rhs) {
				llhd_value_unref(rhs);
				return lhs;
			}

			if ((not_lhs == rhs && !not_rhs) || (not_rhs == lhs && !not_lhs)) {
				llhd_value_unref(lhs);
				llhd_value_unref(rhs);
				return llhd_const_int_new(0);
			}
		}

		if (op == LLHD_BINARY_OR) {
			if (k) {
				if (llhd_const_int_get_value(k) == 1) {
					llhd_value_unref(nk);
					return k;
				} else if (llhd_const_int_get_value(k) == 0) {
					llhd_value_unref(k);
					return nk;
				}
			}

			if (lhs == rhs) {
				llhd_value_unref(rhs);
				return lhs;
			}

			if ((not_lhs == rhs && !not_rhs) || (not_rhs == lhs && !not_lhs)) {
				llhd_value_unref(lhs);
				llhd_value_unref(rhs);
				return llhd_const_int_new(1);
			}
		}

		if (lhs != llhd_inst_binary_get_lhs(V) || rhs != llhd_inst_binary_get_rhs(V)) {
			llhd_value_t I = llhd_inst_binary_new(op, lhs, rhs, llhd_value_get_name(V));
			llhd_value_unref(lhs);
			llhd_value_unref(rhs);
			return I;
		}
	}

bail:
	llhd_value_ref(V);
	return V;
}

void llhd_desequentialize(llhd_value_t proc) {
	llhd_value_t BB;
	llhd_list_t blocks, pos;
	unsigned num_records;
	struct llhd_buffer records;
	struct record *recbase, *rec, *recend;

	llhd_buffer_init(&records, 16*sizeof(struct record));
	num_records = 0;

	blocks = llhd_unit_get_blocks(proc);
	pos = llhd_block_first(blocks);
	while ((BB = llhd_block_next(blocks, &pos))) {
		llhd_value_t I;
		printf("processing block %s %p\n", llhd_value_get_name(BB), BB);

		for (I = llhd_block_get_first_inst(BB); I; I = llhd_inst_next(I)) {
			if (llhd_inst_is(I, LLHD_INST_DRIVE)) {
				struct record rec = {
					.sig = llhd_inst_drive_get_sig(I),
					.inst = I,
				};
				llhd_buffer_append(&records, sizeof(struct record), &rec);
				++num_records;
			}
		}
	}

	qsort(records.data, num_records, sizeof(struct record), compare_records);
	dump_records(records.data, num_records);

	rec = records.data;
	recend = rec+num_records;
	while (rec != recend) {
		llhd_value_t cond = NULL, tmp;

		recbase = rec;
		printf("inspecting signal %p\n", recbase->sig);
		while (rec != recend && rec->sig == recbase->sig) {
			llhd_value_t BB;
			printf("- inst %p\n", rec->inst);
			BB = llhd_inst_get_parent(rec->inst);
			rec->cond = get_block_condition(BB);
			if (cond) {
				llhd_value_t I = llhd_inst_binary_new(LLHD_BINARY_OR, cond, rec->cond, NULL);
				llhd_value_unref(cond);
				cond = I;
			} else {
				llhd_value_ref(rec->cond);
				cond = rec->cond;
			}
			++rec;
		}

		printf("pre-simplify: ");
		dump_bool_ops(cond, stdout);
		printf("\n");
		tmp = simplify(cond);
		llhd_value_unref(cond);
		cond = tmp;
		printf("post-simplify: ");
		dump_bool_ops(cond, stdout);
		printf("\n");

		if (llhd_value_is(cond, LLHD_VALUE_CONST) && llhd_const_int_get_value(cond) == 1) {
			printf("signal %p is always driven\n", recbase->sig);
		} else {
			printf("signal %p describes a storage element\n", recbase->sig);
		}

		llhd_value_unref(cond);
		rec = recbase;
		while (rec != recend && rec->sig == recbase->sig) {
			llhd_value_unref(rec->cond);
			rec->cond = NULL;
			++rec;
		}
	}

	llhd_buffer_free(&records);
}
