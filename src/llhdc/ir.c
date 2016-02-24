// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/ir.h"
#include "ir-internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>


void
llhd_value_init (llhd_value_t *V, const char *name, llhd_type_t *type) {
	assert(V);
	assert(type);
	V->name = name ? strdup(name) : NULL;
	V->type = type;
}

void
llhd_value_dispose (void *v) {
	llhd_value_t *V = v;
	assert(V);
	if (V->_intf && V->_intf->dispose)
		V->_intf->dispose(V);
	if (V->name) {
		free(V->name);
		V->name = NULL;
	}
	llhd_type_destroy(V->type);
	V->type = NULL;
}

void
llhd_value_destroy (void *V) {
	if (V) {
		llhd_value_dispose(V);
		free(V);
	}
}

void
llhd_value_dump (void *v, FILE *f) {
	llhd_value_t *V = v;
	assert(V && V->_intf && V->_intf->dump);
	V->_intf->dump(V, f);
}

void
llhd_value_dump_name (void *v, FILE *f) {
	llhd_value_t *V = v;
	assert(V);
	if (V->name) {
		fprintf(f, "%%%s", V->name);
	} else {
		assert(V->_intf && V->_intf->dump);
		fputc('(', f);
		V->_intf->dump(V, f);
		fputc(')', f);
	}
}

void
llhd_value_set_name (void *v, const char *name) {
	llhd_value_t *V = v;
	assert(V);
	if (V->name)
		free(V->name);
	V->name = name ? strdup(name) : NULL;
}

const char *
llhd_value_get_name (void *V) {
	assert(V);
	return ((llhd_value_t*)V)->name;
}


static void
llhd_const_int_dispose (llhd_const_int_t *C) {
	assert(C);
	free(C->value);
	C->value = NULL;
}

static void
llhd_const_int_dump (llhd_const_int_t *C, FILE *f) {
	assert(C);
	llhd_type_dump(C->_const._value.type, f);
	fputc(' ', f);
	fputs(C->value, f);
}

static struct llhd_value_intf const_int_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_const_int_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_const_int_dump,
};

llhd_const_int_t *
llhd_make_const_int (unsigned width, const char *value) {
	assert(value);
	llhd_const_int_t *C = malloc(sizeof(*C));
	memset(C, 0, sizeof(*C));
	llhd_value_init((llhd_value_t*)C, NULL, llhd_type_make_int(width));
	C->_const._value._intf = &const_int_value_intf;
	C->value = strdup(value);
	return C;
}


static void
llhd_const_logic_dispose (llhd_const_logic_t *C) {
	assert(C);
	free(C->value);
	C->value = NULL;
}

static void
llhd_const_logic_dump (llhd_const_logic_t *C, FILE *f) {
	assert(C);
	llhd_type_dump(C->_const._value.type, f);
	fputs(" \"", f);
	fputs(C->value, f);
	fputc('"', f);
}

static struct llhd_value_intf const_logic_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_const_logic_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_const_logic_dump,
};

llhd_const_logic_t *
llhd_make_const_logic (unsigned width, const char *value) {
	assert(value);
	llhd_const_logic_t *C = malloc(sizeof(*C));
	memset(C, 0, sizeof(*C));
	llhd_value_init((llhd_value_t*)C, NULL, llhd_type_make_logic(width));
	C->_const._value._intf = &const_logic_value_intf;
	C->value = strdup(value);
	return C;
}


static void
llhd_const_time_dispose (llhd_const_time_t *C) {
	assert(C);
	free(C->value);
	C->value = NULL;
}

static void
llhd_const_time_dump (llhd_const_time_t *C, FILE *f) {
	assert(C);
	fputs("time ", f);
	fputs(C->value, f);
}

static struct llhd_value_intf const_time_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_const_time_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_const_time_dump,
};

llhd_const_time_t *
llhd_make_const_time (const char *value) {
	assert(value);
	llhd_const_time_t *C = malloc(sizeof(*C));
	memset(C, 0, sizeof(*C));
	llhd_value_init((llhd_value_t*)C, NULL, llhd_type_make_time());
	C->_const._value._intf = &const_time_value_intf;
	C->value = strdup(value);
	return C;
}


static void
llhd_proc_dispose (llhd_proc_t *P) {
	assert(P);
	unsigned i;
	for (i = 0; i < P->num_in;  ++i) llhd_value_destroy(P->in[i]);
	for (i = 0; i < P->num_out; ++i) llhd_value_destroy(P->out[i]);
	if (P->in)  free(P->in);
	if (P->out) free(P->out);
	llhd_basic_block_t *BB = P->bb_head, *BBn;
	while (BB) {
		BBn = BB->next;
		llhd_value_destroy(BB);
		BB = BBn;
	}
}

static void
llhd_proc_dump (llhd_proc_t *P, FILE *f) {
	unsigned i;
	fprintf(f, "proc @%s (", ((llhd_value_t*)P)->name);
	for (i = 0; i < P->num_in; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_value_dump(P->in[i], f);
	}
	fprintf(f, ") (");
	for (i = 0; i < P->num_out; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_value_dump(P->out[i], f);
	}
	fprintf(f, ") {\n");
	llhd_basic_block_t *BB;
	for (BB = P->bb_head; BB; BB = BB->next) {
		llhd_value_dump(BB, f);
		fputc('\n', f);
	}
	fprintf(f, "}");
}

static struct llhd_value_intf proc_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_proc_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_proc_dump,
};

llhd_proc_t *
llhd_make_proc (const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry) {
	assert(name);
	assert(!num_in || in);
	assert(!num_out || out);
	llhd_proc_t *P = malloc(sizeof(*P));
	memset(P, 0, sizeof(*P));
	llhd_value_init((llhd_value_t*)P, name, llhd_type_make_void());
	P->_unit._value._intf = &proc_value_intf;
	P->num_in = num_in;
	P->num_out = num_out;
	if (num_in > 0) {
		unsigned size = num_in * sizeof(llhd_arg_t*);
		P->in = malloc(size);
		memcpy(P->in, in, size);
	}
	if (num_out > 0) {
		unsigned size = num_out * sizeof(llhd_arg_t*);
		P->out = malloc(size);
		memcpy(P->out, out, size);
	}
	assert(entry);
	assert(entry->parent == NULL);
	entry->parent = (void*)P;
	P->bb_head = entry;
	P->bb_tail = entry;
	return P;
}


static void
llhd_entity_dispose (llhd_entity_t *E) {
	assert(E);
	unsigned i;
	for (i = 0; i < E->num_in;  ++i) llhd_value_destroy(E->in[i]);
	for (i = 0; i < E->num_out; ++i) llhd_value_destroy(E->out[i]);
	if (E->in)  free(E->in);
	if (E->out) free(E->out);
}

static void
llhd_entity_dump (llhd_entity_t *E, FILE *f) {
	assert(E);
	unsigned i;
	fprintf(f, "entity @%s (", E->_unit._value.name);
	for (i = 0; i < E->num_in; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_value_dump(E->in[i], f);
	}
	fprintf(f, ") (");
	for (i = 0; i < E->num_out; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_value_dump(E->out[i], f);
	}
	fprintf(f, ") {");
	llhd_inst_t *I;
	for (I = E->inst_head; I; I = I->next) {
		fputs("\n  ", f);
		llhd_value_dump(I, f);
	}
	fprintf(f, "\n}");
}

static struct llhd_value_intf entity_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_entity_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_entity_dump,
};

llhd_entity_t *
llhd_make_entity (const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out) {
	assert(name);
	assert(!num_in || in);
	assert(!num_out || out);
	llhd_entity_t *E = malloc(sizeof(*E));
	memset(E, 0, sizeof(*E));
	llhd_value_init((llhd_value_t*)E, name, llhd_type_make_void());
	E->_unit._value._intf = &entity_value_intf;
	E->num_in = num_in;
	E->num_out = num_out;
	if (num_in > 0){
		unsigned size = num_in * sizeof(llhd_arg_t*);
		E->in = malloc(size);
		memcpy(E->in, in, size);
	}
	if (num_out > 0){
		unsigned size = num_out * sizeof(llhd_arg_t*);
		E->out = malloc(size);
		memcpy(E->out, out, size);
	}
	return E;
}


static void
llhd_basic_block_dispose (llhd_basic_block_t *BB) {
	assert(BB);
	llhd_inst_t *I = BB->inst_head, *In;
	while (I) {
		In = I->next;
		llhd_value_destroy(I);
		I = In;
	}
}

static void
llhd_basic_block_dump (llhd_basic_block_t *BB, FILE *f) {
	assert(BB);
	fprintf(f, "%s:", ((llhd_value_t*)BB)->name);
	llhd_inst_t *I;
	for (I = BB->inst_head; I; I = I->next) {
		fputs("\n  ", f);
		llhd_value_dump(I, f);
	}
}

static struct llhd_value_intf basic_block_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_basic_block_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_basic_block_dump,
};

llhd_basic_block_t *
llhd_make_basic_block (const char *name) {
	llhd_basic_block_t *BB = malloc(sizeof(*BB));
	memset(BB, 0, sizeof(*BB));
	llhd_value_init((llhd_value_t*)BB, name, llhd_type_make_label());
	BB->_base._intf = &basic_block_value_intf;
	return BB;
}

void
llhd_insert_basic_block_before (llhd_basic_block_t *BB, llhd_basic_block_t *before) {
	assert(BB && before);
	assert(BB->parent == NULL && BB->next == NULL && BB->prev == NULL);
	BB->parent = before->parent;
	BB->prev = before->prev;
	before->prev = BB;
	BB->next = before;
	if (BB->prev) BB->prev->next = BB;
}

void
llhd_insert_basic_block_after (llhd_basic_block_t *BB, llhd_basic_block_t *after) {
	assert(BB && after);
	assert(BB->parent == NULL && BB->next == NULL && BB->prev == NULL);
	BB->parent = after->parent;
	BB->next = after->next;
	after->next = BB;
	BB->prev = after;
	if (BB->next) BB->next->prev = BB;
}


void
llhd_basic_block_append (llhd_basic_block_t *BB, llhd_inst_t *I) {
	assert(BB && I);
	assert(!I->parent && !I->prev && !I->next);
	I->parent = BB;
	if (!BB->inst_tail) {
		BB->inst_tail = I;
		BB->inst_head = I;
	} else {
		I->prev = BB->inst_tail;
		BB->inst_tail->next = I;
		BB->inst_tail = I;
	}
}

void
llhd_entity_append (llhd_entity_t *E, llhd_inst_t *I) {
	assert(E && I);
	assert(!I->parent && !I->prev && !I->next);
	I->parent = (void*)E;
	if (!E->inst_tail) {
		E->inst_tail = I;
		E->inst_head = I;
	} else {
		I->prev = E->inst_tail;
		E->inst_tail->next = I;
		E->inst_tail = I;
	}
}


static void
llhd_drive_inst_dump (llhd_drive_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("drv ", f);
	llhd_type_dump(I->target->type, f);
	fputc(' ', f);
	llhd_value_dump_name(I->target, f);
	fputc(' ', f);
	llhd_value_dump_name(I->value, f);
}

static struct llhd_value_intf drive_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_drive_inst_dump,
};

llhd_drive_inst_t *
llhd_make_drive_inst (llhd_value_t *target, llhd_value_t *value) {
	assert(target && value);
	assert(llhd_type_equal(target->type, value->type));
	llhd_drive_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_void());
	I->_inst._value._intf = &drive_inst_value_intf;
	I->target = target;
	I->value = value;
	return I;
}


static const char *compare_inst_mode_str[] = {
	[LLHD_EQ] = "eq",
	[LLHD_NE] = "ne",
	[LLHD_UGT] = "ugt",
	[LLHD_ULT] = "ult",
	[LLHD_UGE] = "uge",
	[LLHD_ULE] = "ule",
	[LLHD_SGT] = "sgt",
	[LLHD_SLT] = "slt",
	[LLHD_SGE] = "sge",
	[LLHD_SLE] = "sle",
};

static void
llhd_compare_inst_dump (llhd_compare_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fprintf(f, "cmp %s ", compare_inst_mode_str[I->mode]);
	llhd_type_dump(I->lhs->type, f);
	fputc(' ', f);
	llhd_value_dump_name(I->lhs, f);
	fputc(' ', f);
	llhd_value_dump_name(I->rhs, f);
}

static struct llhd_value_intf compare_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_compare_inst_dump,
};

llhd_compare_inst_t *
llhd_make_compare_inst (llhd_compare_mode_t mode, llhd_value_t *lhs, llhd_value_t *rhs) {
	assert(lhs && rhs);
	assert(llhd_type_equal(lhs->type, rhs->type));
	llhd_compare_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_int(1));
	I->_inst._value._intf = &compare_inst_value_intf;
	I->mode = mode;
	I->lhs = lhs;
	I->rhs = rhs;
	return I;
}


static void
llhd_branch_inst_dump (llhd_branch_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("br ", f);
	if (I->cond) {
		llhd_value_dump_name(I->cond, f);
		fputs(", ", f);
		llhd_value_dump_name(I->dst1, f);
		fputs(", ", f);
		llhd_value_dump_name(I->dst0, f);
	} else {
		llhd_value_dump_name(I->dst1, f);
	}
}

static struct llhd_value_intf branch_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_branch_inst_dump,
};

llhd_branch_inst_t *
llhd_make_conditional_branch_inst (llhd_value_t *cond, llhd_basic_block_t *dst1, llhd_basic_block_t *dst0) {
	assert(cond && dst1 && dst0);
	assert(llhd_type_is_int_width(cond->type,1));
	assert(llhd_type_is_label(dst1->_base.type));
	assert(llhd_type_is_label(dst0->_base.type));
	llhd_branch_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_void());
	I->_inst._value._intf = &branch_inst_value_intf;
	I->cond = cond;
	I->dst1 = dst1;
	I->dst0 = dst0;
	return I;
}

llhd_branch_inst_t *
llhd_make_unconditional_branch_inst (llhd_basic_block_t *dst) {
	assert(dst);
	assert(llhd_type_is_label(dst->_base.type));
	llhd_branch_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_void());
	I->_inst._value._intf = &branch_inst_value_intf;
	I->dst1 = dst;
	return I;
}


static const char *unary_inst_op_str[] = {
	[LLHD_NOT] = "not",
};

static void
llhd_unary_inst_dump (llhd_unary_inst_t *I, FILE *f) {
	assert(I);
	const char *name = I->_inst._value.name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs(unary_inst_op_str[I->op], f);
	fputc(' ', f);
	llhd_type_dump(I->_inst._value.type, f);
	fputc(' ', f);
	llhd_value_dump_name(I->arg, f);
}

static struct llhd_value_intf unary_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_unary_inst_dump,
};

llhd_unary_inst_t *
llhd_make_unary_inst (llhd_unary_op_t op, llhd_value_t *arg) {
	assert(arg);
	llhd_unary_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_copy(arg->type));
	I->_inst._value._intf = &unary_inst_value_intf;
	I->op = op;
	I->arg = arg;
	return I;
}


static const char *binary_inst_op_str[] = {
	[LLHD_ADD] = "add",
	[LLHD_SUB] = "sub",
	[LLHD_MUL] = "mul",
	[LLHD_DIV] = "div",
	[LLHD_AND] = "and",
	[LLHD_OR]  = "or",
	[LLHD_XOR] = "xor",
};

static void
llhd_binary_inst_dump (llhd_binary_inst_t *I, FILE *f) {
	assert(I);
	const char *name = I->_inst._value.name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs(binary_inst_op_str[I->op], f);
	fputc(' ', f);
	llhd_type_dump(I->_inst._value.type, f);
	fputc(' ', f);
	llhd_value_dump_name(I->lhs, f);
	fputc(' ', f);
	llhd_value_dump_name(I->rhs, f);
}

static struct llhd_value_intf binary_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_binary_inst_dump,
};

llhd_binary_inst_t *
llhd_make_binary_inst (llhd_binary_op_t op, llhd_value_t *lhs, llhd_value_t *rhs) {
	assert(lhs && rhs);
	assert(llhd_type_equal(lhs->type, rhs->type));
	llhd_binary_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_copy(lhs->type));
	I->_inst._value._intf = &binary_inst_value_intf;
	I->op = op;
	I->lhs = lhs;
	I->rhs = rhs;
	return I;
}


static void
llhd_ret_inst_dump (llhd_ret_inst_t *I, FILE *f) {
	assert(I);
	fputs("ret", f);
	unsigned i;
	for (i = 0; i < I->num_values; ++i) {
		if (i > 0) fputc(',', f);
		fputc(' ', f);
		llhd_value_dump_name(I->values[i], f);
	}
}

static struct llhd_value_intf ret_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_ret_inst_dump,
};

llhd_ret_inst_t *
llhd_make_ret_inst (llhd_value_t **values, unsigned num_values) {
	assert(num_values == 0 || values);
	unsigned values_size = sizeof(llhd_value_t*) * num_values;
	unsigned size = sizeof(llhd_ret_inst_t) + values_size;
	llhd_ret_inst_t *I = malloc(size);
	memset(I, 0, size);
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_void());
	I->_inst._value._intf = &ret_inst_value_intf;
	memcpy(I->values, values, values_size);
	return I;
}


static void
llhd_wait_inst_dump (llhd_wait_inst_t *I, FILE *f) {
	assert(I);
	fputs("wait ", f);
	llhd_value_dump_name(I->duration, f);
}

static struct llhd_value_intf wait_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_wait_inst_dump,
};

llhd_wait_inst_t *
llhd_make_wait_inst (llhd_value_t *duration) {
	// assert(duration);
	llhd_wait_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init((llhd_value_t*)I, NULL, llhd_type_make_void());
	I->_inst._value._intf = &wait_inst_value_intf;
	I->duration = duration;
	return I;
}


static void
llhd_signal_inst_dump (llhd_signal_inst_t *I, FILE *f) {
	assert(I);
	const char *name = I->_inst._value.name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("sig ", f);
	llhd_type_dump(I->_inst._value.type, f);
}

static struct llhd_value_intf signal_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_signal_inst_dump,
};

llhd_signal_inst_t *
llhd_make_signal_inst (llhd_type_t *type) {
	assert(type);
	llhd_signal_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init(&I->_inst._value, NULL, type);
	I->_inst._value._intf = &signal_inst_value_intf;
	return I;
}


static void
llhd_instance_inst_dispose (llhd_instance_inst_t *I) {
	assert(I);
	if (I->in) free(I->in);
	if (I->out) free(I->out);
}

static void
llhd_instance_inst_dump (llhd_instance_inst_t *I, FILE *f) {
	assert(I);
	unsigned i;
	const char *name = I->_inst._value.name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("inst ", f);
	llhd_value_dump_name(I->value, f);
	fputs(" (", f);
	for (i = 0; i < I->num_in; ++i) {
		if (i > 0) fputs(", ", f);
		llhd_value_dump_name(I->in[i], f);
	}
	fputs(") (", f);
	for (i = 0; i < I->num_out; ++i) {
		if (i > 0) fputs(", ", f);
		llhd_value_dump_name(I->out[i], f);
	}
	fputs(")", f);
}

static struct llhd_value_intf instance_inst_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_instance_inst_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_instance_inst_dump,
};

llhd_instance_inst_t *
llhd_make_instance_inst (llhd_value_t *value, llhd_value_t **in, unsigned num_in, llhd_value_t **out, unsigned num_out) {
	assert(value);
	assert(num_in == 0 || in);
	assert(num_out == 0 || out);
	// TODO: assert that the in/out types match
	llhd_instance_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_value_init(&I->_inst._value, NULL, llhd_type_make_void());
	I->_inst._value._intf = &instance_inst_value_intf;
	I->value = value;
	I->num_in = num_in;
	I->num_out = num_out;
	if (num_in > 0) {
		unsigned size = num_in * sizeof(llhd_value_t*);
		I->in = malloc(size);
		memcpy(I->in, in, size);
	}
	if (num_out > 0) {
		unsigned size = num_out * sizeof(llhd_value_t*);
		I->out = malloc(size);
		memcpy(I->out, out, size);
	}
	return I;
}


static void
llhd_arg_dump (llhd_arg_t *A, FILE *f) {
	llhd_type_dump(A->_value.type, f);
	fprintf(f, " %%%s", A->_value.name);
}

static void
llhd_arg_dispose (llhd_arg_t *A) {
	assert(A);
}

static struct llhd_value_intf arg_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_arg_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_arg_dump,
};

llhd_arg_t *
llhd_make_arg (const char *name, llhd_type_t *type) {
	llhd_arg_t *A = malloc(sizeof(*A));
	memset(A, 0, sizeof(*A));
	llhd_value_init((llhd_value_t*)A, name, type);
	A->_value._intf = &arg_value_intf;
	return A;
}
