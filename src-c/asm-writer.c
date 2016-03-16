// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>
#include <stdio.h>

typedef struct llhd_type * llhd_type_t;
int llhd_unit_get_kind(llhd_value_t);
llhd_type_t llhd_value_get_type(llhd_value_t);
unsigned llhd_type_get_scalar_width(llhd_type_t);
unsigned llhd_type_get_array_length(llhd_type_t);
llhd_type_t llhd_type_get_subtype(llhd_type_t);
int llhd_type_get_kind(llhd_type_t);

enum llhd_unit_kind {
	LLHD_UNIT_DECL = 1,
	LLHD_UNIT_DEF_FUNC = 2,
	LLHD_UNIT_DEF_ENTITY = 3,
	LLHD_UNIT_DEF_PROC = 4,
};

enum llhd_type_kind {
	LLHD_TYPE_VOID   = 0x00,
	LLHD_TYPE_LABEL  = 0x01,
	LLHD_TYPE_TIME   = 0x02,
	LLHD_TYPE_INT    = 0x10,
	LLHD_TYPE_LOGIC  = 0x11,
	LLHD_TYPE_STRUCT = 0x20,
	LLHD_TYPE_ARRAY  = 0x21,
	LLHD_TYPE_PTR    = 0x30,
	LLHD_TYPE_SIGNAL = 0x31,
};

static void
write_type (llhd_type_t T, FILE *out) {
	int kind = llhd_type_get_kind(T);
	switch (kind) {
		case LLHD_TYPE_VOID:   fputs("void", out); break;
		case LLHD_TYPE_LABEL:  fputs("label", out); break;
		case LLHD_TYPE_TIME:   fputs("time", out); break;
		case LLHD_TYPE_INT:    fprintf(out, "i%d", llhd_type_get_scalar_width(T)); break;
		case LLHD_TYPE_LOGIC:  fprintf(out, "l%d", llhd_type_get_scalar_width(T)); break;
		case LLHD_TYPE_STRUCT:
			fputc('{', out);
			fputc('}', out);
			break;
		case LLHD_TYPE_ARRAY:
			fprintf(out, "[%d x ", llhd_type_get_array_length(T));
			write_type(llhd_type_get_subtype(T), out);
			fputc(']', out);
			break;
		case LLHD_TYPE_PTR:
			write_type(llhd_type_get_subtype(T), out);
			fputc('*', out);
			break;
		case LLHD_TYPE_SIGNAL:
			write_type(llhd_type_get_subtype(T), out);
			fputc('$', out);
			break;
		default:
			assert(0 && "unsupported type kind");
	}
}

static void
write_decl (llhd_value_t D, FILE *out) {
	fprintf(out, "declare @%s ", llhd_value_get_name(D));
	write_type(llhd_value_get_type(D), out);
	fputc('\n', out);
}

void
llhd_asm_write_unit (llhd_value_t U, FILE *out) {
	int kind = llhd_unit_get_kind(U);
	switch (kind) {
		case LLHD_UNIT_DECL:
			write_decl(U, out);
			break;
		case LLHD_UNIT_DEF_FUNC:
		case LLHD_UNIT_DEF_ENTITY:
		case LLHD_UNIT_DEF_PROC:
		default:
			assert(0 && "unsupported unit kind");
	}
}

void
llhd_asm_write_module (llhd_module_t M, FILE *out) {
	fprintf(out, "; module '%s'\n", llhd_module_get_name(M));
	llhd_value_t U;
	for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
		fputc('\n', out);
		llhd_asm_write_unit(U, out);
	}
}
