// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>
#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main() {
	llhd_module_t M;
	llhd_value_t P, I, Vtmp, Vprb, V, BB;
	llhd_type_t Pty, i32ty, i8ty, i8sigty;

	i32ty = llhd_type_new_int(32);
	i8ty = llhd_type_new_int(8);
	i8sigty = llhd_type_new_signal(i8ty);

	M = llhd_module_new("debug4");

	Pty = llhd_type_new_comp((llhd_type_t[]){i8sigty}, 1, (llhd_type_t[]){i8sigty}, 1);
	P = llhd_proc_new(Pty, "foo");
	llhd_value_set_name(llhd_unit_get_input(P,0), "di");
	llhd_value_set_name(llhd_unit_get_output(P,0), "do");
	llhd_type_unref(Pty);
	llhd_unit_append_to(P,M);

	BB = llhd_block_new("entry");
	llhd_block_append_to(BB,P);
	llhd_value_unref(BB);

	Vprb = llhd_inst_probe_new(llhd_unit_get_input(P,0), "diprb");
	llhd_inst_append_to(Vprb, BB);
	llhd_value_unref(Vprb);

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

	Vprb = llhd_inst_binary_new(LLHD_BINARY_ADD, Vprb, Vprb, NULL);
	llhd_inst_append_to(Vprb, BB);
	llhd_value_unref(Vprb);

	I = llhd_inst_drive_new(llhd_unit_get_output(P,0), Vprb);
	llhd_inst_append_to(I, BB);
	llhd_value_unref(I);

	llhd_asm_write_module(M, stdout);
	llhd_module_free(M);

	llhd_type_unref(i32ty);
	llhd_type_unref(i8ty);
	llhd_type_unref(i8sigty);

	return 0;
}
