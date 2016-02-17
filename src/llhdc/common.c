// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/common.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>


void
llhd_init_value (llhd_value_t *V, const char *name, llhd_type_t *type) {
	assert(V);
	V->name = name ? strdup(name) : NULL;
	V->type = type;
}

void
llhd_dispose_value (void *v) {
	llhd_value_t *V = v;
	assert(V && V->_intf && V->_intf->dispose);
	V->_intf->dispose(V);
	if (V->name) {
		free(V->name);
		V->name = NULL;
	}
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


static void
llhd_dispose_proc (llhd_proc_t *P) {
	assert(P);
	printf("%s %p\n", __FUNCTION__, P);
	unsigned i;
	for (i = 0; i < P->num_in;  ++i) llhd_destroy_value(P->in[i]);
	for (i = 0; i < P->num_out; ++i) llhd_destroy_value(P->out[i]);
	if (P->in)  free(P->in);
	if (P->out) free(P->out);
	llhd_basic_block_t *BB = P->bb_head;
	while (BB) {
		llhd_basic_block_t *next = BB->next;
		llhd_destroy_value(BB);
		BB = next;
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
	printf("%s %p\n", __FUNCTION__, P);
	memset(P, 0, sizeof(*P));
	llhd_init_value((llhd_value_t*)P, name, NULL /* void type */);
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
	entry->parent = P;
	P->bb_head = entry;
	P->bb_tail = entry;
	return P;
}


static void
llhd_dispose_basic_block (llhd_basic_block_t *BB) {
	assert(BB);
	printf("%s %p\n", __FUNCTION__, BB);
}

static void
llhd_dump_basic_block (llhd_basic_block_t *BB, FILE *f) {
	assert(BB);
	fprintf(f, "%s:", ((llhd_value_t*)BB)->name);
	llhd_inst_t *I;
	for (I = BB->inst_head; I; I = I->next) {
		fputs("  ", f);
		llhd_dump_value(I, f);
		fputc('\n', f);
	}
}

static struct llhd_value_intf basic_block_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_dispose_basic_block,
	.dump = (llhd_value_intf_dump_fn)llhd_dump_basic_block,
};

llhd_basic_block_t *
llhd_make_basic_block (const char *name) {
	llhd_basic_block_t *BB = malloc(sizeof(*BB));
	printf("%s %p\n", __FUNCTION__, BB);
	memset(BB, 0, sizeof(*BB));
	llhd_init_value((llhd_value_t*)BB, name, NULL /* label type */);
	BB->_base._intf = &basic_block_value_intf;
	return BB;
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


llhd_drive_inst_t *
llhd_make_drive_inst () {
	llhd_drive_inst_t *I = malloc(sizeof(*I));
	memset(I, 0, sizeof(*I));
	return I;
}

void
llhd_dispose_drive_inst (llhd_drive_inst_t *I) {
	assert(I);
}


static void
llhd_dump_arg (llhd_arg_t *A, FILE *f) {
	fprintf(f, "void %%%s", ((llhd_value_t*)A)->name);
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
	A->_base._intf = &arg_value_intf;
	return A;
}
