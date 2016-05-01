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

static bool
values_equal(llhd_value_t a, llhd_value_t b) {
	int k;

	if (!a || !b)
		return false;
	if (a == b)
		return true;

	k = llhd_value_get_kind(a);
	if (k != llhd_value_get_kind(b))
		return false;

	if (k == LLHD_VALUE_CONST) {
		int k2;

		k2 = llhd_const_get_kind(a);
		if (k2 != llhd_const_get_kind(b))
			return false;

		if (k2 == LLHD_CONST_INT) {
			return llhd_const_int_get_value(a) == llhd_const_int_get_value(b);
		}
	}

	else if (k == LLHD_VALUE_INST) {
		int k2;

		k2 = llhd_inst_get_kind(a);
		if (k2 != llhd_inst_get_kind(b))
			return false;

		if (k2 == LLHD_INST_UNARY) {
			return llhd_inst_unary_get_op(a) == llhd_inst_unary_get_op(b) &&
			       values_equal(llhd_inst_unary_get_arg(a), llhd_inst_unary_get_arg(b));
		}

		else if (k2 == LLHD_INST_BINARY) {
			return llhd_inst_binary_get_op(a) == llhd_inst_binary_get_op(b) &&
			       values_equal(llhd_inst_binary_get_lhs(a), llhd_inst_binary_get_lhs(b)) &&
			       values_equal(llhd_inst_binary_get_rhs(a), llhd_inst_binary_get_rhs(b));
		}
	}

	return false;
}

static unsigned
complexity(llhd_value_t V) {
	int k = llhd_value_get_kind(V);

	if (k == LLHD_VALUE_CONST) {
		return 1;
	}

	if (k == LLHD_VALUE_INST) {
		int k2 = llhd_inst_get_kind(V);

		if (k2 == LLHD_INST_UNARY) {
			return 1 + complexity(llhd_inst_unary_get_arg(V));
		}

		if (k2 == LLHD_INST_BINARY) {
			return 1 + complexity(llhd_inst_binary_get_lhs(V))
			         + complexity(llhd_inst_binary_get_rhs(V));
		}
	}

	return 2;
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

		if (llhd_value_is(arg, LLHD_VALUE_INST)) {

			/* Apply DeMorgan's rule */
			if (op == LLHD_UNARY_NOT && llhd_inst_is(arg, LLHD_INST_BINARY)) {
				int opi = llhd_inst_binary_get_op(arg), opn = opi;

				if (opi == LLHD_BINARY_AND)
					opn = LLHD_BINARY_OR;
				else if (opi == LLHD_BINARY_OR)
					opn = LLHD_BINARY_AND;

				if (opn != opi) {
					llhd_value_t tmp, I, lhs, rhs;

					tmp = llhd_inst_unary_new(LLHD_UNARY_NOT, llhd_inst_binary_get_lhs(arg), NULL);
					lhs = simplify(tmp);
					llhd_value_unref(tmp);

					tmp = llhd_inst_unary_new(LLHD_UNARY_NOT, llhd_inst_binary_get_rhs(arg), NULL);
					rhs = simplify(tmp);
					llhd_value_unref(tmp);

					tmp = llhd_inst_binary_new(opn, lhs, rhs, llhd_value_get_name(V));
					llhd_value_unref(lhs);
					llhd_value_unref(rhs);
					I = simplify(tmp);
					llhd_value_unref(tmp);
					return I;
				}
			}

			else if (op == LLHD_UNARY_NOT && llhd_inst_is(arg, LLHD_INST_UNARY) && llhd_inst_unary_get_op(arg) == LLHD_UNARY_NOT) {
				llhd_value_t argi = llhd_inst_unary_get_arg(arg);
				llhd_value_ref(argi);
				llhd_value_unref(arg);
				return argi;
			}
		}

		if (arg != llhd_inst_unary_get_arg(V)) {
			llhd_value_t I = llhd_inst_unary_new(op, arg, llhd_value_get_name(V));
			llhd_value_unref(arg);
			return I;
		}

	} else if (kind == LLHD_INST_BINARY) {
		int op = llhd_inst_binary_get_op(V);
		llhd_value_t lhs, rhs, not_lhs = NULL, not_rhs = NULL, k = NULL, nk = NULL;
		llhd_value_t extract_binop = NULL, extract_factor = NULL;

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

			if (values_equal(lhs, rhs)) {
				llhd_value_unref(rhs);
				return lhs;
			}

			if ((values_equal(not_lhs, rhs) && !not_rhs) || (values_equal(not_rhs, lhs) && !not_lhs)) {
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

			if (values_equal(lhs, rhs)) {
				llhd_value_unref(rhs);
				return lhs;
			}

			if ((values_equal(not_lhs, rhs) && !not_rhs) || (values_equal(not_rhs, lhs) && !not_lhs)) {
				llhd_value_unref(lhs);
				llhd_value_unref(rhs);
				return llhd_const_int_new(1);
			}
		}


		if (op == LLHD_BINARY_AND) {
			if (llhd_value_is(lhs, LLHD_VALUE_INST) && llhd_inst_is(lhs, LLHD_INST_BINARY) && llhd_inst_binary_get_op(lhs) == LLHD_BINARY_OR) {
				extract_binop = lhs;
				extract_factor = rhs;
			} else if (llhd_value_is(rhs, LLHD_VALUE_INST) && llhd_inst_is(rhs, LLHD_INST_BINARY) && llhd_inst_binary_get_op(rhs) == LLHD_BINARY_OR) {
				extract_binop = rhs;
				extract_factor = lhs;
			}
		}

		if (extract_binop && extract_factor) {
			llhd_value_t lhs_new, rhs_new, tmp, I;

			lhs_new = llhd_inst_binary_new(LLHD_BINARY_AND, llhd_inst_binary_get_lhs(extract_binop), extract_factor, llhd_value_get_name(V));
			rhs_new = llhd_inst_binary_new(LLHD_BINARY_AND, llhd_inst_binary_get_rhs(extract_binop), extract_factor, llhd_value_get_name(V));

			tmp = llhd_inst_binary_new(LLHD_BINARY_OR, lhs_new, rhs_new, llhd_value_get_name(extract_binop));
			llhd_value_unref(lhs_new);
			llhd_value_unref(rhs_new);
			I = simplify(tmp);
			llhd_value_unref(tmp);

			llhd_value_unref(extract_binop);
			llhd_value_unref(extract_factor);

			return I;
		}

		if (llhd_value_is(lhs, LLHD_VALUE_INST) && llhd_inst_is(lhs, LLHD_INST_BINARY) && (!llhd_value_is(rhs, LLHD_VALUE_INST) || !llhd_inst_is(rhs, LLHD_INST_BINARY))) {
			llhd_value_t tmp = lhs;
			lhs = rhs;
			rhs = tmp;
		}

		if (llhd_value_is(lhs, LLHD_VALUE_INST) && llhd_inst_is(lhs, LLHD_INST_BINARY) && llhd_inst_binary_get_op(lhs) == op) {
			llhd_value_t Il, Ir;
			Ir = llhd_inst_binary_new(op, llhd_inst_binary_get_rhs(lhs), rhs, NULL);
			Il = llhd_inst_binary_get_lhs(lhs);
			llhd_value_ref(Il);
			llhd_value_unref(lhs);
			llhd_value_unref(rhs);
			lhs = Il;
			rhs = Ir;
		}

		if (op == LLHD_BINARY_OR) {
			llhd_value_t tmp, lnot, rnot, ltry, rtry;
			unsigned kl, kr, ktryl, ktryr;

			kl = complexity(lhs);
			kr = complexity(rhs);

			lnot = llhd_inst_unary_new(LLHD_UNARY_NOT, lhs, NULL);
			tmp = llhd_inst_binary_new(LLHD_BINARY_AND, lnot, rhs, NULL);
			llhd_value_unref(lnot);
			ltry = simplify(tmp);
			llhd_value_unref(tmp);

			rnot = llhd_inst_unary_new(LLHD_UNARY_NOT, rhs, NULL);
			tmp = llhd_inst_binary_new(LLHD_BINARY_AND, rnot, lhs, NULL);
			llhd_value_unref(rnot);
			rtry = simplify(tmp);
			llhd_value_unref(tmp);

			ktryl = complexity(ltry);
			ktryr = complexity(rtry);

			if (ktryl >= kl) {
				llhd_value_unref(ltry);
				ltry = NULL;
			}
			if (ktryr >= kr) {
				llhd_value_unref(rtry);
				rtry = NULL;
			}

			if (ltry && rtry) {
				if (ktryl > ktryr) {
					llhd_value_unref(ltry);
					ltry = NULL;
				} else {
					llhd_value_unref(rtry);
					rtry = NULL;
				}
			}

			if (ltry) {
				llhd_value_unref(lhs);
				lhs = ltry;
			} else if (rtry) {
				llhd_value_unref(rhs);
				rhs = rtry;
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
		printf("signal %s (%p)\n", llhd_value_get_name(recbase->sig), recbase->sig);
		while (rec != recend && rec->sig == recbase->sig) {
			llhd_value_t BB;
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

		printf("- pre-simplify\n  "); dump_bool_ops(cond, stdout); printf(" [k=%u]\n", complexity(cond));
		tmp = simplify(cond);
		llhd_value_unref(cond);
		cond = tmp;
		printf("- post-simplify\n  "); dump_bool_ops(cond, stdout); printf(" [k=%u]\n", complexity(cond));

		if (llhd_value_is(cond, LLHD_VALUE_CONST) && llhd_const_int_get_value(cond) == 1) {
			printf("- combinatorial\n");
		} else {
			llhd_value_t ncond;
			printf("- sequential\n");
			ncond = llhd_inst_unary_new(LLHD_UNARY_NOT, cond, NULL);
			rec = recbase;
			while (rec != recend && rec->sig == recbase->sig) {
				llhd_value_t drive_cond;
				tmp = llhd_inst_binary_new(LLHD_BINARY_OR, rec->cond, ncond, NULL);
				printf("- drive %p condition pre-simplify\n  ", rec->inst); dump_bool_ops(tmp, stdout); printf(" [k=%u]\n", complexity(tmp));
				drive_cond = simplify(tmp);
				llhd_value_unref(tmp);
				printf("- drive %p condition post-simplify\n  ", rec->inst); dump_bool_ops(drive_cond, stdout); printf(" [k=%u]\n", complexity(drive_cond));
				llhd_value_unref(drive_cond);
				++rec;
			}
			llhd_value_unref(ncond);
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
