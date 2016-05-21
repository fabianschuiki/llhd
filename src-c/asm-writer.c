// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

void
llhd_asm_write_type (llhd_type_t T, FILE *out) {
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
				llhd_asm_write_type(llhd_type_get_field(T,i), out);
			}
			fputc('}', out);
			break;
		case LLHD_TYPE_ARRAY:
			fprintf(out, "[%d x ", llhd_type_get_length(T));
			llhd_asm_write_type(llhd_type_get_subtype(T), out);
			fputc(']', out);
			break;
		case LLHD_TYPE_PTR:
			llhd_asm_write_type(llhd_type_get_subtype(T), out);
			fputc('*', out);
			break;
		case LLHD_TYPE_SIGNAL:
			llhd_asm_write_type(llhd_type_get_subtype(T), out);
			fputc('$', out);
			break;
		case LLHD_TYPE_FUNC:
			fputs("func(", out);
			N = llhd_type_get_num_inputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				llhd_asm_write_type(llhd_type_get_input(T,i), out);
			}
			fputs(")(", out);
			N = llhd_type_get_num_outputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				llhd_asm_write_type(llhd_type_get_output(T,i), out);
			}
			fputs(")", out);
			break;
		case LLHD_TYPE_COMP:
			fputs("comp(", out);
			N = llhd_type_get_num_inputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				llhd_asm_write_type(llhd_type_get_input(T,i), out);
			}
			fputs(")(", out);
			N = llhd_type_get_num_outputs(T);
			for (i = 0; i < N; ++i) {
				if (i > 0) fputs(", ", out);
				llhd_asm_write_type(llhd_type_get_output(T,i), out);
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
	llhd_asm_write_type(llhd_value_get_type(D), out);
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
	llhd_asm_write_type(llhd_value_get_type(P), out);
	const char *name = llhd_value_get_name(P);
	if (name || llhd_value_has_users(P)) {
		const char *an = symtbl_add_name(symtbl, P, name);
		fputs(" %", out);
		fputs(an, out);
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
		llhd_asm_write_type(T, out);
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
		const char *name = symtbl_lookup_sym(symtbl, V);
		if (name) {
			fputc('%', out);
			fputs(name, out);
		} else {
			fprintf(out, "<?%p>", V);
		}
	}
}

static void
write_inst(llhd_value_t I, struct llhd_symtbl *symtbl, FILE *out) {
	int kind = llhd_value_get_kind(I);
	const char *name = llhd_value_get_name(I);
	llhd_value_t cond, comp, func;
	unsigned i, num;
	if (name || llhd_value_has_users(I)) {
		const char *an = symtbl_add_name(symtbl, I, name);
		fputc('%', out);
		fputs(an, out);
		fputs(" = ", out);
	}
	switch (LLHD_AS(kind, LLHD_MASK_INST)) {
		case LLHD_INST_UNARY:
			assert(false && "write unary not implemented");
			break;
		case LLHD_INST_BINARY:
			// fprintf(out, "bin%d ", llhd_inst_binary_get_op(I));
			fputs(llhd_inst_binary_get_opname(I), out);
			fputc(' ', out);
			llhd_asm_write_type(llhd_value_get_type(I), out);
			fputc(' ', out);
			write_value_ref(llhd_inst_binary_get_lhs(I), 0, symtbl, out);
			fputc(' ', out);
			write_value_ref(llhd_inst_binary_get_rhs(I), 0, symtbl, out);
			break;
		case LLHD_INST_SIGNAL:
			fputs("sig ", out);
			llhd_asm_write_type(llhd_value_get_type(I), out);
			break;
		case LLHD_INST_COMPARE:
			fputs("cmp ", out);
			fputs(llhd_inst_compare_get_opname(I), out);
			fputc(' ', out);
			write_value_ref(llhd_inst_compare_get_lhs(I), 1, symtbl, out);
			fputc(' ', out);
			write_value_ref(llhd_inst_compare_get_rhs(I), 0, symtbl, out);
			break;
		case LLHD_INST_BRANCH:
			fputs("br ", out);
			cond = llhd_inst_branch_get_condition(I);
			if (cond) {
				write_value_ref(cond, 1, symtbl, out);
				fputs(", ", out);
				write_value_ref(llhd_inst_branch_get_dst1(I), 1, symtbl, out);
				fputs(", ", out);
				write_value_ref(llhd_inst_branch_get_dst0(I), 1, symtbl, out);
			} else {
				write_value_ref(llhd_inst_branch_get_dst(I), 1, symtbl, out);
			}
			break;
		case LLHD_INST_PROBE:
			fputs("prb ", out);
			write_value_ref(llhd_inst_probe_get_signal(I), 1, symtbl, out);
			break;
		case LLHD_INST_DRIVE:
			fputs("drv ", out);
			write_value_ref(llhd_inst_drive_get_sig(I), 1, symtbl, out);
			fputc(' ', out);
			write_value_ref(llhd_inst_drive_get_val(I), 0, symtbl, out);
			break;
		case LLHD_INST_INST:
			fputs("inst ", out);
			comp = llhd_inst_inst_get_comp(I);
			llhd_asm_write_type(llhd_value_get_type(comp), out);
			fputs(" @", out);
			fputs(llhd_value_get_name(comp), out);
			fputs(" (", out);
			num = llhd_inst_inst_get_num_inputs(I);
			for (i = 0; i < num; ++i) {
				if (i != 0) fputs(", ", out);
				write_value_ref(llhd_inst_inst_get_input(I,i), 0, symtbl, out);
			}
			fputs(") (", out);
			num = llhd_inst_inst_get_num_outputs(I);
			for (i = 0; i < num; ++i) {
				if (i != 0) fputs(", ", out);
				write_value_ref(llhd_inst_inst_get_output(I,i), 0, symtbl, out);
			}
			fputs(")", out);
			break;
		case LLHD_INST_CALL:
			fputs("call ", out);
			func = llhd_inst_call_get_func(I);
			llhd_asm_write_type(llhd_value_get_type(func), out);
			fputs(" @", out);
			fputs(llhd_value_get_name(func), out);
			fputs(" (", out);
			num = llhd_inst_call_get_num_args(I);
			for (i = 0; i < num; ++i) {
				if (i != 0) fputs(", ", out);
				write_value_ref(llhd_inst_call_get_arg(I,i), 0, symtbl, out);
			}
			fputs(")", out);
			break;
		case LLHD_INST_RET:
			fputs("ret", out);
			num = llhd_inst_ret_get_num_args(I);
			for (i = 0; i < num; ++i) {
				fputs(i == 0 ? " " : ", ", out);
				write_value_ref(llhd_inst_ret_get_arg(I,i), 0, symtbl, out);
			}
			break;
		case LLHD_INST_EXTRACT:
			fputs("extract ", out);
			write_value_ref(llhd_inst_extract_get_target(I), 1, symtbl, out);
			fprintf(out, " %u", llhd_inst_extract_get_index(I));
			break;
		case LLHD_INST_INSERT:
			fputs("insert ", out);
			write_value_ref(llhd_inst_insert_get_target(I), 1, symtbl, out);
			fprintf(out, " %u ", llhd_inst_insert_get_index(I));
			write_value_ref(llhd_inst_insert_get_value(I), 0, symtbl, out);
			break;
		case LLHD_INST_REG:
			fputs("reg ", out);
			write_value_ref(llhd_inst_reg_get_value(I), 1, symtbl, out);
			fputs(", ", out);
			write_value_ref(llhd_inst_reg_get_strobe(I), 0, symtbl, out);
			break;
		case LLHD_INST_LOAD:
			fputs("load ", out);
			write_value_ref(llhd_inst_load_get_target(I), 1, symtbl, out);
			break;
		case LLHD_INST_STORE:
			fputs("store ", out);
			write_value_ref(llhd_inst_store_get_target(I), 1, symtbl, out);
			fputs(", ", out);
			write_value_ref(llhd_inst_store_get_value(I), 0, symtbl, out);
			break;
		case LLHD_INST_VAR:
			fputs("var ", out);
			llhd_asm_write_type(llhd_type_get_subtype(llhd_value_get_type(I)), out);
			break;
		default:
			assert(0 && "unknown inst kind");
	}
	fprintf(out, "  ;[#users = %d]", llhd_value_get_num_users(I));
}

static void
write_insts(llhd_value_t I, struct llhd_symtbl *symtbl, FILE *out) {
	for (; I; I = llhd_inst_next(I)) {
		fputc('\t', out);
		write_inst(I, symtbl, out);
		fputc('\n', out);
	}
}

static void
write_blocks(llhd_list_t list, struct llhd_symtbl *symtbl, FILE *out) {
	llhd_list_t pos;
	llhd_value_t BB;
	pos = llhd_block_first(list);
	while ((BB = llhd_block_next(list, &pos))) {
		symtbl_add_name(symtbl, BB, llhd_value_get_name(BB));
	}
	pos = llhd_block_first(list);
	while ((BB = llhd_block_next(list, &pos))) {
		fputs(symtbl_lookup_sym(symtbl, BB), out);
		fputs(":\n", out);
		write_insts(llhd_block_get_first_inst(BB), symtbl, out);
	}
}

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

static void
write_func_def (llhd_value_t D, FILE *out) {
	fprintf(out, "func @%s ", llhd_value_get_name(D));
	struct llhd_symtbl *symtbl = symtbl_new();
	write_unit_params(D, symtbl, out);
	fputs(" {\n", out);
	write_blocks(llhd_unit_get_blocks(D), symtbl, out);
	fputs("}\n", out);
	symtbl_free(symtbl);
}

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

static void
write_proc_def (llhd_value_t D, FILE *out) {
	fprintf(out, "proc @%s ", llhd_value_get_name(D));
	struct llhd_symtbl *symtbl = symtbl_new();
	write_unit_params(D, symtbl, out);
	fputs(" {\n", out);
	write_blocks(llhd_unit_get_blocks(D), symtbl, out);
	fputs("}\n", out);
	symtbl_free(symtbl);
}

void
llhd_asm_write_unit (llhd_value_t U, FILE *out) {
	int kind = llhd_value_get_kind(U);
	switch (LLHD_AS(kind, LLHD_MASK_UNIT)) {
		case LLHD_UNIT_DECL: write_decl(U, out); break;
		case LLHD_UNIT_DEF_FUNC: write_func_def(U, out); break;
		case LLHD_UNIT_DEF_ENTITY: write_entity_def(U, out); break;
		case LLHD_UNIT_DEF_PROC: write_proc_def(U, out); break;
		default:
			assert(0 && "unsupported unit kind");
	}
}

void
llhd_asm_write_module (llhd_module_t M, FILE *out) {
	fprintf(out, "; module '%s'\n", llhd_module_get_name(M));
	llhd_value_t U;
	llhd_list_t units = llhd_module_get_units(M);
	llhd_list_t p = llhd_unit_first(units);
	while ((U = llhd_unit_next(units, &p))) {
		fputc('\n', out);
		llhd_asm_write_unit(U, out);
	}
}
