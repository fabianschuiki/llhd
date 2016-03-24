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

struct llhd_type *
llhd_type_new_comp(
	const struct llhd_type *in,
	unsigned num_in,
	const struct llhd_type *out,
	unsigned num_out
) {
	assert(!num_in || in);
	assert(!num_out || out);
	struct llhd_type *T;
	T = llhd_zalloc(sizeof(*T) + (num_in+num_out)*sizeof(T));
	T->rc = 1;
	T->kind = LLHD_TYPE_COMP;
	T->num_in = num_in;
	T->num_out = num_out;
	memcpy(T->subtypes, in, num_in*sizeof(T));
	memcpy(T->subtypes + num_in, out, num_out*sizeof(T));
	/// @todo Call ref on all subtypes.
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
		/// @todo Make sure all subtypes are release.
		llhd_free(T);
	}
}
