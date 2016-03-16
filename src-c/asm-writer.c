// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdio.h>

void
llhd_asm_write_unit (llhd_value_t U, FILE *out) {
	// switch over unit types
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
