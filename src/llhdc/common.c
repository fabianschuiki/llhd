// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/common.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>


void
llhd_init_value (llhd_value_t *V, const char *name, llhd_type_t *type) {
	assert(V);
	assert(type);
	V->name = name ? strdup(name) : NULL;
	V->type = type;
}

void
llhd_dispose_value (void *v) {
	llhd_value_t *V = v;
	assert(V);
	if (V->_intf && V->_intf->dispose)
		V->_intf->dispose(V);
	if (V->name) {
		free(V->name);
		V->name = NULL;
	}
	llhd_destroy_type(V->type);
	V->type = NULL;
}

void
llhd_destroy_value (void *V) {
	if (V) {
		llhd_dispose_value(V);
		free(V);
	}
}

void
llhd_dump_value (void *v, FILE *f) {
	llhd_value_t *V = v;
	assert(V && V->_intf && V->_intf->dump);
	V->_intf->dump(V, f);
}

void
llhd_dump_value_name (void *v, FILE *f) {
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
llhd_dispose_const_int (llhd_const_int_t *C) {
	assert(C);
	free(C->value);
}

static void
llhd_dump_const_int (llhd_const_int_t *C, FILE *f) {
	assert(C);
	llhd_dump_type(C->_const._value.type, f);
	fputc(' ', f);
	fputs(C->value, f);
}

static struct llhd_value_intf const_int_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_const_int,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_const_int,
};

llhd_const_int_t *
llhd_make_const_int (unsigned width, const char *value) {
	assert(value);
	llhd_const_int_t *C = malloc(sizeof(*C));
	memset(C, 0, sizeof(*C));
	llhd_init_value((llhd_value_t*)C, NULL, llhd_make_int_type(width));
	C->_const._value._intf = &const_int_value_intf;
	C->value = strdup(value);
	return C;
}


static void
llhd_dispose_const_logic (llhd_const_logic_t *C) {
	assert(C);
	free(C->value);
}

static void
llhd_dump_const_logic (llhd_const_logic_t *C, FILE *f) {
	assert(C);
	llhd_dump_type(C->_const._value.type, f);
	fputs(" \"", f);
	fputs(C->value, f);
	fputc('"', f);
}

static struct llhd_value_intf const_logic_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_const_logic,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_const_logic,
};

llhd_const_logic_t *
llhd_make_const_logic (unsigned width, const char *value) {
	assert(value);
	llhd_const_logic_t *C = malloc(sizeof(*C));
	memset(C, 0, sizeof(*C));
	llhd_init_value((llhd_value_t*)C, NULL, llhd_make_logic_type(width));
	C->_const._value._intf = &const_logic_value_intf;
	C->value = strdup(value);
	return C;
}


static void
llhd_dispose_proc (llhd_proc_t *P) {
	assert(P);
	unsigned i;
	for (i = 0; i < P->num_in;  ++i) llhd_destroy_value(P->in[i]);
	for (i = 0; i < P->num_out; ++i) llhd_destroy_value(P->out[i]);
	if (P->in)  free(P->in);
	if (P->out) free(P->out);
	llhd_basic_block_t *BB = P->bb_head, *BBn;
	while (BB) {
		BBn = BB->next;
		llhd_destroy_value(BB);
		BB = BBn;
	}
}

static void
llhd_dump_proc (llhd_proc_t *P, FILE *f) {
	unsigned i;
	fprintf(f, "proc @%s (", ((llhd_value_t*)P)->name);
	for (i = 0; i < P->num_in; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_dump_value(P->in[i], f);
	}
	fprintf(f, ") (");
	for (i = 0; i < P->num_out; ++i) {
		if (i > 0) fprintf(f, ", ");
		llhd_dump_value(P->out[i], f);
	}
	fprintf(f, ") {\n");
	llhd_basic_block_t *BB;
	for (BB = P->bb_head; BB; BB = BB->next) {
		llhd_dump_value(BB, f);
		fputc('\n', f);
	}
	fprintf(f, "}");
}

static struct llhd_value_intf proc_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_proc,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_proc,
};

llhd_proc_t *
llhd_make_proc (const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry) {
	assert(name);
	llhd_proc_t *P = malloc(sizeof(*P));
	memset(P, 0, sizeof(*P));
	llhd_init_value((llhd_value_t*)P, name, llhd_make_void_type());
	P->_base._intf = &proc_value_intf;
	P->num_in = num_in;
	P->num_out = num_out;
	if (num_in > 0) {
		P->in = malloc(num_in * sizeof(llhd_arg_t));
		memcpy(P->in, in, num_in * sizeof(llhd_arg_t));
	}
	if (num_out > 0) {
		P->out = malloc(num_out * sizeof(llhd_arg_t));
		memcpy(P->out, out, num_out * sizeof(llhd_arg_t));
	}
	assert(entry);
	assert(entry->parent == NULL);
	entry->parent = P;
	P->bb_head = entry;
	P->bb_tail = entry;
	return P;
}


static void
llhd_dispose_basic_block (llhd_basic_block_t *BB) {
	assert(BB);
	llhd_inst_t *I = BB->inst_head, *In;
	while (I) {
		In = I->next;
		llhd_destroy_value(I);
		I = In;
	}
}

static void
llhd_dump_basic_block (llhd_basic_block_t *BB, FILE *f) {
	assert(BB);
	fprintf(f, "%s:", ((llhd_value_t*)BB)->name);
	llhd_inst_t *I;
	for (I = BB->inst_head; I; I = I->next) {
		fputs("\n  ", f);
		llhd_dump_value(I, f);
	}
}

static struct llhd_value_intf basic_block_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_basic_block,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_basic_block,
};

llhd_basic_block_t *
llhd_make_basic_block (const char *name) {
	llhd_basic_block_t *BB = malloc(sizeof(*BB));
	memset(BB, 0, sizeof(*BB));
	llhd_init_value((llhd_value_t*)BB, name, llhd_make_label_type());
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
llhd_add_inst (llhd_inst_t *I, llhd_basic_block_t *BB) {
	assert(I && BB);
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


static void
llhd_dump_drive_inst (llhd_drive_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("drv ", f);
	llhd_dump_value_name(I->target, f);
	fputc(' ', f);
	llhd_dump_value_name(I->value, f);
}

static struct llhd_value_intf drive_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_dump_drive_inst,
};

llhd_drive_inst_t *
llhd_make_drive_inst (llhd_value_t *target, llhd_value_t *value) {
	assert(target && value);
	assert(llhd_equal_types(target->type, value->type));
	llhd_drive_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_init_value((llhd_value_t*)I, NULL, llhd_make_void_type());
	I->_base._base._intf = &drive_inst_value_intf;
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
llhd_dump_compare_inst (llhd_compare_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fprintf(f, "cmp %s ", compare_inst_mode_str[I->mode]);
	llhd_dump_value_name(I->lhs, f);
	fputc(' ', f);
	llhd_dump_value_name(I->rhs, f);
}

static struct llhd_value_intf compare_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_dump_compare_inst,
};

llhd_compare_inst_t *
llhd_make_compare_inst (llhd_compare_mode_t mode, llhd_value_t *lhs, llhd_value_t *rhs) {
	assert(lhs && rhs);
	assert(llhd_equal_types(lhs->type, rhs->type));
	llhd_compare_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_init_value((llhd_value_t*)I, NULL, llhd_make_int_type(1));
	I->_base._base._intf = &compare_inst_value_intf;
	I->mode = mode;
	I->lhs = lhs;
	I->rhs = rhs;
	return I;
}


static void
llhd_dump_branch_inst (llhd_branch_inst_t *I, FILE *f) {
	assert(I);
	const char *name = ((llhd_value_t*)I)->name;
	if (name)
		fprintf(f, "%%%s = ", name);
	fputs("br ", f);
	if (I->cond) {
		llhd_dump_value_name(I->cond, f);
		fputs(", ", f);
		llhd_dump_value_name(I->dst1, f);
		fputs(", ", f);
		llhd_dump_value_name(I->dst0, f);
	} else {
		llhd_dump_value_name(I->dst1, f);
	}
}

static struct llhd_value_intf branch_inst_value_intf = {
	.dump = (llhd_value_intf_dump_fn)llhd_dump_branch_inst,
};

llhd_branch_inst_t *
llhd_make_conditional_branch_inst (llhd_value_t *cond, llhd_basic_block_t *dst1, llhd_basic_block_t *dst0) {
	assert(cond && dst1 && dst0);
	assert(llhd_type_is_int_width(cond->type,1));
	assert(llhd_type_is_label(dst1->_base.type));
	assert(llhd_type_is_label(dst0->_base.type));
	llhd_branch_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	llhd_init_value((llhd_value_t*)I, NULL, llhd_make_void_type());
	I->_base._base._intf = &branch_inst_value_intf;
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
	llhd_init_value((llhd_value_t*)I, NULL, llhd_make_void_type());
	I->_base._base._intf = &branch_inst_value_intf;
	I->dst1 = dst;
	return I;
}


static void
llhd_dump_arg (llhd_arg_t *A, FILE *f) {
	llhd_dump_type(A->_value.type, f);
	fprintf(f, " %%%s", A->_value.name);
}

static void
llhd_dispose_arg (llhd_arg_t *A) {
	assert(A);
}

static struct llhd_value_intf arg_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_arg,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_arg,
};

llhd_arg_t *
llhd_make_arg (const char *name, llhd_type_t *type) {
	llhd_arg_t *A = malloc(sizeof(*A));
	memset(A, 0, sizeof(*A));
	llhd_init_value((llhd_value_t*)A, name, type);
	A->_value._intf = &arg_value_intf;
	return A;
}


static llhd_type_t *
make_type(enum llhd_type_kind kind, unsigned num_inner) {
	unsigned size = sizeof(llhd_type_t) + sizeof(llhd_type_t*) * num_inner;
	llhd_type_t *T = malloc(size);
	memset(T, 0, size);
	T->kind = kind;
	return T;
}

llhd_type_t *
llhd_make_void_type () {
	return make_type(LLHD_VOID_TYPE, 0);
}

llhd_type_t *
llhd_make_label_type () {
	return make_type(LLHD_LABEL_TYPE, 0);
}

llhd_type_t *
llhd_make_int_type (unsigned width) {
	llhd_type_t *T = make_type(LLHD_INT_TYPE, 0);
	T->length = width;
	return T;
}

llhd_type_t *
llhd_make_logic_type (unsigned width) {
	llhd_type_t *T = make_type(LLHD_LOGIC_TYPE, 0);
	T->length = width;
	return T;
}

llhd_type_t *
llhd_make_struct_type (llhd_struct_field_t **fields, unsigned num_fields) {
	assert(!num_fields || fields);
	llhd_type_t *T = make_type(LLHD_STRUCT_TYPE, num_fields);
	T->length = num_fields;
	memcpy(T->inner, fields, sizeof(*fields) * num_fields);
	return T;
}

llhd_type_t *
llhd_make_array_type (llhd_type_t *element, unsigned length) {
	assert(element);
	llhd_type_t *T = make_type(LLHD_ARRAY_TYPE, 1);
	T->length = length;
	T->inner[0] = element;
	return T;
}

llhd_type_t *
llhd_make_ptr_type (llhd_type_t *to) {
	assert(to);
	llhd_type_t *T = make_type(LLHD_PTR_TYPE, 1);
	T->inner[0] = to;
	return T;
}

static void
llhd_dispose_type (llhd_type_t *T) {
	assert(T);
	unsigned i;
	switch (T->kind) {
	case LLHD_VOID_TYPE:
	case LLHD_LABEL_TYPE:
	case LLHD_INT_TYPE:
	case LLHD_LOGIC_TYPE:
		break;
	case LLHD_STRUCT_TYPE:
		for (i = 0; i < T->length; ++i)
			llhd_destroy_type(T->inner[i]);
		break;
	case LLHD_ARRAY_TYPE:
	case LLHD_PTR_TYPE:
		llhd_destroy_type(T->inner[0]);
		break;
	}
}

void
llhd_destroy_type (llhd_type_t *T) {
	if (T) {
		llhd_dispose_type(T);
		free(T);
	}
}

void
llhd_dump_type (llhd_type_t *T, FILE *f) {
	assert(T);
	unsigned i;
	switch (T->kind) {
	case LLHD_VOID_TYPE: fputs("void", f); break;
	case LLHD_LABEL_TYPE: fputs("label", f); break;
	case LLHD_INT_TYPE: fprintf(f, "i%u", T->length); break;
	case LLHD_LOGIC_TYPE: fprintf(f, "l%u", T->length); break;
	case LLHD_STRUCT_TYPE:
		fputs("{ ", f);
		for (i = 0; i < T->length; ++i) {
			if (i > 0) fputs(", ", f);
			llhd_dump_type(T->inner[i], f);
		}
		fputs(" }", f);
		break;
	case LLHD_ARRAY_TYPE:
		fprintf(f, "[%u x ", T->length);
		llhd_dump_type(T->inner[0], f);
		fputc(']', f);
		break;
	case LLHD_PTR_TYPE:
		llhd_dump_type(T->inner[0], f);
		fputc('*', f);
		break;
	}
}

int
llhd_equal_types (llhd_type_t *A, llhd_type_t *B) {
	assert(A && B);
	if (A->kind != B->kind)
		return 0;
	unsigned i;
	switch (A->kind) {
	case LLHD_VOID_TYPE:
	case LLHD_LABEL_TYPE:
		return 1;
	case LLHD_INT_TYPE:
	case LLHD_LOGIC_TYPE:
		return A->length == B->length;
	case LLHD_STRUCT_TYPE:
		if (A->length != B->length)
			return 0;
		for (i = 0; i < A->length; ++i)
			if (!llhd_equal_types(A->inner[i], B->inner[i]))
				return 0;
		return 1;
	case LLHD_ARRAY_TYPE:
		if (A->length != B->length)
			return 0;
	case LLHD_PTR_TYPE:
		return llhd_equal_types(A->inner[0], B->inner[0]);
	}
	assert(0);
}

int
llhd_type_is_void (llhd_type_t *T) {
	return T->kind == LLHD_VOID_TYPE;
}

int
llhd_type_is_label (llhd_type_t *T) {
	return T->kind == LLHD_LABEL_TYPE;
}

int
llhd_type_is_int (llhd_type_t *T) {
	return T->kind == LLHD_INT_TYPE;
}

int
llhd_type_is_int_width (llhd_type_t *T, unsigned width) {
	return T->kind == LLHD_INT_TYPE && T->length == width;
}

int
llhd_type_is_logic (llhd_type_t *T) {
	return T->kind == LLHD_LOGIC_TYPE;
}

int
llhd_type_is_logic_width (llhd_type_t *T, unsigned width) {
	return T->kind == LLHD_LOGIC_TYPE && T->length == width;
}

int
llhd_type_is_struct (llhd_type_t *T) {
	return T->kind == LLHD_STRUCT_TYPE;
}

int
llhd_type_is_array (llhd_type_t *T) {
	return T->kind == LLHD_ARRAY_TYPE;
}

int
llhd_type_is_ptr (llhd_type_t *T) {
	return T->kind == LLHD_PTR_TYPE;
}
