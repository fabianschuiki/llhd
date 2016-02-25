// Copyright (c) 2016 Fabian Schuiki
#include "llhdc/ir.h"
#include <stdio.h>

static llhd_proc_t *make_alu () {
	llhd_arg_t *Aa = llhd_make_arg("a", llhd_type_make_logic(8));
	llhd_arg_t *Ab = llhd_make_arg("b", llhd_type_make_logic(8));
	llhd_arg_t *Aop = llhd_make_arg("op", llhd_type_make_logic(2));
	llhd_arg_t *Ar = llhd_make_arg("r", llhd_type_make_logic(8));

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
	llhd_basic_block_insert_after(BBnot00, BBentry);
	llhd_basic_block_insert_after(BBnot01, BBnot00);
	llhd_basic_block_insert_after(BBnot10, BBnot01);
	llhd_basic_block_insert_after(BBnot11, BBnot10);
	llhd_basic_block_insert_after(BB00, BBnot11);
	llhd_basic_block_insert_after(BB01, BB00);
	llhd_basic_block_insert_after(BB10, BB01);
	llhd_basic_block_insert_after(BB11, BB10);

	llhd_inst_t *I;

	// r <= (others => 'U')
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)llhd_make_const_logic(8,"UUUUUUUU"));
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BBnot11, I);

	// when "00"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"00"));
	llhd_value_set_name(I, "0");
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB00, BBnot00);
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_ADD, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "6");
	llhd_basic_block_append_inst(BB00, I);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_basic_block_append_inst(BB00, I);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BB00, I);

	// when "01"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"01"));
	llhd_value_set_name(I, "1");
	llhd_basic_block_append_inst(BBnot00, I);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB01, BBnot01);
	llhd_basic_block_append_inst(BBnot00, I);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_SUB, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "7");
	llhd_basic_block_append_inst(BB01, I);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_basic_block_append_inst(BB01, I);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BB01, I);

	// when "10"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"10"));
	llhd_value_set_name(I, "2");
	llhd_basic_block_append_inst(BBnot01, I);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB10, BBnot10);
	llhd_basic_block_append_inst(BBnot01, I);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_AND, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "4");
	llhd_basic_block_append_inst(BB10, I);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_basic_block_append_inst(BB10, I);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BB10, I);

	// when "11"
	I = (llhd_inst_t*)llhd_make_compare_inst(LLHD_EQ, (llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"11"));
	llhd_value_set_name(I, "3");
	llhd_basic_block_append_inst(BBnot10, I);
	I = (llhd_inst_t*)llhd_make_conditional_branch_inst((llhd_value_t*)I, BB11, BBnot11);
	llhd_basic_block_append_inst(BBnot10, I);
	I = (llhd_inst_t*)llhd_make_binary_inst(LLHD_OR, (llhd_value_t*)Aa, (llhd_value_t*)Ab);
	llhd_value_set_name(I, "5");
	llhd_basic_block_append_inst(BB11, I);
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ar, (llhd_value_t*)I);
	llhd_basic_block_append_inst(BB11, I);
	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BB11, I);

	return P;
}

static llhd_proc_t *make_stim() {
	llhd_arg_t *Aa = llhd_make_arg("a", llhd_type_make_logic(8));
	llhd_arg_t *Ab = llhd_make_arg("b", llhd_type_make_logic(8));
	llhd_arg_t *Aop = llhd_make_arg("op", llhd_type_make_logic(2));
	llhd_arg_t *Ar = llhd_make_arg("r", llhd_type_make_logic(8));

	llhd_basic_block_t *BBentry = llhd_make_basic_block("entry");
	llhd_proc_t *P = llhd_make_proc("stim", (llhd_arg_t*[]){Ar}, 1, (llhd_arg_t*[]){Aa, Ab, Aop}, 3, BBentry);

	llhd_inst_t *I;

	// a <= "00010010"
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aa, (llhd_value_t*)llhd_make_const_logic(8,"00010010"));
	llhd_basic_block_append_inst(BBentry, I);

	// b <= "00001010"
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Ab, (llhd_value_t*)llhd_make_const_logic(8,"00001010"));
	llhd_basic_block_append_inst(BBentry, I);

	// op <= "00"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"00"));
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_basic_block_append_inst(BBentry, I);

	// op <= "01"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"01"));
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_basic_block_append_inst(BBentry, I);

	// op <= "10"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"10"));
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_basic_block_append_inst(BBentry, I);

	// op <= "11"; wait for 1 ns
	I = (llhd_inst_t*)llhd_make_drive_inst((llhd_value_t*)Aop, (llhd_value_t*)llhd_make_const_logic(2,"11"));
	llhd_basic_block_append_inst(BBentry, I);
	I = (llhd_inst_t*)llhd_make_wait_inst((llhd_value_t*)llhd_make_const_time("1ns"));
	llhd_basic_block_append_inst(BBentry, I);

	I = (llhd_inst_t*)llhd_make_ret_inst(NULL, 0);
	llhd_basic_block_append_inst(BBentry, I);

	return P;
}

int main() {
	llhd_proc_t *Palu = make_alu();
	llhd_proc_t *Pstim = make_stim();

	llhd_entity_t *Etb = llhd_make_entity("tb", NULL, 0, NULL, 0);
	void *Sa = llhd_make_signal_inst(llhd_type_make_logic(8));
	llhd_value_set_name(Sa, "a");
	llhd_entity_append_inst(Etb, Sa);
	void *Sb = llhd_make_signal_inst(llhd_type_make_logic(8));
	llhd_value_set_name(Sb, "b");
	llhd_entity_append_inst(Etb, Sb);
	void *Sop = llhd_make_signal_inst(llhd_type_make_logic(2));
	llhd_value_set_name(Sop, "op");
	llhd_entity_append_inst(Etb, Sop);
	void *Sr = llhd_make_signal_inst(llhd_type_make_logic(8));
	llhd_value_set_name(Sr, "r");
	llhd_entity_append_inst(Etb, Sr);
	void *Ialu = llhd_make_instance_inst((llhd_value_t*)Palu, (llhd_value_t*[]){Sa,Sb,Sop}, 3, (llhd_value_t*[]){Sr}, 1);
	llhd_value_set_name(Ialu, "alu_i");
	llhd_entity_append_inst(Etb, Ialu);
	void *Istim = llhd_make_instance_inst((llhd_value_t*)Pstim, (llhd_value_t*[]){Sr}, 1, (llhd_value_t*[]){Sa,Sb,Sop}, 3);
	llhd_value_set_name(Istim, "stim_i");
	llhd_entity_append_inst(Etb, Istim);

	llhd_value_dump(Palu, stdout); fputs("\n\n", stdout);
	llhd_value_dump(Pstim, stdout); fputs("\n\n", stdout);
	llhd_value_dump(Etb, stdout); fputs("\n\n", stdout);

	llhd_value_destroy(Palu);
	llhd_value_destroy(Pstim);
	llhd_value_destroy(Etb);
	return 0;
}
