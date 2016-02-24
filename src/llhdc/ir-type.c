// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/ir.h"
#include "src/llhdc/ir-internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>


static llhd_type_t *
make_type(enum llhd_type_kind kind, unsigned num_inner) {
	unsigned size = sizeof(llhd_type_t) + sizeof(llhd_type_t*) * num_inner;
	llhd_type_t *T = malloc(size);
	memset(T, 0, size);
	T->kind = kind;
	return T;
}

llhd_type_t *
llhd_type_make_void () {
	return make_type(LLHD_VOID_TYPE, 0);
}

llhd_type_t *
llhd_type_make_label () {
	return make_type(LLHD_LABEL_TYPE, 0);
}

llhd_type_t *
llhd_type_make_time () {
	return make_type(LLHD_TIME_TYPE, 0);
}

llhd_type_t *
llhd_type_make_int (unsigned width) {
	llhd_type_t *T = make_type(LLHD_INT_TYPE, 0);
	T->length = width;
	return T;
}

llhd_type_t *
llhd_type_make_logic (unsigned width) {
	llhd_type_t *T = make_type(LLHD_LOGIC_TYPE, 0);
	T->length = width;
	return T;
}

llhd_type_t *
llhd_type_make_struct (llhd_type_t **fields, unsigned num_fields) {
	assert(!num_fields || fields);
	llhd_type_t *T = make_type(LLHD_STRUCT_TYPE, num_fields);
	T->length = num_fields;
	memcpy(T->inner, fields, sizeof(*fields) * num_fields);
	return T;
}

llhd_type_t *
llhd_type_make_array (llhd_type_t *element, unsigned length) {
	assert(element);
	llhd_type_t *T = make_type(LLHD_ARRAY_TYPE, 1);
	T->length = length;
	T->inner[0] = element;
	return T;
}

llhd_type_t *
llhd_type_make_ptr (llhd_type_t *to) {
	assert(to);
	llhd_type_t *T = make_type(LLHD_PTR_TYPE, 1);
	T->inner[0] = to;
	return T;
}

llhd_type_t *
llhd_type_copy (llhd_type_t *T) {
	assert(T);
	switch (T->kind) {
	case LLHD_VOID_TYPE: return llhd_type_make_void();
	case LLHD_LABEL_TYPE: return llhd_type_make_label();
	case LLHD_TIME_TYPE: return llhd_type_make_time();
	case LLHD_INT_TYPE: return llhd_type_make_int(T->length);
	case LLHD_LOGIC_TYPE: return llhd_type_make_logic(T->length);
	case LLHD_STRUCT_TYPE: return llhd_type_make_struct(T->inner, T->length);
	case LLHD_ARRAY_TYPE: return llhd_type_make_array(T->inner[0], T->length);
	case LLHD_PTR_TYPE: return llhd_type_make_ptr(T->inner[0]);
	}
	assert(0);
}

static void
llhd_type_dispose (llhd_type_t *T) {
	assert(T);
	unsigned i;
	switch (T->kind) {
	case LLHD_VOID_TYPE:
	case LLHD_LABEL_TYPE:
	case LLHD_TIME_TYPE:
	case LLHD_INT_TYPE:
	case LLHD_LOGIC_TYPE:
		break;
	case LLHD_STRUCT_TYPE:
		for (i = 0; i < T->length; ++i)
			llhd_type_destroy(T->inner[i]);
		break;
	case LLHD_ARRAY_TYPE:
	case LLHD_PTR_TYPE:
		llhd_type_destroy(T->inner[0]);
		break;
	}
}

void
llhd_type_destroy (llhd_type_t *T) {
	if (T) {
		llhd_type_dispose(T);
		free(T);
	}
}

void
llhd_type_dump (llhd_type_t *T, FILE *f) {
	assert(T);
	unsigned i;
	switch (T->kind) {
	case LLHD_VOID_TYPE: fputs("void", f); break;
	case LLHD_LABEL_TYPE: fputs("label", f); break;
	case LLHD_TIME_TYPE: fputs("time", f); break;
	case LLHD_INT_TYPE: fprintf(f, "i%u", T->length); break;
	case LLHD_LOGIC_TYPE: fprintf(f, "l%u", T->length); break;
	case LLHD_STRUCT_TYPE:
		fputs("{ ", f);
		for (i = 0; i < T->length; ++i) {
			if (i > 0) fputs(", ", f);
			llhd_type_dump(T->inner[i], f);
		}
		fputs(" }", f);
		break;
	case LLHD_ARRAY_TYPE:
		fprintf(f, "[%u x ", T->length);
		llhd_type_dump(T->inner[0], f);
		fputc(']', f);
		break;
	case LLHD_PTR_TYPE:
		llhd_type_dump(T->inner[0], f);
		fputc('*', f);
		break;
	}
}

int
llhd_type_equal (llhd_type_t *A, llhd_type_t *B) {
	assert(A && B);
	if (A->kind != B->kind)
		return 0;
	unsigned i;
	switch (A->kind) {
	case LLHD_VOID_TYPE:
	case LLHD_LABEL_TYPE:
	case LLHD_TIME_TYPE:
		return 1;
	case LLHD_INT_TYPE:
	case LLHD_LOGIC_TYPE:
		return A->length == B->length;
	case LLHD_STRUCT_TYPE:
		if (A->length != B->length)
			return 0;
		for (i = 0; i < A->length; ++i)
			if (!llhd_type_equal(A->inner[i], B->inner[i]))
				return 0;
		return 1;
	case LLHD_ARRAY_TYPE:
		if (A->length != B->length)
			return 0;
	case LLHD_PTR_TYPE:
		return llhd_type_equal(A->inner[0], B->inner[0]);
	}
	assert(0);
}

int
llhd_type_is (llhd_type_t *T, llhd_type_kind_t kind) {
	return T->kind == kind;
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
llhd_type_is_time (llhd_type_t *T) {
	return T->kind == LLHD_TIME_TYPE;
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

llhd_type_kind_t
llhd_type_get_kind (llhd_type_t *T) {
	return T->kind;
}


unsigned
llhd_type_scalar_get_width (llhd_type_t *T) {
	assert(T && (T->kind == LLHD_INT_TYPE || T->kind == LLHD_LOGIC_TYPE));
	return T->length;
}

unsigned
llhd_type_struct_get_num_fields (llhd_type_t *T) {
	assert(T && T->kind == LLHD_STRUCT_TYPE);
	return T->length;
}

llhd_type_t **
llhd_type_struct_get_fields (llhd_type_t *T) {
	assert(T && T->kind == LLHD_STRUCT_TYPE);
	return T->inner;
}

llhd_type_t *
llhd_type_struct_get_field (llhd_type_t *T, unsigned index) {
	assert(T && T->kind == LLHD_STRUCT_TYPE);
	assert(index < T->length);
	return T->inner[index];
}

unsigned
llhd_type_array_get_length (llhd_type_t *T) {
	assert(T && T->kind == LLHD_ARRAY_TYPE);
	return T->length;
}

llhd_type_t *
llhd_type_array_get_subtype (llhd_type_t *T) {
	assert(T && T->kind == LLHD_ARRAY_TYPE);
	return T->inner[0];
}

llhd_type_t *
llhd_type_ptr_get_subtype (llhd_type_t *T) {
	assert(T && T->kind == LLHD_PTR_TYPE);
	return T->inner[0];
}
