// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include "util.h"
#include "value.h"
#include "boolexpr.h"
#include <stdlib.h>
#include <assert.h>
#include <string.h>

/**
 * @file
 *
 * This file implements the desequentialization algorithm which finds sequential
 * constructs in a process and extracts them into explicit storage instructions.
 * The algorithm works as follows:
 *
 * 1. Make a list of drive instructions and find the condition under which each
 *    of them is executed.
 * 2. Make a list of signals and find the condition under which each of them is
 *    driven. A signal's condition can be found by iterating over all
 *    instructions that drive it and OR'ing together the individual drive
 *    conditions determined in (1). Signals whose drive condition is the
 *    constant 1 are combinatorial signals, all others are sequential signals.
 * 3. Find the instruction graph required to calculate the drive condition and
 *    driven value of every sequential signal.
 * 4. Create a separate function that contains the instruction graph determined
 *    in (3). For every sequential signal the function produces two outputs: An
 *    i1 strobe that is 1 whenever the signal should change its state, and the
 *    value the signal should assume.
 * 5. Remove the drive instructions for sequential signals and simplify the
 *    process such that its inputs and outputs only contain what is necessary to
 *    produce the combinatorial signals.
 * 6. Create an entity with the same type as the original process. Call the
 *    function created in (4) and instantiated the simplified process created in
 *    (5). For every sequential signal, create a register and drive instruction.
 *    Replace all uses of the original process with the entity. Remove the
 *    original process.
 *
 * @author Fabian Schuiki <fabian@schuiki.ch>
 *
 * @todo Implement the desequentialization algorithm for signals that have
 * multiple possible driven values. This requires a piece of code to be
 * generated that selects between these values. Such code would require either
 * branches and variables, branches and a phi instruction, or a mux instruction.
 */


/**
 * A record to keep information associated with one drive instruction around.
 */
struct record {
	llhd_value_t sig;
	llhd_value_t inst;
	struct llhd_boolexpr *cond;
	int is_sequential;
};

/**
 * A record to keep information associated with one driven signal around.
 * Spans multiple record instances, one for each driven value.
 */
struct signal_record {
	llhd_value_t sig;
	struct record *records;
	unsigned num_records;
	struct llhd_boolexpr *cond;
};


static struct llhd_boolexpr *get_boolexpr(llhd_value_t V);
static llhd_value_t build_boolexpr_binary(int, struct llhd_boolexpr*, llhd_value_t, struct llhd_ptrmap*);


static int
compare_records(const void *pa, const void *pb) {
	const struct record *a = pa, *b = pb;
	if (a->sig < b->sig) return -1;
	if (a->sig > b->sig) return 1;
	return 0;
}


/**
 * Find the boolean expression that describes under what condition control flow
 * arrives at a basic block.
 *
 * @param BB  The basic block whose condition for execution shall be determined.
 *
 * @return A newly allocated llhd_boolexpr that represents the chain of branches
 *         and control flow changes that lead to the basic block @a BB. The
 *         expression is neither in its canonical form nor simplified.
 */
static struct llhd_boolexpr *
get_block_condition(llhd_value_t BB) {
	struct llhd_list *pos;
	struct llhd_boolexpr *result = NULL;

	for (pos = BB->users.next; pos != &BB->users; pos = pos->next) {
		struct llhd_value_use *use;
		llhd_value_t V;
		struct llhd_boolexpr *precond, *cond;

		use = llhd_container_of(pos, use, link);
		V = llhd_inst_branch_get_condition(use->user);
		precond = get_block_condition(llhd_inst_get_parent(use->user));

		if (V) {
			cond = get_boolexpr(V);
			if (use->arg == 2) {
				llhd_boolexpr_negate(cond);
			}
			cond = llhd_boolexpr_new_and((struct llhd_boolexpr*[]){cond,precond}, 2);
		} else {
			cond = precond;
		}

		if (result) {
			result = llhd_boolexpr_new_or((struct llhd_boolexpr*[]){result,cond}, 2);
		} else {
			result = cond;
		}
	}

	return result ? result : llhd_boolexpr_new_const_1();
}


/**
 * Convert a graph of instructions into an equivalent llhd_boolexpr. Integer
 * constants, binary AND and OR, and unary NOT instructions are translated; all
 * other values are wrapped in opaque boolexpr symbols. The resulting expression
 * is easy to mutate and simplify, and can then be converted back to a tree of
 * instructions and values using the build_boolexpr() function.
 *
 * @param V  The value to be translated into a llhd_boolexpr.
 *
 * @return A newly allocated llhd_boolexpr that is equivalent to @a V, but is
 *         neither in its canonical form nor simplified.
 */
static struct llhd_boolexpr *
get_boolexpr(llhd_value_t V) {
	int k = llhd_value_get_kind(V);

	if (LLHD_ISA(k, LLHD_CONST_INT)) {
		if (llhd_const_int_get_value(V) == 0)
			return llhd_boolexpr_new_const_0();
		if (llhd_const_int_get_value(V) == 1)
			return llhd_boolexpr_new_const_1();
	}

	else if (LLHD_ISA(k, LLHD_INST_UNARY)) {
		int op = llhd_inst_unary_get_op(V);
		if (op == LLHD_UNARY_NOT) {
			struct llhd_boolexpr *expr = get_boolexpr(llhd_inst_unary_get_arg(V));
			llhd_boolexpr_negate(expr);
			return expr;
		}
	}

	else if (LLHD_ISA(k, LLHD_INST_BINARY)) {
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

	return llhd_boolexpr_new_symbol(V);
}


/**
 * Collects a value and all its dependencies into a pointer set. This function
 * is designed to gather a full set of values and instructions that are required
 * to arrive at the value @a V.
 *
 * @param V     The value whose dependencies shall be gathered.
 * @param deps  The llhd_ptrset into which the dependencies are inserted.
 */
static void
gather_dependencies(llhd_value_t V, struct llhd_ptrset *deps) {
	assert(V && deps);

	if (!llhd_ptrset_insert(deps, V)) {
		return;
	}

	if (llhd_value_is(V, LLHD_VALUE_INST)) {
		unsigned num = llhd_inst_get_num_params(V), i;
		for (i = 0; i < num; ++i) {
			gather_dependencies(llhd_inst_get_param(V,i), deps);
		}
	}
}


/**
 * From a boolean expression, collects all values and their dependencies into a
 * pointer set. Every symbol in @a expr is visited and passed to
 * gather_dependencies(). The result is the full set of values and instructions
 * that are required to arrive at the result of the expression @a expr.
 *
 * @param expr  The expression whose symbols' dependencies shall be gathered.
 * @param deps  The llhd_ptrset into which the dependencies are inserted.
 */
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


static llhd_value_t
migrate(llhd_value_t V, llhd_value_t dst, struct llhd_ptrmap *migrated) {
	llhd_value_t *slot, *migrated_params = NULL;
	unsigned i, num_params;

	slot = (llhd_value_t*)llhd_ptrmap_expand(migrated, V);
	if (*slot) {
		llhd_value_ref(*slot);
		return *slot;
	}

	if (!llhd_value_is(V, LLHD_VALUE_INST)) {
		*slot = V;
		llhd_value_ref(V);
		return V;
	}

	num_params = llhd_inst_get_num_params(V);
	migrated_params = llhd_zalloc(num_params * sizeof(llhd_value_t));
	for (i = 0; i < num_params; ++i) {
		migrated_params[i] = migrate(llhd_inst_get_param(V,i), dst, migrated);
	}

	*slot = llhd_value_copy(V);
	llhd_inst_append_to(*slot, dst);

	for (i = 0; i < num_params; ++i) {
		llhd_value_substitute(*slot, llhd_inst_get_param(V,i), migrated_params[i]);
		llhd_value_unref(migrated_params[i]);
	}
	llhd_free(migrated_params);

	return *slot;
}


static llhd_value_t
build_boolexpr(struct llhd_boolexpr *expr, llhd_value_t dst, struct llhd_ptrmap *migrated) {
	enum llhd_boolexpr_kind kind = llhd_boolexpr_get_kind(expr);
	assert(expr && dst);
	switch (kind) {
		case LLHD_BOOLEXPR_CONST_0: return llhd_const_int_new(1,0);
		case LLHD_BOOLEXPR_CONST_1: return llhd_const_int_new(1,1);
		case LLHD_BOOLEXPR_SYMBOL: return migrate(llhd_boolexpr_get_symbol(expr), dst, migrated);
		case LLHD_BOOLEXPR_OR: return build_boolexpr_binary(LLHD_BINARY_OR, expr, dst, migrated);
		case LLHD_BOOLEXPR_AND: return build_boolexpr_binary(LLHD_BINARY_AND, expr, dst, migrated);
	}
	assert(0 && "should not arrive here");
	return NULL;
}


static llhd_value_t
build_boolexpr_binary(int kind, struct llhd_boolexpr *expr, llhd_value_t dst, struct llhd_ptrmap *migrated) {
	unsigned i,num;
	llhd_value_t V, Vp, Vn;
	struct llhd_boolexpr **children;

	num = llhd_boolexpr_get_num_children(expr);
	children = llhd_boolexpr_get_children(expr);
	Vp = NULL;
	for (i = 0; i < num; ++i) {
		V = build_boolexpr(children[i], dst, migrated);
		if (Vp) {
			Vn = llhd_inst_binary_new(kind, Vp, V, "");
			llhd_value_unref(Vp);
			llhd_value_unref(V);
			Vp = Vn;
		} else {
			Vp = V;
		}
	}
	return Vp;
}


/**
 * Split a process into a combinatorial and a sequential part.
 *
 * @param proc  The process to be desequentialized. Will be invalidated during
 *              the execution of the function and replaced by an equivalent
 *              entity.
 *
 * @postcond @a proc is empty
 */
void
llhd_desequentialize(llhd_value_t proc) {
	llhd_value_t BB;
	llhd_list_t blocks, pos;
	unsigned num_records, num;
	unsigned num_signal_records;
	struct llhd_buffer records;
	struct record *recbase, *rec, *recend;
	struct signal_record *signal_records, *sigrec, *sigrecend;
	llhd_type_t new_proc_type;
	llhd_value_t new_proc, *input_values, *output_values;

	llhd_buffer_init(&records, 16*sizeof(struct record));
	num_records = 0;

	blocks = llhd_unit_get_blocks(proc);
	pos = llhd_block_first(blocks);
	while ((BB = llhd_block_next(blocks, &pos))) {
		llhd_value_t I;
		for (I = llhd_block_get_first_inst(BB); I; I = llhd_inst_next(I)) {
			if (llhd_value_is(I, LLHD_INST_DRIVE)) {
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
		while (rec != recend && rec->sig == recbase->sig) {
			++sigrec->num_records;
			rec->cond = get_block_condition(llhd_inst_get_parent(rec->inst));
			if (sigrec->cond) {
				sigrec->cond = llhd_boolexpr_new_or((struct llhd_boolexpr *[]){sigrec->cond, llhd_boolexpr_copy(rec->cond)}, 2);
			} else {
				sigrec->cond = llhd_boolexpr_copy(rec->cond);
			}
			++rec;
		}

		llhd_boolexpr_disjunctive_cnf(&sigrec->cond);

		if (!llhd_boolexpr_is(sigrec->cond, LLHD_BOOLEXPR_CONST_1)) {
			++num_signal_records;
			rec = recbase;
			while (rec != recend && rec->sig == recbase->sig) {
				struct llhd_boolexpr *ncond = llhd_boolexpr_copy(sigrec->cond);
				rec->is_sequential = 1;
				llhd_boolexpr_negate(ncond);
				rec->cond = llhd_boolexpr_new_or((struct llhd_boolexpr*[]){rec->cond, ncond}, 2);
				llhd_boolexpr_disjunctive_cnf(&rec->cond);
				++rec;
			}
			++sigrec;
		} else {
			llhd_boolexpr_free(sigrec->cond);
		}
	}

	/// @todo If num_signal_records (the number of sequential signals) is 0,
	///       abort the algorithm and clean up all of the above.

	printf("%d sequential signals\n", num_signal_records);


	// Find the set of instructions and values required to determine the driven
	// values and drive conditions of each sequential signal.
	struct llhd_ptrset used;
	llhd_ptrset_init(&used, 16);
	sigrec = signal_records;
	sigrecend = sigrec + num_signal_records;
	while (sigrec != sigrecend) {
		rec = sigrec->records;
		recend = rec + sigrec->num_records;
		gather_boolexpr_dependencies(sigrec->cond, &used);
		while (rec != recend) {
			gather_boolexpr_dependencies(rec->cond, &used);
			gather_dependencies(llhd_inst_drive_get_val(rec->inst), &used);
			++rec;
		}
		++sigrec;
	}


	// Assemble a function that calculates the drive condition and driven value
	// for each sequential signal.
	unsigned i, n, num_inputs, num_outputs, *input_mapping, *output_mapping;
	llhd_type_t *inputs, *outputs, proc_type, i1ty, func_type;
	llhd_value_t func, *func_returns, I, entity, call;
	const char *name;
	char *name_buffer;

	i1ty = llhd_type_new_int(1);
	proc_type = llhd_value_get_type(proc);
	num_inputs = llhd_type_get_num_inputs(proc_type);
	num_outputs = 2*num_signal_records;
	inputs = llhd_zalloc(num_inputs * sizeof(llhd_type_t));
	outputs = llhd_zalloc(num_outputs * sizeof(llhd_type_t));
	func_returns = llhd_zalloc(num_outputs * sizeof(llhd_value_t));
	input_mapping = llhd_zalloc(num_inputs * sizeof(unsigned));

	for (i = 0, n = 0; i < num_inputs; ++i) {
		llhd_value_t P = llhd_unit_get_input(proc, i);
		if (llhd_ptrset_has(&used, P)) {
			inputs[n] = llhd_type_get_input(proc_type, i);
			input_mapping[n] = i;
			++n;
		}
	}
	num_inputs = n;
	llhd_ptrset_dispose(&used);

	for (i = 0; i < num_signal_records; ++i) {
		sigrec = signal_records + i;
		outputs[i*2+0] = i1ty;
		outputs[i*2+1] = llhd_type_get_subtype(llhd_value_get_type(sigrec->sig));
	}

	name = llhd_value_get_name(proc);
	name_buffer = llhd_alloc(strlen(name) + 6);

	entity = llhd_entity_new(proc_type, name);
	n = llhd_unit_get_num_inputs(proc);
	for (i = 0; i < n; ++i) {
		llhd_value_set_name(llhd_unit_get_input(entity,i), llhd_value_get_name(llhd_unit_get_input(proc,i)));
	}
	n = llhd_unit_get_num_outputs(proc);
	for (i = 0; i < n; ++i) {
		llhd_value_set_name(llhd_unit_get_output(entity,i), llhd_value_get_name(llhd_unit_get_output(proc,i)));
	}
	llhd_value_replace_uses(proc, entity);
	llhd_unit_insert_before(entity, proc);

	strcpy(name_buffer, name);
	strcat(name_buffer, "_seq");
	func_type = llhd_type_new_func(inputs, num_inputs, outputs, num_outputs);
	func = llhd_func_new(func_type, name_buffer);
	llhd_unit_insert_after(func, proc);
	llhd_type_unref(func_type);
	llhd_type_unref(i1ty);
	llhd_free(inputs);
	llhd_free(outputs);

	strcpy(name_buffer, name);
	strcat(name_buffer, "_comb");
	llhd_value_set_name(proc, name_buffer);

	llhd_free(name_buffer);

	input_values = llhd_zalloc(num_inputs * sizeof(llhd_value_t));
	for (i = 0; i < num_inputs; ++i) {
		input_values[i] = llhd_unit_get_input(entity, input_mapping[i]);
	}
	call = llhd_inst_call_new(func, input_values, num_inputs, NULL);
	llhd_inst_append_to(call, entity);
	llhd_value_unref(call);
	llhd_free(input_values);

	for (i = 0; i < num_signal_records; ++i) {
		const char *signame;
		char *buffer;
		unsigned signamelen;
		llhd_value_t ret_cond, ret_val;

		sigrec = signal_records + i;

		signame = llhd_value_get_name(sigrec->sig);
		signamelen = signame ? strlen(signame) : 0;
		buffer = llhd_alloc(signamelen + 6);

		strcpy(buffer, signame);
		strcat(buffer, "_strb");
		llhd_value_set_name(llhd_unit_get_output(func, i*2+0), buffer);
		ret_cond = llhd_inst_extract_new(call, i*2+0, buffer);

		strcpy(buffer, signame);
		strcat(buffer, "_val");
		llhd_value_set_name(llhd_unit_get_output(func, i*2+1), buffer);
		ret_val = llhd_inst_extract_new(call, i*2+1, buffer);

		llhd_free(buffer);

		llhd_inst_append_to(ret_cond, entity);
		llhd_inst_append_to(ret_val, entity);
		llhd_value_unref(ret_cond);
		llhd_value_unref(ret_val);

		I = llhd_inst_reg_new(ret_val, ret_cond, signame);
		llhd_inst_append_to(I, entity);
		llhd_value_unref(I);

		num = llhd_unit_get_num_outputs(proc);
		for (n = 0; n < num; ++n) {
			if (sigrec->sig == llhd_unit_get_output(proc,n)) {
				I = llhd_inst_drive_new(llhd_unit_get_output(entity,n), I);
				llhd_inst_append_to(I, entity);
				llhd_value_unref(I);
				break;
			}
		}
	}


	// Create a basic block with the instructions required to calculate the
	// drive condition as well as the driven value for every sequential signal.
	struct llhd_ptrmap migrated;
	llhd_value_t func_block;

	func_block = llhd_block_new("entry");
	llhd_block_append_to(func_block, func);
	llhd_value_unref(func_block);

	llhd_ptrmap_init(&migrated, 16);
	for (i = 0; i < num_signal_records; ++i) {
		sigrec = signal_records+i;
		func_returns[2*i+0] = build_boolexpr(sigrec->cond, func_block, &migrated);

		if (sigrec->num_records == 1) {
			func_returns[2*i+1] = migrate(llhd_inst_drive_get_val(sigrec->records->inst), func_block, &migrated);
		} else {
			assert(0 && "not implemented");
			for (rec = sigrec->records, recend = rec + sigrec->num_records; rec != recend; ++rec) {
				/// @todo Generate code that selects the appropriate driven value.
			}
		}
	}
	llhd_ptrmap_dispose(&migrated);

	I = llhd_inst_ret_new_many(func_returns, num_outputs);
	for (i = 0; i < num_outputs; ++i) {
		llhd_value_unref(func_returns[i]);
	}
	llhd_free(func_returns);
	llhd_inst_append_to(I, func_block);
	llhd_value_unref(I);


	// Substitute the old process' input parameters with the new function's
	// input parameters, since the above basic block still refers to the old
	// inputs.
	for (i = 0; i < num_inputs; ++i) {
		llhd_value_t new, old;
		new = llhd_unit_get_input(func, i);
		old = llhd_unit_get_input(proc, input_mapping[i]);
		llhd_value_set_name(new, llhd_value_get_name(old));
		llhd_value_substitute(func_block, old, new);
	}
	llhd_free(input_mapping);


	// Remove the instructions that drive sequential signals, and create a new,
	// stripped-down version of the process that only has combinatorial signals
	// as outputs.
	for (rec = records.data, recend = rec + num_records; rec != recend; ++rec) {
		if (rec->is_sequential) {
			llhd_value_unlink(rec->inst);
			rec->inst = NULL;
		}
	}

	/// @todo Simplify proc in case certain instructions are now obsolete and
	///       certain parameters are no longer used. However, this step is
	///       optional.


	// Determine the type signature of the simplified process that only contains
	// the combinatorial part of the circuit. Only input and output parameters
	// that are in use are kept around.
	num_inputs = llhd_unit_get_num_inputs(proc);
	num_outputs = llhd_unit_get_num_outputs(proc);
	input_mapping = llhd_zalloc(num_inputs * sizeof(unsigned));
	output_mapping = llhd_zalloc(num_outputs * sizeof(unsigned));

	for (i = 0, n = 0; i < num_inputs; ++i) {
		if (llhd_value_has_users(llhd_unit_get_input(proc, i))) {
			input_mapping[n++] = i;
		}
	}
	num_inputs = n;

	for (i = 0, n = 0; i < num_outputs; ++i) {
		if (llhd_value_has_users(llhd_unit_get_output(proc, i))) {
			output_mapping[n++] = i;
		}
	}
	num_outputs = n;

	inputs = llhd_alloc(num_inputs * sizeof(llhd_type_t));
	outputs = llhd_alloc(num_outputs * sizeof(llhd_type_t));
	for (i = 0; i < num_inputs; ++i) {
		inputs[i] = llhd_type_get_input(proc_type, input_mapping[i]);
	}
	for (i = 0; i < num_outputs; ++i) {
		outputs[i] = llhd_type_get_output(proc_type, output_mapping[i]);
	}
	new_proc_type = llhd_type_new_comp(inputs, num_inputs, outputs, num_outputs);
	llhd_free(inputs);
	llhd_free(outputs);

	// Create the replacement process with the type signature found above. Then
	// instantiate the process within the entity, passing the inputs and outputs
	// that remain.
	new_proc = llhd_proc_new(new_proc_type, llhd_value_get_name(proc));
	llhd_type_unref(new_proc_type);
	llhd_unit_insert_after(new_proc, proc);
	llhd_value_unref(new_proc);

	input_values = llhd_alloc(num_inputs * sizeof(llhd_value_t));
	output_values = llhd_alloc(num_outputs * sizeof(llhd_value_t));

	for (i = 0; i < num_inputs; ++i) {
		input_values[i] = llhd_unit_get_input(entity, input_mapping[i]);
	}
	for (i = 0; i < num_outputs; ++i) {
		output_values[i] = llhd_unit_get_output(entity, output_mapping[i]);
	}

	I = llhd_inst_instance_new(new_proc, input_values, num_inputs, output_values, num_outputs, NULL);
	llhd_inst_append_to(I, entity);
	llhd_value_unref(I);

	llhd_free(input_values);
	llhd_free(output_values);


	// Move the blocks from the old process over to the new process, and replace
	// the old process' parameters with the new process' parameters.
	blocks = llhd_unit_get_blocks(proc);
	for (pos = llhd_block_first(blocks); (BB = llhd_block_next(blocks, &pos));) {
		llhd_value_ref(BB);
		llhd_value_unlink_from_parent(BB);
		llhd_block_append_to(BB, new_proc);
		llhd_value_unref(BB);
	}

	for (i = 0; i < num_inputs; ++i) {
		llhd_value_t old, new;
		old = llhd_unit_get_input(proc, input_mapping[i]);
		new = llhd_unit_get_input(new_proc, i);
		llhd_value_set_name(new, llhd_value_get_name(old));
		llhd_value_substitute(new_proc, old, new);
	}

	for (i = 0; i < num_outputs; ++i) {
		llhd_value_t old, new;
		old = llhd_unit_get_output(proc, output_mapping[i]);
		new = llhd_unit_get_output(new_proc, i);
		llhd_value_set_name(new, llhd_value_get_name(old));
		llhd_value_substitute(new_proc, old, new);
	}

	llhd_free(input_mapping);
	llhd_free(output_mapping);

	/// @todo Verify that the created function and entity are self-contained.
	///       That is, they should only refer to globals and values embedded
	///       within themselves. This catches bugs where certain instructions
	///       have not been unlinked from the old process properly.

	llhd_value_unlink(proc);
	llhd_value_unref(entity);
	llhd_value_unref(func);

	// Clean up.
	sigrec = signal_records;
	sigrecend = signal_records + num_signal_records;
	while (sigrec != sigrecend) {
		llhd_boolexpr_free(sigrec->cond);
		++sigrec;
	}

	rec = records.data;
	while (rec != recend) {
		llhd_boolexpr_free(rec->cond);
		++rec;
	}

	llhd_free(signal_records);
	llhd_buffer_dispose(&records);
}
