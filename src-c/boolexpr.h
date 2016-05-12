/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include <stdio.h>

struct llhd_boolexpr *llhd_boolexpr_new_const_0();
struct llhd_boolexpr *llhd_boolexpr_new_const_1();
struct llhd_boolexpr *llhd_boolexpr_new_symbol(void*);
struct llhd_boolexpr *llhd_boolexpr_new_and(struct llhd_boolexpr*[], unsigned);
struct llhd_boolexpr *llhd_boolexpr_new_or(struct llhd_boolexpr*[], unsigned);
struct llhd_boolexpr *llhd_boolexpr_copy(struct llhd_boolexpr*);
void llhd_boolexpr_free(struct llhd_boolexpr*);

void llhd_boolexpr_negate(struct llhd_boolexpr*);
void llhd_boolexpr_disjunctive_cnf(struct llhd_boolexpr**);

void llhd_boolexpr_write(struct llhd_boolexpr*, void(*)(void*,FILE*), FILE*);
