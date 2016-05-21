/* Copyright (c) 2016 Fabian Schuiki */
#include "value.h"
#include "inst.h"
#include "module.h"

#include <stdint.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>

/**
 * @file
 * @author Fabian Schuiki <fabian@schuiki.ch>
 *
 * @todo Remove llhd_unit_get_blocks and replace it with the corresponding first
 *       and last block accessor. This makes the API much easier to use, as the
 *       llhd_list approach requires the user to properly calculate the offset
 *       from the list link to the start of the containing structure. Yuck. Do
 *       the same for llhd_module_get_units.
 */

static char *const_int_to_string(void*);
static void const_int_dispose(void*);

static void unit_unlink_from_parent(void*);

static void entity_add_inst(void*, struct llhd_value*, int);
static void entity_remove_inst(void*, struct llhd_value*);
static void entity_dispose(void*);
static void param_dispose(void*);

static void proc_add_block(void*, struct llhd_block*, int);
static void proc_remove_block(void*, struct llhd_block*);
static void proc_dispose(void*);
static void proc_substitute(void*,void*,void*);

static void func_add_block(void*, struct llhd_block*, int);
static void func_remove_block(void*, struct llhd_block*);
static void func_dispose(void*);

static void block_add_inst(void*, struct llhd_value*, int);
static void block_remove_inst(void*, struct llhd_value*);
static void block_dispose(void*);
static void block_substitute(void*,void*,void*);
static void block_unlink_from_parent(void*);

struct llhd_param {
	struct llhd_value super;
	struct llhd_type *type;
	char *name;
};

static struct llhd_value_vtbl vtbl_param = {
	.kind = LLHD_VALUE_PARAM,
	.type_offset = offsetof(struct llhd_param, type),
	.name_offset = offsetof(struct llhd_param, name),
	.dispose_fn = param_dispose,
};

static struct llhd_const_vtbl vtbl_const_int = {
	.super = {
		.kind = LLHD_CONST_INT,
		.type_offset = offsetof(struct llhd_const_int, type),
		.dispose_fn = const_int_dispose,
	},
	.to_string_fn = const_int_to_string,
};

static struct llhd_unit_vtbl vtbl_entity = {
	.super = {
		.kind = LLHD_UNIT_DEF_ENTITY,
		.name_offset = offsetof(struct llhd_entity, name),
		.type_offset = offsetof(struct llhd_entity, type),
		.add_inst_fn = entity_add_inst,
		.remove_inst_fn = entity_remove_inst,
		.dispose_fn = entity_dispose,
		.unlink_from_parent_fn = unit_unlink_from_parent,
	},
};

static struct llhd_unit_vtbl vtbl_proc = {
	.super = {
		.kind = LLHD_UNIT_DEF_PROC,
		.name_offset = offsetof(struct llhd_proc, name),
		.type_offset = offsetof(struct llhd_proc, type),
		.add_block_fn = proc_add_block,
		.remove_block_fn = proc_remove_block,
		.dispose_fn = proc_dispose,
		.unlink_from_parent_fn = unit_unlink_from_parent,
		.substitute_fn = proc_substitute,
	},
	.block_list_offset = offsetof(struct llhd_proc, blocks),
};

static struct llhd_unit_vtbl vtbl_func = {
	.super = {
		.kind = LLHD_UNIT_DEF_FUNC,
		.name_offset = offsetof(struct llhd_func, name),
		.type_offset = offsetof(struct llhd_func, type),
		.add_block_fn = func_add_block,
		.remove_block_fn = func_remove_block,
		.dispose_fn = func_dispose,
		.unlink_from_parent_fn = unit_unlink_from_parent,
	},
	.block_list_offset = offsetof(struct llhd_func, blocks),
};

static struct llhd_value_vtbl vtbl_block = {
	.kind = LLHD_VALUE_BLOCK,
	.name_offset = offsetof(struct llhd_block, name),
	.type_offset = offsetof(struct llhd_block, type),
	.add_inst_fn = block_add_inst,
	.remove_inst_fn = block_remove_inst,
	.dispose_fn = block_dispose,
	.substitute_fn = block_substitute,
	.unlink_from_parent_fn = block_unlink_from_parent,
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
llhd_const_int_new(unsigned bits, uint64_t v) {
	struct llhd_const_int *C;
	assert(bits > 0);
	C = llhd_alloc_value(sizeof(*C), &vtbl_const_int);
	C->type = llhd_type_new_int(bits);
	C->value = v;
	return (struct llhd_value *)C;
}

static char *
const_int_to_string(void *ptr) {
	struct llhd_const_int *C = ptr;
	char buf[21];
	snprintf(buf, 21, "%lu", C->value);
	return strdup(buf);
}

static void
const_int_dispose(void *ptr) {
	struct llhd_const_int *C = ptr;
	assert(ptr);
	llhd_type_unref(C->type);
}

uint64_t
llhd_const_int_get_value(struct llhd_value *V) {
	struct llhd_const_int *C = (void*)V;
	assert(V && V->vtbl && LLHD_ISA(V->vtbl->kind, LLHD_CONST_INT));
	return C->value;
}

char *
llhd_const_to_string(struct llhd_value *V) {
	struct llhd_const_vtbl *vtbl;
	assert(V && V->vtbl && LLHD_ISA(V->vtbl->kind, LLHD_VALUE_CONST));
	vtbl = (void*)V->vtbl;
	assert(vtbl->to_string_fn);
	return vtbl->to_string_fn(V);
}

bool
llhd_value_is(struct llhd_value *V, int kind) {
	int k = llhd_value_get_kind(V);
	return LLHD_ISA(k, kind);
}

int
llhd_value_get_kind(struct llhd_value *V) {
	size_t offset;
	assert(V && V->vtbl);
	offset = V->vtbl->kind_offset;
	if (offset)
		return *(int*)((void*)V + offset);
	else
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
	U->value = V;
}

void
llhd_value_unuse(struct llhd_value_use *U) {
	assert(U);
	if (U->value) {
		llhd_list_remove(&U->link);
		U->value = NULL;
	}
}

void
llhd_value_replace_uses(struct llhd_value *V, struct llhd_value *R) {
	struct llhd_list *u;
	assert(V && V->users.next);
	u = V->users.next;
	while (u != &V->users) {
		struct llhd_value_use *U;
		U = llhd_container_of(u, U, link);
		u = u->next;
		llhd_value_substitute(U->user, V, R);
	}
}

void
llhd_value_substitute(struct llhd_value *V, struct llhd_value *ref, struct llhd_value *sub) {
	assert(V && V->vtbl && V->vtbl->substitute_fn);
	V->vtbl->substitute_fn(V, ref, sub);
}

static struct llhd_param *
param_new(struct llhd_type *T) {
	struct llhd_param *P;
	assert(T);
	P = llhd_alloc_value(sizeof(*P), &vtbl_param);
	P->type = T;
	llhd_type_ref(T);
	return P;
}

static void
param_dispose(void *ptr) {
	assert(ptr);
	struct llhd_param *P = ptr;
	llhd_type_unref(P->type);
	if (P->name)
		llhd_free(P->name);
}

struct llhd_value *
llhd_entity_new(struct llhd_type *T, const char *name) {
	unsigned i;
	struct llhd_entity *E;
	assert(T && name && llhd_type_is(T, LLHD_TYPE_COMP));
	llhd_type_ref(T);
	unsigned num_inputs = llhd_type_get_num_inputs(T);
	unsigned num_outputs = llhd_type_get_num_outputs(T);
	E = llhd_alloc_unit(sizeof(*E), &vtbl_entity, num_inputs+num_outputs);
	E->name = strdup(name);
	E->type = T;
	E->super.num_inputs = num_inputs;
	E->super.num_outputs = num_outputs;
	for (i = 0; i < num_inputs; ++i)
		E->super.params[i] = param_new(llhd_type_get_input(T,i));
	for (i = 0; i < num_outputs; ++i)
		E->super.params[i+num_inputs] = param_new(llhd_type_get_output(T,i));
	llhd_list_init(&E->insts);
	return (struct llhd_value *)E;
}

static void
entity_dispose(void *ptr) {
	unsigned i;
	assert(ptr);
	struct llhd_entity *E = ptr;
	struct llhd_list *link;
	link = E->insts.next;
	while (link != &E->insts) {
		struct llhd_inst *I;
		I = llhd_container_of(link, I, link);
		link = link->next;
		llhd_value_unlink_uses((struct llhd_value *)I);
	}
	link = E->insts.next;
	while (link != &E->insts) {
		struct llhd_inst *I;
		I = llhd_container_of(link, I, link);
		link = link->next;
		I->parent = NULL;
		llhd_value_unref((struct llhd_value *)I);
	}
	llhd_free(E->name);
	llhd_type_unref(E->type);
	for (i = 0; i < E->super.num_inputs + E->super.num_outputs; ++i)
		llhd_value_unref((struct llhd_value *)E->super.params[i]);
}

static void
entity_add_inst(void *ptr, struct llhd_value *I, int append) {
	assert(llhd_value_is(I, LLHD_VALUE_INST));
	struct llhd_entity *E = ptr;
	llhd_value_ref(I);
	llhd_list_insert(append ? E->insts.prev : &E->insts, &((struct llhd_inst *)I)->link);
}

static void
entity_remove_inst(void *ptr, struct llhd_value *I) {
	assert(llhd_value_is(I, LLHD_VALUE_INST));
	llhd_list_remove(&((struct llhd_inst *)I)->link);
	llhd_value_unref(I);
}

struct llhd_value *
llhd_proc_new(struct llhd_type *T, const char *name) {
	struct llhd_proc *P;
	unsigned i, num_inputs, num_outputs;
	assert(T && name && llhd_type_is(T, LLHD_TYPE_COMP));
	llhd_type_ref(T);
	num_inputs = llhd_type_get_num_inputs(T);
	num_outputs = llhd_type_get_num_outputs(T);
	P = llhd_alloc_unit(sizeof(*P), &vtbl_proc, num_inputs+num_outputs);
	P->name = strdup(name);
	P->type = T;
	P->super.num_inputs = num_inputs;
	P->super.num_outputs = num_outputs;
	for (i = 0; i < num_inputs; ++i)
		P->super.params[i] = param_new(llhd_type_get_input(T,i));
	for (i = 0; i < num_outputs; ++i)
		P->super.params[i+num_inputs] = param_new(llhd_type_get_output(T,i));
	llhd_list_init(&P->blocks);
	return (struct llhd_value *)P;
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

void
llhd_value_set_name(struct llhd_value *V, const char *name) {
	assert(V && V->vtbl);
	size_t off = V->vtbl->name_offset;
	assert(off);
	char **ptr = (void*)V+off;
	if (*ptr)
		llhd_free(*ptr);
	*ptr = name ? strdup(name) : NULL;
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
	return llhd_value_is(V, kind);
}

int
llhd_unit_get_kind(struct llhd_value *V) {
	return llhd_value_get_kind(V);
}

bool
llhd_unit_is_def(struct llhd_value *V) {
	int k = llhd_value_get_kind(V);
	assert(LLHD_ISA(k, LLHD_VALUE_UNIT));
	return LLHD_ISA(k, LLHD_UNIT_DEF_FUNC) ||
	       LLHD_ISA(k, LLHD_UNIT_DEF_ENTITY) ||
	       LLHD_ISA(k, LLHD_UNIT_DEF_PROC);
}

bool
llhd_unit_is_decl(struct llhd_value *V) {
	int k = llhd_value_get_kind(V);
	assert(LLHD_ISA(k, LLHD_VALUE_UNIT));
	return LLHD_ISA(k, LLHD_UNIT_DECL);
}

struct llhd_value *
llhd_entity_get_first_inst(struct llhd_value *V) {
	struct llhd_entity *E = (void*)V;
	assert(llhd_value_is(V, LLHD_UNIT_DEF_ENTITY));
	if (E->insts.next == &E->insts)
		return NULL;
	return (struct llhd_value*)llhd_container_of2(E->insts.next, struct llhd_inst, link);
}

struct llhd_value *
llhd_entity_get_last_inst(struct llhd_value *V) {
	struct llhd_entity *E = (void*)V;
	assert(llhd_value_is(V, LLHD_UNIT_DEF_ENTITY));
	if (E->insts.prev == &E->insts)
		return NULL;
	return (struct llhd_value*)llhd_container_of2(E->insts.prev, struct llhd_inst, link);
}

unsigned
llhd_entity_get_num_insts(struct llhd_value *V) {
	struct llhd_entity *E = (void*)V;
	assert(llhd_value_is(V, LLHD_UNIT_DEF_ENTITY));
	return llhd_list_length(&E->insts);
}

unsigned
llhd_unit_get_num_inputs(struct llhd_value *V) {
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	return ((struct llhd_unit*)V)->num_inputs;
}

unsigned
llhd_unit_get_num_outputs(struct llhd_value *V) {
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	return ((struct llhd_unit*)V)->num_outputs;
}

struct llhd_value *
llhd_unit_get_input(struct llhd_value *V, unsigned idx) {
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	struct llhd_unit *U = (void*)V;
	assert(idx < U->num_inputs);
	return (struct llhd_value*)U->params[idx];
}

struct llhd_value *
llhd_unit_get_output(struct llhd_value *V, unsigned idx) {
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
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

void
llhd_value_unlink_uses(struct llhd_value *V) {
	assert(V && V->vtbl);
	if (V->vtbl->unlink_uses_fn)
		V->vtbl->unlink_uses_fn(V);
}

void
llhd_value_unlink_from_parent(struct llhd_value *V) {
	assert(V && V->vtbl);
	if (V->vtbl->unlink_from_parent_fn)
		V->vtbl->unlink_from_parent_fn(V);
}

static void
proc_dispose(void *ptr) {
	unsigned i;
	struct llhd_proc *P;
	struct llhd_list *pos;
	struct llhd_value *BB;
	assert(ptr);
	P = ptr;
	pos = llhd_block_first(&P->blocks);
	while ((BB = llhd_block_next(&P->blocks, &pos))) {
		llhd_value_unlink(BB);
	}
	llhd_free(P->name);
	llhd_type_unref(P->type);
	for (i = 0; i < P->super.num_inputs + P->super.num_outputs; ++i)
		llhd_value_unref((struct llhd_value *)P->super.params[i]);
}

static void
proc_add_block(void *ptr, struct llhd_block *BB, int append) {
	struct llhd_proc *P = ptr;
	assert(BB);
	llhd_value_ref((struct llhd_value *)BB);
	llhd_list_insert(append ? P->blocks.prev : &P->blocks, &BB->link);
}

static void
proc_remove_block(void *ptr, struct llhd_block *BB) {
	assert(BB);
	llhd_list_remove(&BB->link);
	llhd_value_unref((struct llhd_value *)BB);
}

static void
proc_substitute(void *ptr, void *ref, void *sub) {
	struct llhd_proc *P = ptr;
	struct llhd_list *link;

	link = P->blocks.next;
	while (link != &P->blocks) {
		void *BB = llhd_container_of2(link, struct llhd_block, link);
		link = link->next;
		llhd_value_substitute(BB, ref, sub);
	}
}

static void
unit_unlink_from_parent(void *ptr) {
	struct llhd_unit *U = ptr;
	assert(U->module);
	llhd_list_remove(&U->link);
	U->module = NULL;
	llhd_value_unref(ptr);
}

struct llhd_value *
llhd_block_new(const char *name) {
	struct llhd_block *B;
	assert(name);
	B = llhd_alloc_value(sizeof(*B), &vtbl_block);
	B->name = strdup(name);
	B->type = llhd_type_new_label();
	llhd_list_init(&B->insts);
	return (struct llhd_value *)B;
}

void
llhd_block_append_to(struct llhd_value *V, struct llhd_value *to) {
	struct llhd_block *BB = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	assert(!BB->parent);
	assert(to && to->vtbl && to->vtbl->add_block_fn);
	BB->parent = to;
	to->vtbl->add_block_fn(to,BB,1);
}

void
llhd_block_prepend_to(struct llhd_value *V, struct llhd_value *to) {
	struct llhd_block *BB = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	assert(!BB->parent);
	assert(to && to->vtbl && to->vtbl->add_block_fn);
	BB->parent = to;
	to->vtbl->add_block_fn(to,BB,0);
}

void
llhd_block_insert_after(struct llhd_value *V, struct llhd_value *Vpos) {
	struct llhd_block *BB = (void*)V, *BBpos = (void*)Vpos;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	assert(llhd_value_is(Vpos, LLHD_VALUE_BLOCK));
	assert(!BB->parent);
	BB->parent = BBpos->parent;
	llhd_list_insert(&BBpos->link, &BB->link);
}

void
llhd_block_insert_before(struct llhd_value *V, struct llhd_value *Vpos) {
	struct llhd_block *BB = (void*)V, *BBpos = (void*)Vpos;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	assert(llhd_value_is(Vpos, LLHD_VALUE_BLOCK));
	assert(!BB->parent);
	BB->parent = BBpos->parent;
	llhd_list_insert(BBpos->link.prev, &BB->link);
}

static void
block_add_inst(void *ptr, struct llhd_value *I, int append) {
	struct llhd_block *BB = ptr;
	assert(llhd_value_is(I, LLHD_VALUE_INST));
	llhd_value_ref(I);
	llhd_list_insert(append ? BB->insts.prev : &BB->insts, &((struct llhd_inst *)I)->link);
}

static void
block_remove_inst(void *ptr, struct llhd_value *I) {
	assert(llhd_value_is(I, LLHD_VALUE_INST));
	llhd_list_remove(&((struct llhd_inst *)I)->link);
	llhd_value_unref(I);
}

static void
block_dispose(void *ptr) {
	struct llhd_block *BB = ptr;
	struct llhd_list *link;

	link = BB->insts.next;
	while (link != &BB->insts) {
		struct llhd_inst *I;
		I = llhd_container_of(link, I, link);
		link = link->next;
		llhd_value_unlink_uses((struct llhd_value *)I);
	}
	link = BB->insts.next;
	while (link != &BB->insts) {
		struct llhd_inst *I;
		I = llhd_container_of(link, I, link);
		link = link->next;
		I->parent = NULL;
		llhd_value_unref((struct llhd_value *)I);
	}
	llhd_free(BB->name);
	llhd_type_unref(BB->type);
}

static void
block_unlink_from_parent(void *ptr) {
	struct llhd_block *BB = ptr;
	struct llhd_value *P = BB->parent;
	assert(P && P->vtbl);
	// Must go before remove_block_fn, since that might dispose and free the
	// block, which triggers an assert on parent == NULL in the dispose function.
	BB->parent = NULL;
	if (P->vtbl->remove_block_fn)
		P->vtbl->remove_block_fn(P, ptr);
}

struct llhd_list *
llhd_block_first(struct llhd_list *head) {
	assert(head);
	return head->next;
}

struct llhd_list *
llhd_block_last(struct llhd_list *head) {
	assert(head);
	return head->prev;
}

struct llhd_value *
llhd_block_next(struct llhd_list *head, struct llhd_list **pos) {
	assert(head && pos);
	if (*pos != head) {
		void *ptr = llhd_container_of2(*pos, struct llhd_block, link);
		*pos = (*pos)->next;
		return ptr;
	} else {
		return NULL;
	}
}

struct llhd_value *
llhd_block_prev(struct llhd_list *head, struct llhd_list **pos) {
	assert(head && pos);
	if (*pos != head) {
		void *ptr = llhd_container_of2(*pos, struct llhd_block, link);
		*pos = (*pos)->prev;
		return ptr;
	} else {
		return NULL;
	}
}

struct llhd_value *
llhd_block_get_first_inst(struct llhd_value *V) {
	struct llhd_block *BB = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	if (BB->insts.next == &BB->insts)
		return NULL;
	return (struct llhd_value *)llhd_container_of2(BB->insts.next, struct llhd_inst, link);
}

struct llhd_value *
llhd_block_get_last_inst(struct llhd_value *V) {
	struct llhd_block *BB = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_BLOCK));
	if (BB->insts.prev == &BB->insts)
		return NULL;
	return (struct llhd_value *)llhd_container_of2(BB->insts.prev, struct llhd_inst, link);
}

struct llhd_list *
llhd_unit_get_blocks(struct llhd_value *V) {
	struct llhd_unit_vtbl *vtbl;
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	vtbl = (struct llhd_unit_vtbl *)V->vtbl;
	assert(vtbl->block_list_offset);
	return (void*)V + vtbl->block_list_offset;
}

struct llhd_list *
llhd_unit_first(struct llhd_list *head) {
	assert(head);
	return head->next;
}

struct llhd_list *
llhd_unit_last(struct llhd_list *head) {
	assert(head);
	return head->prev;
}

struct llhd_value *
llhd_unit_next(struct llhd_list *head, struct llhd_list **pos) {
	assert(head && pos);
	if (*pos != head) {
		void *ptr = llhd_container_of2(*pos, struct llhd_unit, link);
		*pos = (*pos)->next;
		return ptr;
	} else {
		return NULL;
	}
}

struct llhd_value *
llhd_unit_prev(struct llhd_list *head, struct llhd_list **pos) {
	assert(head && pos);
	if (*pos != head) {
		void *ptr = llhd_container_of2(*pos, struct llhd_unit, link);
		*pos = (*pos)->prev;
		return ptr;
	} else {
		return NULL;
	}
}

void
llhd_unit_append_to(struct llhd_value *V, struct llhd_module *M) {
	struct llhd_unit *U = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	assert(M);
	assert(!U->module);
	U->module = M;
	llhd_value_ref(V);
	llhd_list_insert(M->units.prev, &U->link);
}

void
llhd_unit_prepend_to(struct llhd_value *V, struct llhd_module *M) {
	struct llhd_unit *U = (void*)V;
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	assert(M);
	assert(!U->module);
	U->module = M;
	llhd_value_ref(V);
	llhd_list_insert(&M->units, &U->link);
}

void
llhd_unit_insert_after(struct llhd_value *V, struct llhd_value *Vother) {
	struct llhd_unit *U = (void*)V, *other = (void*)Vother;
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	assert(llhd_value_is(Vother, LLHD_VALUE_UNIT));
	assert(!U->module);
	U->module = other->module;
	llhd_value_ref(V);
	llhd_list_insert(&other->link, &U->link);
}

void
llhd_unit_insert_before(struct llhd_value *V, struct llhd_value *Vother) {
	struct llhd_unit *U = (void*)V, *other = (void*)Vother;
	assert(llhd_value_is(V, LLHD_VALUE_UNIT));
	assert(llhd_value_is(Vother, LLHD_VALUE_UNIT));
	assert(!U->module);
	U->module = other->module;
	llhd_value_ref(V);
	llhd_list_insert(other->link.prev, &U->link);
}

struct llhd_value *
llhd_func_new(struct llhd_type *T, const char *name) {
	struct llhd_func *F;
	unsigned i, num_inputs, num_outputs;
	assert(T && name && llhd_type_is(T, LLHD_TYPE_FUNC));
	llhd_type_ref(T);
	num_inputs = llhd_type_get_num_inputs(T);
	num_outputs = llhd_type_get_num_outputs(T);
	F = llhd_alloc_unit(sizeof(*F), &vtbl_func, num_inputs+num_outputs);
	F->name = strdup(name);
	F->type = T;
	F->super.num_inputs = num_inputs;
	F->super.num_outputs = num_outputs;
	for (i = 0; i < num_inputs; ++i)
		F->super.params[i] = param_new(llhd_type_get_input(T,i));
	for (i = 0; i < num_outputs; ++i)
		F->super.params[i+num_inputs] = param_new(llhd_type_get_output(T,i));
	llhd_list_init(&F->blocks);
	return (struct llhd_value *)F;
}

static void
func_dispose(void *ptr) {
	unsigned i;
	struct llhd_func *F;
	struct llhd_list *pos;
	struct llhd_value *BB;
	assert(ptr);
	F = ptr;
	pos = llhd_block_first(&F->blocks);
	while ((BB = llhd_block_next(&F->blocks, &pos))) {
		llhd_value_unlink(BB);
	}
	llhd_free(F->name);
	llhd_type_unref(F->type);
	for (i = 0; i < F->super.num_inputs + F->super.num_outputs; ++i)
		llhd_value_unref((struct llhd_value *)F->super.params[i]);
}

static void
func_add_block(void *ptr, struct llhd_block *BB, int append) {
	struct llhd_func *F = ptr;
	assert(BB);
	llhd_value_ref((struct llhd_value *)BB);
	llhd_list_insert(append ? F->blocks.prev : &F->blocks, &BB->link);
}

static void
func_remove_block(void *ptr, struct llhd_block *BB) {
	assert(BB);
	llhd_list_remove(&BB->link);
	llhd_value_unref((struct llhd_value *)BB);
}

struct llhd_value *
llhd_value_copy(struct llhd_value *V) {
	assert(V && V->vtbl && V->vtbl->copy_fn);
	return V->vtbl->copy_fn(V);
}

static void
block_substitute(void *ptr, void *ref, void *sub) {
	struct llhd_block *BB = ptr;
	struct llhd_list *link;

	link = BB->insts.next;
	while (link != &BB->insts) {
		void *I = llhd_container_of2(link, struct llhd_inst, link);
		link = link->next;
		llhd_value_substitute(I, ref, sub);
	}
}
