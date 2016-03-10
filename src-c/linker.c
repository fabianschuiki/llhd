// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdlib.h>
#include <string.h>

int compare_units(const void *a, const void *b) {
	return strcmp(llhd_value_get_name((void*)a), llhd_value_get_name((void*)b));
}

/// Moves all definitions and declarations into the first module and replaces
/// declarations by corresponding definitions where possible.
void llhd_link_modules(llhd_module_t modules[], unsigned num_modules) {
	unsigned i;
	if (num_modules < 2)
		return;

	// Algorithm:
	// 1) Count number of definitions and declarations across all modules.
	// 2) Allocate and fill two arrays with the definitions and declarations.
	// 3) Sort both arrays by name.
	// 4) Iterate over declarations, find corresponding definition, replace all
	//    uses if found and remove. Optimize if next declaration is the same.
	// 5) Iterate over definitions and declarations across all modules and move
	//    into the first.

	// Count the number of definitions and declarations across all modules.
	unsigned num_defs = 0;
	unsigned num_decls = 0;

	for (i = 0; i < num_modules; ++i) {
		llhd_module_t M = modules[i];
		llhd_value_t U;
		for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
			if (llhd_unit_is_def(U)) ++num_defs;
			if (llhd_unit_is_decl(U)) ++num_decls;
		}
	}

	// Build two lists for definitions and declarations each, sorted by name.
	llhd_value_t *defs = malloc(num_defs * sizeof(llhd_value_t));
	llhd_value_t *decls = malloc(num_decls * sizeof(llhd_value_t));

	llhd_value_t *defs_ptr = defs;
	llhd_value_t *decls_ptr = decls;
	for (i = 0; i < num_modules; ++i) {
		llhd_module_t M = modules[i];
		llhd_value_t U;
		for (U = llhd_module_get_first_unit(M); U; U = llhd_unit_next(U)) {
			if (llhd_unit_is_def(U)) *defs_ptr++ = U;
			if (llhd_unit_is_decl(U)) *decls_ptr++ = U;
		}
	}

	qsort(defs, num_defs, sizeof(llhd_value_t), compare_units);
	qsort(decls, num_decls, sizeof(llhd_value_t), compare_units);

	// Clean up.
	free(defs);
	free(decls);
}
