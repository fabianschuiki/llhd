// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include "util.h"
#include "value.h"
#include "boolexpr.h"
#include <stdlib.h>
#include <assert.h>
#include <string.h>

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
//
// Left to do:
// 1) Determine whether sequential or combinatorial. Ignore if combinatorial.
// 2) Find drive condition for individual values.
// 3) Create function that calculates next value and store strobe.
// 4) Create entity with process' signature, call above function and instantiate
//    storage instructions based on the function's return values.
// 5) Create process without the sequential output signals and migrate all basic
//    blocks. Be sure to replace the parameters. Remove the drive instructions
//    for the sequential signals.
// 6) Instantiate new process in entity and replace uses of old process with new
//    entity.


struct record {
	llhd_value_t sig;
	llhd_value_t inst;
	struct llhd_boolexpr *cond_expr;
};

struct signal_record {
	llhd_value_t sig;
	struct record *records;
	unsigned num_records;
	struct llhd_boolexpr *cond;
};


static struct llhd_boolexpr *get_boolexpr(llhd_value_t V);


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

static struct llhd_boolexpr *
get_block_condition(llhd_value_t BB) {
	struct llhd_list *pos;
	struct llhd_boolexpr *result = NULL;

	/*
	 * 1) get predecessors
	 * 2) figure out if true or false branch
	 * 3) get branch condition, negate if false branch
	 */
	// printf("calculating block condition of %s %p\n", llhd_value_get_name(BB), BB);

	for (pos = BB->users.next; pos != &BB->users; pos = pos->next) {
		struct llhd_value_use *use;
		llhd_value_t V;
		struct llhd_boolexpr *precond, *cond;

		use = llhd_container_of(pos, use, link);
		V = llhd_inst_branch_get_condition(use->user);
		precond = get_block_condition(llhd_inst_get_parent(use->user));
		// printf("- user %p (arg %d)\n", use->user, use->arg);

		if (V) {
			// printf("  merging branch condition with predecessor condition\n");
			cond = get_boolexpr(V); /// TODO: make use of get_boolexpr here
			if (use->arg == 2) {
				// V = llhd_inst_unary_new(LLHD_UNARY_NOT, V, NULL);
				llhd_boolexpr_negate(cond);
			// } else {
				// llhd_value_ref(V);
			}
			// I = llhd_inst_binary_new(LLHD_BINARY_AND, V, precond, NULL);
			// llhd_value_unref(V);
			// llhd_value_unref(precond);
			// V = I;
			cond = llhd_boolexpr_new_and((struct llhd_boolexpr*[]){cond,precond}, 2);
		} else {
			// printf("  unconditional branch, reusing predecessor condition\n");
			cond = precond;
		}

		if (result) {
			// I = llhd_inst_binary_new(LLHD_BINARY_OR, result, V, NULL);
			// llhd_value_unref(result);
			// llhd_value_unref(V);
			// result = I;
			result = llhd_boolexpr_new_or((struct llhd_boolexpr*[]){result,cond}, 2);
		} else {
			result = cond;
		}
	}

	// return result ? result : llhd_const_int_new(1);
	return result ? result : llhd_boolexpr_new_const_1();
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

		// if (op == LLHD_BINARY_OR) {
		// 	llhd_value_t tmp, lnot, rnot, ltry, rtry;
		// 	unsigned kl, kr, ktryl, ktryr;

		// 	kl = complexity(lhs);
		// 	kr = complexity(rhs);

		// 	lnot = llhd_inst_unary_new(LLHD_UNARY_NOT, lhs, NULL);
		// 	tmp = llhd_inst_binary_new(LLHD_BINARY_AND, lnot, rhs, NULL);
		// 	llhd_value_unref(lnot);
		// 	ltry = simplify(tmp);
		// 	llhd_value_unref(tmp);

		// 	rnot = llhd_inst_unary_new(LLHD_UNARY_NOT, rhs, NULL);
		// 	tmp = llhd_inst_binary_new(LLHD_BINARY_AND, rnot, lhs, NULL);
		// 	llhd_value_unref(rnot);
		// 	rtry = simplify(tmp);
		// 	llhd_value_unref(tmp);

		// 	ktryl = complexity(ltry);
		// 	ktryr = complexity(rtry);

		// 	if (ktryl >= kl) {
		// 		llhd_value_unref(ltry);
		// 		ltry = NULL;
		// 	}
		// 	if (ktryr >= kr) {
		// 		llhd_value_unref(rtry);
		// 		rtry = NULL;
		// 	}

		// 	if (ltry && rtry) {
		// 		if (ktryl > ktryr) {
		// 			llhd_value_unref(ltry);
		// 			ltry = NULL;
		// 		} else {
		// 			llhd_value_unref(rtry);
		// 			rtry = NULL;
		// 		}
		// 	}

		// 	if (ltry) {
		// 		llhd_value_unref(lhs);
		// 		lhs = ltry;
		// 	} else if (rtry) {
		// 		llhd_value_unref(rhs);
		// 		rhs = rtry;
		// 	}
		// }

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

static struct llhd_boolexpr *
get_boolexpr(llhd_value_t V) {
	int k = llhd_value_get_kind(V);

	if (k == LLHD_VALUE_CONST) {
		int k2 = llhd_const_get_kind(V);
		if (k2 == LLHD_CONST_INT) {
			if (llhd_const_int_get_value(V) == 0)
				return llhd_boolexpr_new_const_0();
			if (llhd_const_int_get_value(V) == 1)
				return llhd_boolexpr_new_const_1();
		}
	}

	else if (k == LLHD_VALUE_INST) {
		int k2 = llhd_inst_get_kind(V);

		if (k2 == LLHD_INST_UNARY) {
			int op = llhd_inst_unary_get_op(V);
			if (op == LLHD_UNARY_NOT) {
				struct llhd_boolexpr *expr = get_boolexpr(llhd_inst_unary_get_arg(V));
				llhd_boolexpr_negate(expr);
				return expr;
			}
		}

		else if (k2 == LLHD_INST_BINARY) {
			int op = llhd_inst_binary_get_op(V);
			struct llhd_boolexpr *args[2] = {
				get_boolexpr(llhd_inst_binary_get_lhs(V)),
				get_boolexpr(llhd_inst_binary_get_rhs(V)),
			};

			if (op == LLHD_BINARY_AND)
				return llhd_boolexpr_new_and(args,2);
			if (op == LLHD_BINARY_OR)
				return llhd_boolexpr_new_or(args,2);

			llhd_boolexpr_free(args[0]);
			llhd_boolexpr_free(args[1]);
		}
	}

	return llhd_boolexpr_new_symbol(V);
}


static void
gather_dependencies(llhd_value_t V, struct llhd_ptrset *deps) {
	unsigned num, i;
	printf("gather_dependencies(%p, %d deps)\n", V, deps->num);

	if (!llhd_ptrset_insert(deps, V)) {
		return;
	}

	if (!llhd_value_is(V, LLHD_VALUE_INST)) {
		return;
	}

	num = llhd_inst_get_num_params(V);
	for (i = 0; i < num; ++i) {
		gather_dependencies(llhd_inst_get_param(V,i), deps);
	}
}


static void
gather_boolexpr_dependencies(struct llhd_boolexpr *expr, struct llhd_ptrset *deps) {
	if (llhd_boolexpr_is(expr, LLHD_BOOLEXPR_SYMBOL)) {
		gather_dependencies(llhd_boolexpr_get_symbol(expr), deps);
	} else {
		unsigned i, num = llhd_boolexpr_get_num_children(expr);
		struct llhd_boolexpr **children = llhd_boolexpr_get_children(expr);
		for (i = 0; i < num; ++i)
			gather_boolexpr_dependencies(children[i], deps);
	}
}


void
llhd_desequentialize(llhd_value_t proc) {
	llhd_value_t BB;
	llhd_list_t blocks, pos;
	unsigned num_records;
	unsigned num_signal_records;
	struct llhd_buffer records;
	struct record *recbase, *rec, *recend;
	struct signal_record *signal_records, *sigrec, *sigrecend;

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

	// Allocate one record for each driven signal.
	num_signal_records = 0;
	rec = records.data;
	recend = rec+num_records;
	while (rec != recend) {
		struct record *prev = rec;
		++num_signal_records;
		while (rec != recend && rec->sig == prev->sig)
			++rec;
	}
	signal_records = llhd_alloc(num_signal_records * sizeof(struct signal_record));

	rec = records.data;
	recend = rec+num_records;
	sigrec = signal_records;
	num_signal_records = 0;
	while (rec != recend) {
		memset(sigrec, 0, sizeof(*sigrec));
		sigrec->sig = rec->sig;
		sigrec->records = rec;

		recbase = rec;
		printf("signal %s (%p)\n", llhd_value_get_name(recbase->sig), recbase->sig);
		while (rec != recend && rec->sig == recbase->sig) {
			++sigrec->num_records;
			rec->cond_expr = get_block_condition(llhd_inst_get_parent(rec->inst));
			if (sigrec->cond) {
				sigrec->cond = llhd_boolexpr_new_or((struct llhd_boolexpr *[]){sigrec->cond, llhd_boolexpr_copy(rec->cond_expr)}, 2);
			} else {
				sigrec->cond = llhd_boolexpr_copy(rec->cond_expr);
			}
			++rec;
		}

		printf("- driven if ");
		llhd_boolexpr_write(sigrec->cond, NULL, stdout);
		llhd_boolexpr_disjunctive_cnf(&sigrec->cond);
		printf(" = ");
		llhd_boolexpr_write(sigrec->cond, NULL, stdout);
		printf("\n");

		if (!llhd_boolexpr_is(sigrec->cond, LLHD_BOOLEXPR_CONST_1)) {
			printf("- sequential\n");
			++num_signal_records;
			rec = recbase;
			while (rec != recend && rec->sig == recbase->sig) {
				struct llhd_boolexpr *ncond = llhd_boolexpr_copy(sigrec->cond);
				llhd_boolexpr_negate(ncond);
				rec->cond_expr = llhd_boolexpr_new_or((struct llhd_boolexpr*[]){rec->cond_expr, ncond}, 2);
				printf("- drive %p if ", rec->inst);
				llhd_boolexpr_write(rec->cond_expr, NULL, stdout);
				llhd_boolexpr_disjunctive_cnf(&rec->cond_expr);
				printf(" = ");
				llhd_boolexpr_write(rec->cond_expr, NULL, stdout);
				printf("\n");
				++rec;
			}
			++sigrec;
		} else {
			llhd_boolexpr_free(sigrec->cond);
		}
	}

	printf("%d sequential signals\n", num_signal_records);

	struct llhd_ptrset used;
	llhd_ptrset_init(&used, 16);
	sigrec = signal_records;
	sigrecend = sigrec + num_signal_records;
	while (sigrec != sigrecend) {
		rec = sigrec->records;
		recend = rec + sigrec->num_records;
		gather_boolexpr_dependencies(sigrec->cond, &used);
		while (rec != recend) {
			gather_boolexpr_dependencies(rec->cond_expr, &used);
			gather_dependencies(llhd_inst_drive_get_val(rec->inst), &used);
			++rec;
		}
		++sigrec;
	}
	printf("used.num = %d\n", used.num);



	// Assemble a function that calculates the drive condition and driven value
	// for each sequential signal.
	unsigned i, n, num_inputs, num_outputs;
	llhd_type_t *inputs, *outputs, proc_type, i1ty, func_type;
	llhd_value_t func;

	i1ty = llhd_type_new_int(1);
	proc_type = llhd_value_get_type(proc);
	num_inputs = llhd_type_get_num_inputs(proc_type);
	num_outputs = 2*num_signal_records;
	inputs = llhd_zalloc(num_inputs * sizeof(llhd_type_t));
	outputs = llhd_zalloc(num_outputs * sizeof(llhd_type_t));

	for (i = 0, n = 0; i < num_inputs; ++i) {
		llhd_value_t P = llhd_unit_get_input(proc, i);
		if (llhd_ptrset_has(&used, P)) {
			inputs[n] = llhd_type_get_input(proc_type, i);
			printf("uses input %s %p\n", llhd_value_get_name(P), P);
			++n;
		} else {
			printf("ignores input %s %p\n", llhd_value_get_name(P), P);
		}
	}
	num_inputs = n;

	for (i = 0; i < num_signal_records; ++i) {
		sigrec = signal_records + i;
		outputs[i*2+0] = i1ty;
		outputs[i*2+1] = llhd_value_get_type(sigrec->sig);
	}

	func_type = llhd_type_new_func(inputs, num_inputs, outputs, num_outputs);
	func = llhd_func_new(func_type, llhd_value_get_name(proc));
	llhd_type_unref(func_type);
	llhd_type_unref(i1ty);

	// do stuff with it
	printf("func_type = ");
	llhd_asm_write_type(func_type, stdout);
	printf("\n");
	llhd_asm_write_unit(func, stdout);

	/// @todo: Verify that the created function and entity are self-contained.
	///        That is, they should only refer to globals and values embedded
	///        within themselves. This catches bugs where certain instructions
	///        have not been unlinked from the old process properly.

	llhd_value_unref(func);
	llhd_ptrset_dispose(&used);

	// Clean up.
	sigrec = signal_records;
	sigrecend = signal_records + num_signal_records;
	while (sigrec != sigrecend) {
		llhd_boolexpr_free(sigrec->cond);
		++sigrec;
	}

	rec = records.data;
	while (rec != recend) {
		llhd_boolexpr_free(rec->cond_expr);
		++rec;
	}

	llhd_free(signal_records);
	llhd_buffer_dispose(&records);
}
