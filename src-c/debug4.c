// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main() {
	llhd_module_t M;
	llhd_value_t P, I, Vtmp, V, BB;
	llhd_type_t Pty, i32ty;

	i32ty = llhd_type_new_int(32);
	M = llhd_module_new("debug4");

	Pty = llhd_type_new_comp(NULL, 0, NULL, 0);
	P = llhd_proc_new(Pty, "foo");
	llhd_type_unref(Pty);
	llhd_unit_append_to(P,M);

	BB = llhd_block_new("entry");
	llhd_block_append_to(BB,P);
	llhd_value_unref(BB);

	Vtmp = llhd_inst_var_new(i32ty, "tmp");
	llhd_inst_append_to(Vtmp, BB);
	llhd_value_unref(Vtmp);

	V = llhd_const_int_new(32, 42);
	I = llhd_inst_store_new(Vtmp, V);
	llhd_value_unref(V);
	llhd_inst_append_to(I, BB);
	llhd_value_unref(I);

	I = llhd_inst_load_new(Vtmp, "tmp");
	llhd_inst_append_to(I, BB);
	llhd_value_unref(I);

	llhd_asm_write_module(M, stdout);
	llhd_module_free(M);

	return 0;
}
