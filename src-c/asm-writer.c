// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static void
write_type (llhd_type_t T, FILE *out) {
	unsigned i,N;
	int kind = llhd_type_get_kind(T);
	switch (kind) {
		case LLHD_TYPE_VOID:   fputs("void", out); break;
		case LLHD_TYPE_LABEL:  fputs("label", out); break;
		case LLHD_TYPE_TIME:   fputs("time", out); break;
		case LLHD_TYPE_INT:    fprintf(out, "i%d", llhd_type_get_length(T)); break;
		case LLHD_TYPE_LOGIC:  fprintf(out, "l%d", llhd_type_get_length(T)); break;
		case LLHD_TYPE_STRUCT:
			fputc('{', out);
			N = llhd_type_get_num_fields(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				write_type(llhd_type_get_field(T,i), out);
			}
			fputc('}', out);
			break;
		case LLHD_TYPE_ARRAY:
			fprintf(out, "[%d x ", llhd_type_get_length(T));
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
		case LLHD_TYPE_FUNC:
			fputs("func(", out);
			N = llhd_type_get_num_inputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				write_type(llhd_type_get_input(T,i), out);
			}
			fputs(")(", out);
			N = llhd_type_get_num_outputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				write_type(llhd_type_get_output(T,i), out);
			}
			fputs(")", out);
			break;
		case LLHD_TYPE_COMP:
			fputs("comp(", out);
			N = llhd_type_get_num_inputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				write_type(llhd_type_get_input(T,i), out);
			}
			fputs(")(", out);
			N = llhd_type_get_num_outputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				write_type(llhd_type_get_output(T,i), out);
			}
			fputs(")", out);
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

struct llhd_symtbl {
	struct llhd_symtbl_entry *entries_by_name;
	struct llhd_symtbl_entry *entries_by_sym;
	unsigned size;
	unsigned alloc;
	unsigned tmp_index;
};

struct llhd_symtbl_entry {
	char *name;
	void *sym;
};

static int
cmp_symtbl_name_qsort (const void *va, const void *vb) {
	return strcmp(
		((struct llhd_symtbl_entry*)va)->name,
		((struct llhd_symtbl_entry*)vb)->name
	);
}

static int
cmp_symtbl_sym_qsort (const void *va, const void *vb) {
	const struct llhd_symtbl_entry *a = va, *b = vb;
	if (a->sym < b->sym) return -1;
	if (a->sym > b->sym) return 1;
	return 0;
}

static int
cmp_symtbl_name_bsearch (const void *ka, const void *vb) {
	return strcmp(
		ka,
		((struct llhd_symtbl_entry*)vb)->name
	);
}

static int
cmp_symtbl_sym_bsearch (const void *ka, const void *vb) {
	void *b = ((struct llhd_symtbl_entry*)vb)->sym;
	if (ka < b) return -1;
	if (ka > b) return 1;
	return 0;
}

static struct llhd_symtbl *
symtbl_new () {
	struct llhd_symtbl *tbl = llhd_zalloc(sizeof(*tbl));
	tbl->alloc = 16;
	tbl->entries_by_name = llhd_alloc(tbl->alloc * sizeof(*tbl));
	tbl->entries_by_sym  = llhd_alloc(tbl->alloc * sizeof(*tbl));
	return tbl;
}

static void
symtbl_free (struct llhd_symtbl *tbl) {
	unsigned i;
	for (i = 0; i < tbl->size; ++i) {
		llhd_free(tbl->entries_by_name[i].name);
	}
	llhd_free(tbl->entries_by_name);
	llhd_free(tbl->entries_by_sym);
	llhd_free(tbl);
}

static void *
symtbl_lookup_name (struct llhd_symtbl *tbl, const char *name) {
	struct llhd_symtbl_entry *e = bsearch(
		name,
		tbl->entries_by_name,
		tbl->size,
		sizeof(struct llhd_symtbl_entry),
		cmp_symtbl_name_bsearch
	);
	return e ? e->sym : 0;
}

static const char *
symtbl_lookup_sym (struct llhd_symtbl *tbl, const void *sym) {
	struct llhd_symtbl_entry *e = bsearch(
		sym,
		tbl->entries_by_sym,
		tbl->size,
		sizeof(struct llhd_symtbl_entry),
		cmp_symtbl_sym_bsearch
	);
	return e ? e->name : 0;
}

/// Adds the symbol sym to the table under the given name. If the name is
/// already taken, it will be uniquified by appending a number. If the name is
/// omitted, an anonymous temporary name will be used. In any case, the
/// resulting name shall be returned.
static const char *
symtbl_add_name (struct llhd_symtbl *tbl, void *sym, const char *name) {
	char *actual_name;

	if (name) {
		unsigned name_len = strlen(name);
		char tmp_name[name_len+10];
		tmp_name[0] = 0;

		void *found = symtbl_lookup_name(tbl, name);
		while (found) {
			snprintf(tmp_name, name_len+10, "%s%d", name, tbl->tmp_index);
			++tbl->tmp_index;
			found = symtbl_lookup_name(tbl, tmp_name);
		}
		if (*tmp_name) {
			unsigned len = strlen(tmp_name);
			actual_name = llhd_alloc(len+1);
			memcpy(actual_name, tmp_name, len+1);
		} else {
			actual_name = llhd_alloc(name_len+1);
			memcpy(actual_name, name, name_len+1);
		}
	} else {
		char tmp_name[16] = {0};
		void *found;
		do {
			snprintf(tmp_name, 16, "%d", tbl->tmp_index);
			++tbl->tmp_index;
			found = symtbl_lookup_name(tbl, tmp_name);
		} while (found);

		unsigned len = strlen(tmp_name);
		actual_name = llhd_alloc(len+1);
		memcpy(actual_name, tmp_name, len+1);
	}

	if (tbl->size == tbl->alloc) {
		tbl->alloc *= 2;
		tbl->entries_by_name = llhd_realloc(
			tbl->entries_by_name,
			tbl->alloc * sizeof(struct llhd_symtbl_entry)
		);
		tbl->entries_by_sym = llhd_realloc(
			tbl->entries_by_sym,
			tbl->alloc * sizeof(struct llhd_symtbl_entry)
		);
	}
	tbl->entries_by_name[tbl->size].name = actual_name;
	tbl->entries_by_name[tbl->size].sym = sym;
	tbl->entries_by_sym[tbl->size].name = actual_name;
	tbl->entries_by_sym[tbl->size].sym = sym;
	++tbl->size;

	qsort(tbl->entries_by_name, tbl->size, sizeof(struct llhd_symtbl_entry), cmp_symtbl_name_qsort);
	qsort(tbl->entries_by_sym, tbl->size, sizeof(struct llhd_symtbl_entry), cmp_symtbl_sym_qsort);

	return actual_name;
}

static void
write_param (llhd_value_t P, struct llhd_symtbl *symtbl, FILE *out) {
	write_type(llhd_value_get_type(P), out);
	const char *name = llhd_value_get_name(P);
	if (name || llhd_value_has_users(P)) {
		name = symtbl_add_name(symtbl, P, name);
	}
	if (name) {
		fputs(" %", out);
		fputs(name, out);
	}
}

static void
write_value_ref(llhd_value_t V, int withType, struct llhd_symtbl *symtbl, FILE *out) {
	// print local values as %<name>
	// print proc, func, entity names as @<name>
	// print globals as @<name>
	// print scalar constants as <ty> <value>

	if (withType) {
		llhd_type_t T = llhd_value_get_type(V);
		write_type(T, out);
		fputc(' ', out);
	}

	if (llhd_value_is(V,LLHD_VALUE_CONST)) {
		char *v = llhd_const_to_string(V);
		fputs(v, out);
		llhd_free(v);
	}
	else if (llhd_value_is(V,LLHD_VALUE_UNIT)) {
		fputc('@', out);
		fputs(llhd_value_get_name(V), out);
	}
	else {
		fputc('%', out);
		fputs(symtbl_lookup_sym(symtbl, V), out);
	}
}

static void
write_inst(llhd_value_t I, struct llhd_symtbl *symtbl, FILE *out) {
	int kind = llhd_inst_get_kind(I);
	const char *name = llhd_value_get_name(I);
	if (name || llhd_value_has_users(I)) {
		const char *an = symtbl_add_name(symtbl, I, name);
		fputc('%', out);
		fputs(an, out);
		fputs(" = ", out);
	}
	switch (kind) {
		case LLHD_INST_BRANCH:
			// fputs("br ", out);
			// llhd_value_t cond = llhd_inst_branch_get_condition(I);
			// if (cond) {
			// 	llhd_value_t dst0 = llhd_inst_branch_get_dst0(I);
			// 	llhd_value_t dst1 = llhd_inst_branch_get_dst1(I);
			// 	write_value_ref(cond, 1, symtbl, out);
			// 	fputs(", ", out);
			// 	write_value_ref(dst1, 1, symtbl, out);
			// 	fputs(", ", out);
			// 	write_value_ref(dst0, 1, symtbl, out);
			// } else {
			// 	llhd_value_t dst = llhd_inst_branch_get_dst(I);
			// 	write_value_ref(dst, 1, symtbl, out);
			// }
			// break;
			assert(false && "write branch not implemented");
		case LLHD_INST_UNARY:
			assert(false && "write unary not implemented");
			break;
		case LLHD_INST_BINARY:
			// fprintf(out, "bin%d ", llhd_inst_binary_get_op(I));
			fputs(llhd_inst_binary_get_opname(I), out);
			fputc(' ', out);
			write_type(llhd_value_get_type(I), out);
			fputc(' ', out);
			write_value_ref(llhd_inst_binary_get_lhs(I), 0, symtbl, out);
			fputc(' ', out);
			write_value_ref(llhd_inst_binary_get_rhs(I), 0, symtbl, out);
			break;
		default:
			assert(0 && "unknown inst kind");
	}
	fprintf(out, "  ; [#users = %d]", llhd_value_get_num_users(I));
}

static void
write_insts(llhd_value_t I, struct llhd_symtbl *symtbl, FILE *out) {
	for (; I; I = llhd_inst_next(I)) {
		fputc('\t', out);
		write_inst(I, symtbl, out);
		fputc('\n', out);
	}
}

// static void
// write_blocks(llhd_value_t BB, struct llhd_symtbl *symtbl, FILE *out) {
// 	for (; BB; BB = llhd_block_next(BB)) {
// 		fputs(llhd_value_get_name(BB), out);
// 		fputs(":\n", out);
// 		write_insts(llhd_block_get_first_inst(BB), symtbl, out);
// 	}
// }

static void
write_unit_params (llhd_value_t U, struct llhd_symtbl *symtbl, FILE *out) {
	unsigned i,N;
	fputc('(', out);
	N = llhd_unit_get_num_inputs(U);
	for (i = 0; i < N; ++i) {
		if (i > 0) fputs(", ", out);
		write_param(llhd_unit_get_input(U,i), symtbl, out);
	}
	fputs(") (", out);
	N = llhd_unit_get_num_outputs(U);
	for (i = 0; i < N; ++i) {
		if (i > 0) fputs(", ", out);
		write_param(llhd_unit_get_output(U,i), symtbl, out);
	}
	fputc(')', out);
}

// static void
// write_func_def (llhd_value_t D, FILE *out) {
// 	fprintf(out, "func @%s ", llhd_value_get_name(D));
// 	struct llhd_symtbl *symtbl = symtbl_new();
// 	write_unit_params(D, symtbl, out);
// 	fputs(" {\n", out);
// 	write_blocks(llhd_unit_get_first_block(D), symtbl, out);
// 	fputs("}\n", out);
// 	symtbl_free(symtbl);
// }

static void
write_entity_def (llhd_value_t D, FILE *out) {
	fprintf(out, "entity @%s ", llhd_value_get_name(D));
	struct llhd_symtbl *symtbl = symtbl_new();
	write_unit_params(D, symtbl, out);
	fputs(" {\n", out);
	write_insts(llhd_entity_get_first_inst(D), symtbl, out);
	fputs("}\n", out);
	symtbl_free(symtbl);
}

// static void
// write_proc_def (llhd_value_t D, FILE *out) {
// 	fprintf(out, "proc @%s ", llhd_value_get_name(D));
// 	struct llhd_symtbl *symtbl = symtbl_new();
// 	write_unit_params(D, symtbl, out);
// 	fputs(" {\n", out);
// 	write_blocks(llhd_unit_get_first_block(D), symtbl, out);
// 	fputs("}\n", out);
// 	symtbl_free(symtbl);
// }

void
llhd_asm_write_unit (llhd_value_t U, FILE *out) {
	int kind = llhd_unit_get_kind(U);
	switch (kind) {
		case LLHD_UNIT_DECL: write_decl(U, out); break;
		// case LLHD_UNIT_DEF_FUNC: write_func_def(U, out); break;
		case LLHD_UNIT_DEF_ENTITY: write_entity_def(U, out); break;
		// case LLHD_UNIT_DEF_PROC: write_proc_def(U, out); break;
		default:
			assert(0 && "unsupported unit kind");
	}
}

// void
// llhd_asm_write_module (llhd_module_t M, FILE *out) {
// 	fprintf(out, "; module '%s'\n", llhd_module_get_name(M));
// 	llhd_value_t U;
// 	for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
// 		fputc('\n', out);
// 		llhd_asm_write_unit(U, out);
// 	}
// }
