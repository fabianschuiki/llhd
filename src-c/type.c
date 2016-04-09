// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>
#include <string.h>

struct llhd_type {
	int kind;
	int rc;
	unsigned num_in;
	unsigned num_out;
	struct llhd_type *subtypes[];
};

void *
llhd_alloc_type(int kind, unsigned num_subtypes) {
	struct llhd_type *T;
	T = llhd_zalloc(sizeof(*T) + sizeof(T)*num_subtypes);
	T->rc = 1;
	T->kind = kind;
	return T;
}

struct llhd_type *
llhd_type_new_comp(
	struct llhd_type **in,
	unsigned num_in,
	struct llhd_type **out,
	unsigned num_out
) {
	unsigned i;
	struct llhd_type *T;
	assert(!num_in || in);
	assert(!num_out || out);
	T = llhd_alloc_type(LLHD_TYPE_COMP, num_in+num_out);
	T->num_in = num_in;
	T->num_out = num_out;
	memcpy(T->subtypes, in, num_in*sizeof(T));
	memcpy(T->subtypes + num_in, out, num_out*sizeof(T));
	for (i = 0; i < num_in+num_out; ++i)
		llhd_type_ref(T->subtypes[i]);
	return T;
}

struct llhd_type *
llhd_type_new_int(unsigned bits) {
	assert(bits > 0);
	struct llhd_type *T;
	T = llhd_alloc_type(LLHD_TYPE_INT,0);
	T->num_in = bits;
	return T;
}

void
llhd_type_ref(struct llhd_type *T) {
	assert(T->rc > 0);
	++T->rc;
}

void
llhd_type_unref(struct llhd_type *T) {
	assert(T->rc > 0);
	if (--T->rc == 0) {
		unsigned i;
		// Dispose of the subtypes.
		switch (T->kind) {
			case LLHD_TYPE_COMP:
			case LLHD_TYPE_FUNC:
				for (i = 0; i < T->num_in + T->num_out; ++i)
					llhd_type_unref(T->subtypes[i]);
				break;
			case LLHD_TYPE_ARRAY:
			case LLHD_TYPE_PTR:
			case LLHD_TYPE_SIGNAL:
				llhd_type_unref(T->subtypes[0]);
				break;
			case LLHD_TYPE_STRUCT:
				for (i = 0; i < T->num_in; ++i)
					llhd_type_unref(T->subtypes[i]);
				break;
		}
		llhd_free(T);
	}
}

int
llhd_type_get_kind(struct llhd_type *T) {
	return T->kind;
}

unsigned
llhd_type_get_length(struct llhd_type *T) {
	assert(T->kind == LLHD_TYPE_INT ||
	       T->kind == LLHD_TYPE_LOGIC ||
	       T->kind == LLHD_TYPE_ARRAY);
	return T->num_in;
}

struct llhd_type *
llhd_type_get_subtype(struct llhd_type *T) {
	assert(T->kind == LLHD_TYPE_ARRAY ||
	       T->kind == LLHD_TYPE_PTR ||
	       T->kind == LLHD_TYPE_SIGNAL);
	return T->subtypes[0];
}

unsigned
llhd_type_get_num_fields(struct llhd_type *T) {
	assert(T->kind == LLHD_TYPE_STRUCT);
	return T->num_in;
}

struct llhd_type *
llhd_type_get_field(struct llhd_type *T, unsigned idx) {
	assert(T->kind == LLHD_TYPE_STRUCT);
	assert(idx < T->num_in);
	return T->subtypes[idx];
}

unsigned
llhd_type_get_num_inputs(struct llhd_type *T) {
	assert(T->kind == LLHD_TYPE_FUNC || T->kind == LLHD_TYPE_COMP);
	return T->num_in;
}

unsigned
llhd_type_get_num_outputs(struct llhd_type *T) {
	assert(T->kind == LLHD_TYPE_FUNC || T->kind == LLHD_TYPE_COMP);
	return T->num_out;
}

struct llhd_type *
llhd_type_get_input(struct llhd_type *T, unsigned idx) {
	assert(T->kind == LLHD_TYPE_FUNC || T->kind == LLHD_TYPE_COMP);
	assert(idx < T->num_in);
	return T->subtypes[idx];
}

struct llhd_type *
llhd_type_get_output(struct llhd_type *T, unsigned idx) {
	assert(T->kind == LLHD_TYPE_FUNC || T->kind == LLHD_TYPE_COMP);
	assert(idx < T->num_out);
	return T->subtypes[T->num_in + idx];
}
