// Copyright (c) 2016 Fabian Schuiki
#include "value.h"
#include "inst.h"
#include <stdint.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>

llhd_type_t llhd_type_new_int(unsigned);

static char *const_int_to_string(void*);

static void entity_add_inst(void*, struct llhd_value*, int);
static void entity_remove_inst(void*, struct llhd_value*);
static void entity_dispose(void*);

static struct llhd_const_vtbl vtbl_const_int = {
	.super = {
		.kind = LLHD_VALUE_CONST,
		.type_offset = offsetof(struct llhd_const_int, type),
	},
	.kind = LLHD_CONST_INT,
	.to_string_fn = const_int_to_string,
};

static struct llhd_unit_vtbl vtbl_entity = {
	.super = {
		.kind = LLHD_VALUE_UNIT,
		.name_offset = offsetof(struct llhd_entity, name),
		.type_offset = offsetof(struct llhd_entity, type),
		.add_inst_fn = entity_add_inst,
		.remove_inst_fn = entity_remove_inst,
		.dispose_fn = entity_dispose,
	},
	.kind = LLHD_UNIT_DEF_ENTITY,
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
	C->type = llhd_type_new_int(32);
	C->value = v;
	return (struct llhd_value *)C;
}

static char *
const_int_to_string(void *ptr) {
	struct llhd_const_int *C = ptr;
	char buf[21];
	snprintf(buf, 21, "%llu", C->value);
	return strdup(buf);
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

char *
llhd_const_to_string(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_CONST);
	struct llhd_const_vtbl *vtbl = (void*)V->vtbl;
	assert(vtbl->to_string_fn);
	return vtbl->to_string_fn(V);
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
		assert(!llhd_value_has_users(V));
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
	llhd_type_ref(T);
	unsigned num_inputs = llhd_type_get_num_inputs(T);
	unsigned num_outputs = llhd_type_get_num_outputs(T);
	E = llhd_alloc_unit(sizeof(*E), &vtbl_entity, num_inputs+num_outputs);
	E->name = strdup(name);
	E->type = T;
	E->super.num_inputs = num_inputs;
	E->super.num_outputs = num_outputs;
	llhd_list_init(&E->insts);
	return (struct llhd_value *)E;
}

static void
entity_dispose(void *ptr) {
	assert(ptr);
	struct llhd_entity *E = ptr;
	llhd_free(E->name);
	llhd_type_unref(E->type);
}

static void
entity_add_inst(void *ptr, struct llhd_value *I, int append) {
	assert(I && I->vtbl && I->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_entity *E = ptr;
	llhd_value_ref(I);
	llhd_list_insert(append ? E->insts.prev : &E->insts, &((struct llhd_inst *)I)->link);
}

static void
entity_remove_inst(void *ptr, struct llhd_value *I) {
	assert(I && I->vtbl && I->vtbl->kind == LLHD_VALUE_INST);
	llhd_list_remove(&((struct llhd_inst *)I)->link);
	llhd_value_unref(I);
}

const char *
llhd_value_get_name(struct llhd_value *V) {
	assert(V && V->vtbl);
	size_t off = V->vtbl->name_offset;
	if (!off)
		return NULL;
	else
		return *(const char**)((void*)V+off);
}

struct llhd_type *
llhd_value_get_type(struct llhd_value *V) {
	assert(V && V->vtbl);
	size_t off = V->vtbl->type_offset;
	if (!off)
		return NULL;
	else
		return *(struct llhd_type**)((void*)V+off);
}

void *
llhd_alloc_unit(size_t sz, void *vtbl, unsigned num_params) {
	struct llhd_unit *U;
	assert(sz >= sizeof(*U));
	U = llhd_alloc_value(sz + sizeof(struct llhd_param*)*num_params,vtbl);
	U->params = (void*)U + sz;
	return U;
}

bool
llhd_unit_is(struct llhd_value *V, int kind) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	return ((struct llhd_unit_vtbl *)V->vtbl)->kind == kind;
}

int
llhd_unit_get_kind(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	return ((struct llhd_unit_vtbl *)V->vtbl)->kind;
}

bool
llhd_unit_is_def(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	int k = ((struct llhd_unit_vtbl *)V->vtbl)->kind;
	return k == LLHD_UNIT_DEF_FUNC || k == LLHD_UNIT_DEF_ENTITY || k == LLHD_UNIT_DEF_PROC;
}

bool
llhd_unit_is_decl(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	int k = ((struct llhd_unit_vtbl *)V->vtbl)->kind;
	return k == LLHD_UNIT_DECL;
}

struct llhd_value *
llhd_entity_get_first_inst(struct llhd_value *V) {
	assert(V && V->vtbl);
	struct llhd_entity *E = (void*)V;
	struct llhd_unit_vtbl *vtbl = (void*)V->vtbl;
	assert(V->vtbl->kind == LLHD_VALUE_UNIT && vtbl->kind == LLHD_UNIT_DEF_ENTITY);
	if (E->insts.next == &E->insts)
		return NULL;
	return (struct llhd_value*)llhd_container_of2(E->insts.next, struct llhd_inst, link);
}

struct llhd_value *
llhd_entity_get_last_inst(struct llhd_value *V) {
	assert(V && V->vtbl);
	struct llhd_entity *E = (void*)V;
	struct llhd_unit_vtbl *vtbl = (void*)V->vtbl;
	assert(V->vtbl->kind == LLHD_VALUE_UNIT && vtbl->kind == LLHD_UNIT_DEF_ENTITY);
	if (E->insts.prev == &E->insts)
		return NULL;
	return (struct llhd_value*)llhd_container_of2(E->insts.prev, struct llhd_inst, link);
}

unsigned
llhd_entity_get_num_insts(struct llhd_value *V) {
	assert(V && V->vtbl);
	struct llhd_entity *E = (void*)V;
	struct llhd_unit_vtbl *vtbl = (void*)V->vtbl;
	assert(V->vtbl->kind == LLHD_VALUE_UNIT && vtbl->kind == LLHD_UNIT_DEF_ENTITY);
	return llhd_list_length(&E->insts);
}

unsigned
llhd_unit_get_num_inputs(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	return ((struct llhd_unit*)V)->num_inputs;
}

unsigned
llhd_unit_get_num_outputs(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	return ((struct llhd_unit*)V)->num_outputs;
}

struct llhd_value *
llhd_unit_get_input(struct llhd_value *V, unsigned idx) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	struct llhd_unit *U = (void*)V;
	assert(idx < U->num_inputs);
	return (struct llhd_value*)U->params[idx];
}

struct llhd_value *
llhd_unit_get_output(struct llhd_value *V, unsigned idx) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_UNIT);
	struct llhd_unit *U = (void*)V;
	assert(idx < U->num_outputs);
	return (struct llhd_value*)U->params[U->num_inputs + idx];
}

void
llhd_value_unlink(struct llhd_value *V) {
	assert(V && V->vtbl);
	if (V->vtbl->unlink_uses_fn)
		V->vtbl->unlink_uses_fn(V);
	if (V->vtbl->unlink_from_parent_fn)
		V->vtbl->unlink_from_parent_fn(V);
}
