// Copyright (c) 2016 Fabian Schuiki
#include "value.h"
#include "inst.h"
#include <stdint.h>
#include <assert.h>
#include <stdio.h>

static void entity_add_inst(void*,struct llhd_value*,int);

struct llhd_const_int {
	struct llhd_value super;
	uint64_t value;
};

struct llhd_const_vtbl {
	struct llhd_value_vtbl super;
	int kind;
};

struct llhd_entity {
	struct llhd_value super;
	struct llhd_list in_params;
	struct llhd_list out_params;
	struct llhd_list insts;
};

static struct llhd_const_vtbl vtbl_const_int = {
	.super = {
		.kind = LLHD_VALUE_CONST,
	},
	.kind = LLHD_CONST_INT,
};

static struct llhd_value_vtbl vtbl_entity = {
	.kind = LLHD_VALUE_UNIT,
	.add_inst_fn = entity_add_inst,
};

void *
llhd_alloc_value(size_t sz, void *vtbl) {
	struct llhd_value *V;
	assert(sz >= sizeof(*V));
	assert(vtbl);
	V = llhd_zalloc(sz);
	V->vtbl = vtbl;
	V->rc = 1;
	llhd_list_init(&V->users);
	return V;
}

struct llhd_value *
llhd_const_int_new(uint64_t v) {
	struct llhd_const_int *C;
	C = llhd_alloc_value(sizeof(*C), &vtbl_const_int);
	C->value = v;
	return (struct llhd_value *)C;
}

bool
llhd_const_is(struct llhd_value *V, int kind) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_CONST);
	return ((struct llhd_const_vtbl *)V->vtbl)->kind == kind;
}

int
llhd_const_get_kind(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_CONST);
	return ((struct llhd_const_vtbl *)V->vtbl)->kind;
}

uint64_t
llhd_const_int_get_value(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_CONST);
	struct llhd_const_vtbl *vtbl = (void*)V->vtbl;
	struct llhd_const_int *C = (void*)V;
	assert(vtbl->kind == LLHD_CONST_INT);
	return C->value;
}

bool
llhd_value_is(struct llhd_value *V, int kind) {
	assert(V);
	assert(V->vtbl);
	return V->vtbl->kind == kind;
}

int
llhd_value_get_kind(struct llhd_value *V) {
	assert(V);
	assert(V->vtbl);
	return V->vtbl->kind;
}

bool
llhd_value_has_users(struct llhd_value *V) {
	return !llhd_list_empty(&V->users);
}

unsigned
llhd_value_get_num_users(struct llhd_value *V) {
	return llhd_list_length(&V->users);
}

/**
 * Increase the reference count of a value.
 *
 * @param V The value whose reference count to increase.
 *
 * @memberof llhd_value
 */
void
llhd_value_ref(struct llhd_value *V) {
	assert(V->rc > 0 && "ref on invalid value");
	++V->rc;
}

/**
 * Decrease the reference count of a value. Frees the value with a call to
 * llhd_free if no more references are held to the value.
 *
 * @param V The value whose reference count to decrease.
 *
 * @memberof llhd_value
 */
void
llhd_value_unref(struct llhd_value *V) {
	assert(V->rc > 0 && "double-unref");
	if (--V->rc == 0) {
		/// @todo Assert unlinked.
		if (V->vtbl->dispose_fn)
			V->vtbl->dispose_fn(V);
		llhd_free(V);
	}
}

void
llhd_value_use(struct llhd_value *V, struct llhd_value_use *U) {
	assert(V && U);
	llhd_list_insert(&V->users, &U->link);
}

void
llhd_value_unuse(struct llhd_value_use *U) {
	assert(U);
	llhd_list_remove(&U->link);
}

void
llhd_value_replace_uses(struct llhd_value *V, struct llhd_value *R) {
	struct llhd_list *u, *uz;
	unsigned count;

	u = V->users.next;
	uz = &V->users;
	count = 0;
	while (u != uz) {
		struct llhd_value_use *U;
		U = llhd_container_of(u, U, link);
		u = u->next;
		++count;
		assert(U->user && U->user->vtbl && U->user->vtbl->substitute_fn);
		U->user->vtbl->substitute_fn(U->user, V, R);
	}
}

struct llhd_value *
llhd_entity_new(struct llhd_type *T, const char *name) {
	struct llhd_entity *E;
	assert(T && name);
	E = llhd_alloc_value(sizeof(*E), &vtbl_entity);
	llhd_list_init(&E->in_params);
	llhd_list_init(&E->out_params);
	llhd_list_init(&E->insts);
	return (struct llhd_value *)E;
}

static void
entity_add_inst(void *ptr, struct llhd_value *I, int append) {
	assert(I && I->vtbl && I->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_entity *E = ptr;
	llhd_list_insert(append ? E->insts.prev : &E->insts, &((struct llhd_inst *)I)->link);
}
