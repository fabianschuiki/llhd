// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdlib.h>
#include <string.h>

// Moves all definitions and declarations into the first module and replaces
// declarations by corresponding definitions where possible.
//
// Algorithm:
// 1) Count number of definitions and declarations across all modules.
// 2) Allocate and fill two arrays with the definitions and declarations.
// 3) Sort both arrays by name.
// 4) Iterate over declarations, find corresponding definition, replace all uses
//    if found and remove. Optimize if next declaration is the same.
// 5) Iterate over definitions and declarations across all modules and move into
//    the first.

static int
compare_units (const void *pa, const void *pb) {
	llhd_value_t a = *(llhd_value_t*)pa;
	llhd_value_t b = *(llhd_value_t*)pb;
	return strcmp(llhd_value_get_name(a), llhd_value_get_name(b));
}

static int
compare_units2 (const void *key, const void *pu) {
	llhd_value_t u = *(llhd_value_t*)pu;
	return strcmp(key, llhd_value_get_name(u));
}

void
llhd_link_modules (llhd_module_t modules[], unsigned num_modules) {
	unsigned i;
	if (num_modules < 2)
		return;

	// Count the number of definitions and declarations across all modules.
	unsigned num_defs  = 0;
	unsigned num_decls = 0;

	for (i = 0; i < num_modules; ++i) {
		llhd_module_t M = modules[i];
		llhd_value_t U;
		for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
			if (llhd_unit_is_def(U))  ++num_defs;
			if (llhd_unit_is_decl(U)) ++num_decls;
		}
	}

	// Build lists for definitions and declarations, sorted by name.
	llhd_value_t *defs  = malloc(num_defs  * sizeof(llhd_value_t));
	llhd_value_t *decls = malloc(num_decls * sizeof(llhd_value_t));

	llhd_value_t *defs_ptr  = defs;
	llhd_value_t *decls_ptr = decls;
	for (i = 0; i < num_modules; ++i) {
		llhd_module_t M = modules[i];
		llhd_value_t U;
		for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
			if (llhd_unit_is_def(U))  *defs_ptr++ = U;
			if (llhd_unit_is_decl(U)) *decls_ptr++ = U;
		}
	}

	qsort(defs,  num_defs,  sizeof(llhd_value_t), compare_units);
	qsort(decls, num_decls, sizeof(llhd_value_t), compare_units);

	// Find a definition to replace each declaration with.
	for (i = 0; i < num_decls; ++i) {
		llhd_value_t decl = decls[i];
		const char *decl_name = llhd_value_get_name(decl);
		llhd_value_t *def = bsearch(decl_name, defs, num_defs, sizeof(llhd_value_t), compare_units2);
		// TODO: check whether types match
		if (def) {
			llhd_value_replace_uses(decl,*def);
			llhd_value_unlink(decl);
			llhd_value_free(decl);
		}
	}

	// Move all units into the first module and verify that the result is
	// selfcontained, i.e. that no references cross module boundaries.
	for (i = 1; i < num_modules; ++i) {
		llhd_module_t M = modules[i];
		llhd_value_t U, Un;
		for (U = llhd_module_get_first_unit(M); U; U = Un) {
			Un = llhd_unit_next(U);
			llhd_value_unlink(U);
			llhd_unit_append_to(U, modules[0]);
		}
	}
	llhd_verify_module_selfcontained(modules[0]);

	// Clean up.
	free(defs);
	free(decls);
}
