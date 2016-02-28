// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/ir.h"
#include "src/llhdc/ir.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>



llhd_module_t *
llhd_unit_get_parent (llhd_unit_t *U) {
	assert(U);
	return U->parent;
}

llhd_unit_t *
llhd_unit_get_next (llhd_unit_t *U) {
	assert(U);
	return U->next;
}

llhd_unit_t *
llhd_unit_get_prev (llhd_unit_t *U) {
	assert(U);
	return U->prev;
}

void
llhd_unit_remove_from_parent (llhd_unit_t *U) {
	assert(U);
	assert(U->parent);
	if (U->parent->unit_head == U) U->parent->unit_head = NULL;
	if (U->parent->unit_tail == U) U->parent->unit_tail = NULL;
	U->parent = NULL;
}


void
llhd_unit_append_basic_block (llhd_unit_t *U, llhd_basic_block_t *BB) {
	assert(U);
	assert(BB);
	assert(!BB->parent && !BB->prev && !BB->next);
	BB->parent = U;
	++U->bb_num;
	if (!U->bb_tail) {
		U->bb_tail = BB;
		U->bb_head = BB;
	} else {
		BB->prev = U->bb_tail;
		U->bb_tail->next = BB;
		U->bb_tail = BB;
	}
}

llhd_basic_block_t *
llhd_unit_get_first_basic_block (llhd_unit_t *U) {
	assert(U);
	return U->bb_head;
}

llhd_basic_block_t *
llhd_unit_get_last_basic_block (llhd_unit_t *U) {
	assert(U);
	return U->bb_tail;
}

unsigned
llhd_unit_get_num_basic_block (llhd_unit_t *U) {
	assert(U);
	return U->bb_num;
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
	llhd_basic_block_t *BB = E->bb_head, *BBn;
	while (BB) {
		BBn = BB->next;
		llhd_value_destroy(BB);
		BB = BBn;
	}
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
	fprintf(f, ") {\n");
	llhd_basic_block_t *BB;
	for (BB = E->bb_head; BB; BB = BB->next) {
		llhd_value_dump(BB, f);
		fputc('\n', f);
	}
	fprintf(f, "}");
}

static struct llhd_value_intf entity_value_intf = {
	.dispose = (llhd_value_intf_dispose_fn)llhd_entity_dispose,
	.dump = (llhd_value_intf_dump_fn)llhd_entity_dump,
};

llhd_entity_t *
llhd_make_entity (const char *name, llhd_arg_t **in, unsigned num_in, llhd_arg_t **out, unsigned num_out, llhd_basic_block_t *entry) {
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
	assert(entry);
	assert(entry->parent == NULL);
	entry->parent = (void*)E;
	E->bb_head = entry;
	E->bb_tail = entry;
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
	BB->_value._intf = &basic_block_value_intf;
	return BB;
}

llhd_unit_t *
llhd_basic_block_get_parent (llhd_basic_block_t *BB) {
	assert(BB);
	return BB->parent;
}

llhd_basic_block_t *
llhd_basic_block_get_next (llhd_basic_block_t *BB) {
	assert(BB);
	return BB->next;
}

llhd_basic_block_t *
llhd_basic_block_get_prev (llhd_basic_block_t *BB) {
	assert(BB);
	return BB->prev;
}

void
llhd_basic_block_insert_before (llhd_basic_block_t *BB, llhd_basic_block_t *before) {
	assert(BB && before);
	assert(!BB->parent && !BB->next && !BB->prev);
	BB->parent = before->parent;
	BB->prev = before->prev;
	before->prev = BB;
	BB->next = before;
	if (BB->prev)
		BB->prev->next = BB;
	else
		BB->parent->bb_head = BB;
}

void
llhd_basic_block_insert_after (llhd_basic_block_t *BB, llhd_basic_block_t *after) {
	assert(BB && after);
	assert(!BB->parent && !BB->next && !BB->prev);
	BB->parent = after->parent;
	BB->next = after->next;
	after->next = BB;
	BB->prev = after;
	if (BB->next)
		BB->next->prev = BB;
	else
		BB->parent->bb_tail = BB;
}

void
llhd_basic_block_append_inst (llhd_basic_block_t *BB, llhd_inst_t *I) {
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

llhd_inst_t *
llhd_basic_block_get_first_inst (llhd_basic_block_t *BB) {
	assert(BB);
	return BB->inst_head;
}

llhd_inst_t *
llhd_basic_block_get_last_inst (llhd_basic_block_t *BB) {
	assert(BB);
	return BB->inst_tail;
}
