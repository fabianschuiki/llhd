// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/common.h"
#include <stdio.h>

static llhd_proc_t *make_alu () {
	llhd_arg_t *Aa = llhd_make_arg("a", llhd_make_logic_type(8));
	llhd_arg_t *Ab = llhd_make_arg("b", llhd_make_logic_type(8));
	llhd_arg_t *Aop = llhd_make_arg("op", llhd_make_logic_type(2));
	llhd_arg_t *Ar = llhd_make_arg("r", llhd_make_logic_type(8));

	llhd_basic_block_t *BBentry = llhd_make_basic_block("entry");
	llhd_proc_t *P = llhd_make_proc("alu", (llhd_arg_t*[]){Aa, Ab, Aop}, 3, (llhd_arg_t*[]){Ar}, 1, BBentry);

	llhd_basic_block_t *BBnot00 = llhd_make_basic_block("not00");
	llhd_basic_block_t *BBnot01 = llhd_make_basic_block("not01");
	llhd_basic_block_t *BBnot10 = llhd_make_basic_block("not10");
	llhd_basic_block_t *BBnot11 = llhd_make_basic_block("not11");
	llhd_basic_block_t *BB00 = llhd_make_basic_block("op00");
	llhd_basic_block_t *BB01 = llhd_make_basic_block("op01");
	llhd_basic_block_t *BB10 = llhd_make_basic_block("op10");
	llhd_basic_block_t *BB11 = llhd_make_basic_block("op11");
	llhd_insert_basic_block_after(BBnot00, BBentry);
	llhd_insert_basic_block_after(BBnot01, BBnot00);
	llhd_insert_basic_block_after(BBnot10, BBnot01);
	llhd_insert_basic_block_after(BBnot11, BBnot10);
	llhd_insert_basic_block_after(BB00, BBnot11);
	llhd_insert_basic_block_after(BB01, BB00);
	llhd_insert_basic_block_after(BB10, BB01);
	llhd_insert_basic_block_after(BB11, BB10);

	llhd_inst_t *I;

	// r <= (others => 'U')
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)llhd_make_const_logic(8,"UUUUUUUU"));
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BBnot11);

	// when "00"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"00"));
	llhd_value_set_name(I, "0");
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB00, BBnot00);
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_ADD, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "6");
	llhd_add_inst(I, BB00);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_add_inst(I, BB00);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BB00);

	// when "01"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"01"));
	llhd_value_set_name(I, "1");
	llhd_add_inst(I, BBnot00);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB01, BBnot01);
	llhd_add_inst(I, BBnot00);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_SUB, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "7");
	llhd_add_inst(I, BB01);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_add_inst(I, BB01);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BB01);

	// when "10"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"10"));
	llhd_value_set_name(I, "2");
	llhd_add_inst(I, BBnot01);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB10, BBnot10);
	llhd_add_inst(I, BBnot01);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_AND, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "4");
	llhd_add_inst(I, BB10);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_add_inst(I, BB10);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BB10);

	// when "11"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"11"));
	llhd_value_set_name(I, "3");
	llhd_add_inst(I, BBnot10);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB11, BBnot11);
	llhd_add_inst(I, BBnot10);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_OR, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "5");
	llhd_add_inst(I, BB11);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_add_inst(I, BB11);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BB11);

	return P;
}

static llhd_proc_t *make_stim() {
	llhd_arg_t *Aa = llhd_make_arg("a", llhd_make_logic_type(8));
	llhd_arg_t *Ab = llhd_make_arg("b", llhd_make_logic_type(8));
	llhd_arg_t *Aop = llhd_make_arg("op", llhd_make_logic_type(2));
	llhd_arg_t *Ar = llhd_make_arg("r", llhd_make_logic_type(8));

	llhd_basic_block_t *BBentry = llhd_make_basic_block("entry");
	llhd_proc_t *P = llhd_make_proc("stim", (llhd_arg_t*[]){Ar}, 1, (llhd_arg_t*[]){Aa, Ab, Aop}, 3, BBentry);

	llhd_inst_t *I;

	// a <= "00010010"
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aa, (llhd_value_t*)llhd_make_const_logic(8,"00010010"));
	llhd_add_inst(I, BBentry);

	// b <= "00001010"
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ab, (llhd_value_t*)llhd_make_const_logic(8,"00001010"));
	llhd_add_inst(I, BBentry);

	// op <= "00"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"00"));
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_add_inst(I, BBentry);

	// op <= "01"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"01"));
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_add_inst(I, BBentry);

	// op <= "10"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"10"));
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_add_inst(I, BBentry);

	// op <= "11"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"11"));
	llhd_add_inst(I, BBentry);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_add_inst(I, BBentry);

	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_add_inst(I, BBentry);

	return P;
}

int main() {
	llhd_proc_t *Palu = make_alu();
	llhd_proc_t *Pstim = make_stim();

	llhd_entity_t *Etb = NULL;
	// llhd_signal_t *Sa = llhd_make_signal(llhd_make_logic_type(8));
	// llhd_signal_t *Sb = llhd_make_signal(llhd_make_logic_type(8));
	// llhd_signal_t *Sop = llhd_make_signal(llhd_make_logic_type(2));
	// llhd_signal_t *Sr = llhd_make_signal(llhd_make_logic_type(8));
	// llhd_instance_t *Ialu = llhd_make_instance(Palu, {Sa,Sb,Sop}, 3, {Sr}, 1);
	// llhd_instance_t *Istim = llhd_make_instance(Pstim, {Sr}, 1, {Sa,Sb,Sop}, 3);

	llhd_dump_value(Palu, stdout); fputs("\n\n", stdout);
	llhd_dump_value(Pstim, stdout); fputs("\n\n", stdout);
	llhd_dump_value(Etb, stdout); fputs("\n\n", stdout);

	llhd_destroy_value(Palu);
	llhd_destroy_value(Pstim);
	llhd_destroy_value(Etb);
	return 0;
}
