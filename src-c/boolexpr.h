/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include <stdio.h>

enum llhd_boolexpr_kind {
	LLHD_BOOLEXPR_CONST_0 = 1,
	LLHD_BOOLEXPR_CONST_1 = 2,
	LLHD_BOOLEXPR_SYMBOL  = 3,
	LLHD_BOOLEXPR_OR      = 4,
	LLHD_BOOLEXPR_AND     = 5,
};

struct llhd_boolexpr *llhd_boolexpr_new_const_0();
struct llhd_boolexpr *llhd_boolexpr_new_const_1();
struct llhd_boolexpr *llhd_boolexpr_new_symbol(void*);
struct llhd_boolexpr *llhd_boolexpr_new_and(struct llhd_boolexpr*[], unsigned);
struct llhd_boolexpr *llhd_boolexpr_new_or(struct llhd_boolexpr*[], unsigned);
struct llhd_boolexpr *llhd_boolexpr_copy(struct llhd_boolexpr*);
void llhd_boolexpr_free(struct llhd_boolexpr*);

enum llhd_boolexpr_kind llhd_boolexpr_get_kind(struct llhd_boolexpr*);
enum llhd_boolexpr_kind llhd_boolexpr_is(struct llhd_boolexpr*, enum llhd_boolexpr_kind);
void *llhd_boolexpr_get_symbol(struct llhd_boolexpr*);
unsigned llhd_boolexpr_get_num_children(struct llhd_boolexpr*);
struct llhd_boolexpr **llhd_boolexpr_get_children(struct llhd_boolexpr*);

void llhd_boolexpr_negate(struct llhd_boolexpr*);
void llhd_boolexpr_disjunctive_cnf(struct llhd_boolexpr**);

void llhd_boolexpr_write(struct llhd_boolexpr*, void(*)(void*,FILE*), FILE*);
