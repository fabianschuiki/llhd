// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/common.h"
#include <stdio.h>

int main() {
	llhd_arg_t *Adata_a = llhd_make_arg("data_a", NULL);
	llhd_arg_t *Adata_b = llhd_make_arg("data_b", NULL);
	llhd_arg_t *Aoperation = llhd_make_arg("operation", NULL);
	llhd_arg_t *Acarry = llhd_make_arg("carry", NULL);
	llhd_arg_t *Aflag = llhd_make_arg("flag", NULL);
	llhd_arg_t *Aresult = llhd_make_arg("result", NULL);
	llhd_basic_block_t *BBentry = llhd_make_basic_block("entry");
	llhd_proc_t *P = llhd_make_proc("alu", (llhd_arg_t*[]){Adata_a, Adata_b, Aoperation}, 3, (llhd_arg_t*[]){Acarry, Aflag, Aresult}, 3, BBentry);

	llhd_inst_t *I = (llhd_inst_t*)llhd_make_drive_inst();
	llhd_add_inst(I, BBentry);

	printf("P = ");
	llhd_dump_value(P, stdout);
	printf("\n");

	llhd_destroy_value(P);
	return 0;
}
