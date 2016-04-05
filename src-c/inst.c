// Copyright (c) 2016 Fabian Schuiki
#include "value.h"
#include "inst.h"
#include <llhd.h>
#include <assert.h>
#include <string.h>

static void binary_dispose(void*);
static void binary_substitute(void*,void*,void*);
static void binary_unlink_from_parent(void*);
static void binary_unlink_uses(void*);

static struct llhd_inst_vtbl vtbl_binary_inst = {
	.super = {
		.kind = LLHD_VALUE_INST,
		.type_offset = offsetof(struct llhd_inst, type),
		.name_offset = offsetof(struct llhd_inst, name),
		.dispose_fn = binary_dispose,
		.substitute_fn = binary_substitute,
		.unlink_from_parent_fn = binary_unlink_from_parent,
		.unlink_uses_fn = binary_unlink_uses,
	},
	.kind = LLHD_INST_BINARY,
};

static const char *binary_opnames[] = {
	[LLHD_BINARY_ADD]  = "add",
	[LLHD_BINARY_SUB]  = "sub",
	[LLHD_BINARY_MUL]  = "mul",
	[LLHD_BINARY_UDIV] = "udiv",
	[LLHD_BINARY_UREM] = "urem",
	[LLHD_BINARY_SDIV] = "sdiv",
	[LLHD_BINARY_SREM] = "srem",
	[LLHD_BINARY_LSL]  = "lsl",
	[LLHD_BINARY_LSR]  = "lsr",
	[LLHD_BINARY_ASR]  = "asr",
	[LLHD_BINARY_AND]  = "and",
	[LLHD_BINARY_OR]   = "or",
	[LLHD_BINARY_XOR]  = "xor",
};

struct llhd_value *
llhd_inst_binary_new(int op, struct llhd_value *lhs, struct llhd_value *rhs, const char *name) {
	struct llhd_binary_inst *I;
	llhd_value_ref(lhs);
	llhd_value_ref(rhs);
	I = llhd_alloc_value(sizeof(*I), &vtbl_binary_inst);
	struct llhd_type *T = llhd_value_get_type(lhs);
	assert(T);
	llhd_type_ref(T);
	I->super.type = T;
	I->super.name = name ? strdup(name) : NULL;
	I->op = op;
	I->lhs = lhs;
	I->rhs = rhs;
	I->uses[0] = (struct llhd_value_use){ .user = (struct llhd_value*)I, .arg = 0 };
	I->uses[1] = (struct llhd_value_use){ .user = (struct llhd_value*)I, .arg = 1 };
	llhd_value_use(lhs, &I->uses[0]);
	llhd_value_use(rhs, &I->uses[1]);
	return (struct llhd_value *)I;
}

static void
binary_dispose(void *ptr) {
	struct llhd_binary_inst *I = ptr;
	assert(!I->super.parent);
	llhd_value_unref(I->lhs);
	llhd_value_unref(I->rhs);
	llhd_type_unref(I->super.type);
	llhd_free(I->super.name);
}

static void
binary_substitute(void *ptr, void *ref, void *sub) {
	struct llhd_binary_inst *I = ptr;
	if (I->lhs == ref && I->lhs != sub) {
		llhd_value_ref(sub);
		llhd_value_unuse(&I->uses[0]);
		llhd_value_use(sub, &I->uses[0]);
		llhd_value_unref(I->lhs);
		I->lhs = sub;
	}
	if (I->rhs == ref && I->rhs != sub) {
		llhd_value_ref(sub);
		llhd_value_unuse(&I->uses[1]);
		llhd_value_use(sub, &I->uses[1]);
		llhd_value_unref(I->rhs);
		I->rhs = sub;
	}
}

int
llhd_inst_binary_get_op(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst_vtbl *vtbl = (void*)V->vtbl;
	struct llhd_binary_inst *I = (void*)V;
	assert(vtbl->kind == LLHD_INST_BINARY);
	return I->op;
}

const char *
llhd_inst_binary_get_opname(struct llhd_value *V) {
	return binary_opnames[llhd_inst_binary_get_op(V)];
}

struct llhd_value *
llhd_inst_binary_get_lhs(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst_vtbl *vtbl = (void*)V->vtbl;
	struct llhd_binary_inst *I = (void*)V;
	assert(vtbl->kind == LLHD_INST_BINARY);
	return I->lhs;
}

struct llhd_value *
llhd_inst_binary_get_rhs(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst_vtbl *vtbl = (void*)V->vtbl;
	struct llhd_binary_inst *I = (void*)V;
	assert(vtbl->kind == LLHD_INST_BINARY);
	return I->rhs;
}


bool
llhd_inst_is(struct llhd_value *V, int kind) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	return ((struct llhd_inst_vtbl *)V->vtbl)->kind == kind;
}

int
llhd_inst_get_kind(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	return ((struct llhd_inst_vtbl *)V->vtbl)->kind;
}

void
llhd_inst_append_to(struct llhd_value *V, struct llhd_value *to) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst *I = (void*)V;
	assert(!I->parent);
	assert(to && to->vtbl && to->vtbl->add_inst_fn);
	I->parent = to;
	to->vtbl->add_inst_fn(to,V,1);
}

void
llhd_inst_prepend_to(struct llhd_value *V, struct llhd_value *to) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst *I = (void*)V;
	assert(!I->parent);
	assert(to && to->vtbl && to->vtbl->add_inst_fn);
	I->parent = to;
	to->vtbl->add_inst_fn(to,V,0);
}

struct llhd_value *
llhd_inst_next(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst *I = (void*)V;
	if (llhd_entity_get_last_inst(I->parent) == V)
		return NULL;
	return (struct llhd_value*)llhd_container_of(I->link.next,I,link);
}

struct llhd_value *
llhd_inst_prev(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->kind == LLHD_VALUE_INST);
	struct llhd_inst *I = (void*)V;
	if (llhd_entity_get_first_inst(I->parent) == V)
		return NULL;
	return (struct llhd_value*)llhd_container_of(I->link.prev,I,link);
}

static void
binary_unlink_from_parent(void *ptr) {
	struct llhd_inst *I = (struct llhd_inst*)ptr;
	struct llhd_value *P = I->parent;
	assert(P && P->vtbl);
	/// @todo Unlink from parent.
	// Must go before remove_inst_fn, since that might dispose and free the
	// inst, which triggers an assert on parent == NULL in the dispose function.
	I->parent = NULL;
	if (P->vtbl->remove_inst_fn)
		P->vtbl->remove_inst_fn(P, ptr);
}

static void
binary_unlink_uses(void *ptr) {
	struct llhd_binary_inst *I = (struct llhd_binary_inst*)ptr;
	/// @todo Unlink uses.
	llhd_value_unuse(&I->uses[0]);
	llhd_value_unuse(&I->uses[1]);
}
