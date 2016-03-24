// Copyright (c) 2016 Fabian Schuiki
#include "value.h"
#include "inst.h"
#include <llhd.h>
#include <assert.h>

static void binary_dispose(void*);
static void binary_substitute(void*,void*,void*);

static struct llhd_inst_vtbl vtbl_binary_inst = {
	.super = {
		.kind = LLHD_VALUE_INST,
		.dispose_fn = binary_dispose,
		.substitute_fn = binary_substitute,
	},
	.kind = LLHD_INST_BINARY,
};

struct llhd_value *
llhd_inst_binary_new(int op, struct llhd_value *lhs, struct llhd_value *rhs, const char *name) {
	struct llhd_binary_inst *I;
	llhd_value_ref(lhs);
	llhd_value_ref(rhs);
	I = llhd_alloc_value(sizeof(*I), &vtbl_binary_inst);
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
	llhd_value_unref(I->lhs);
	llhd_value_unref(I->rhs);
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
	assert(to && to->vtbl && to->vtbl->add_inst_fn);
	to->vtbl->add_inst_fn(to,V,1);
}

void
llhd_inst_prepend_to(struct llhd_value *V, struct llhd_value *to) {
	assert(to && to->vtbl && to->vtbl->add_inst_fn);
	to->vtbl->add_inst_fn(to,V,0);
}
