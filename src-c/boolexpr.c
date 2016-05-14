/* Copyright (c) 2016 Fabian Schuiki */
#include <llhd.h>
#include "boolexpr.h"
#include <assert.h>
#include <string.h>
#include <stdlib.h>

struct llhd_boolexpr {
	unsigned kind:3;
	unsigned negate:1;
	unsigned num_children:28;
	struct llhd_boolexpr *children[];
};

struct llhd_boolexpr *
alloc_boolexpr(unsigned num_children) {
	struct llhd_boolexpr *expr;
	expr = llhd_zalloc(sizeof(*expr) + num_children * sizeof(void*));
	expr->num_children = num_children;
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_new_const_0() {
	struct llhd_boolexpr *expr;
	expr = alloc_boolexpr(0);
	expr->kind = LLHD_BOOLEXPR_CONST_0;
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_new_const_1() {
	struct llhd_boolexpr *expr;
	expr = alloc_boolexpr(0);
	expr->kind = LLHD_BOOLEXPR_CONST_1;
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_new_symbol(void *symbol) {
	struct llhd_boolexpr *expr;
	expr = alloc_boolexpr(1);
	expr->kind = LLHD_BOOLEXPR_SYMBOL;
	expr->children[0] = symbol;
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_new_and(struct llhd_boolexpr *children[], unsigned num_children) {
	struct llhd_boolexpr *expr;
	assert(num_children == 0 || children);
	expr = alloc_boolexpr(num_children);
	expr->kind = LLHD_BOOLEXPR_AND;
	memcpy(expr->children, children, num_children * sizeof(void*));
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_new_or(struct llhd_boolexpr *children[], unsigned num_children) {
	struct llhd_boolexpr *expr;
	assert(num_children == 0 || children);
	expr = alloc_boolexpr(num_children);
	expr->kind = LLHD_BOOLEXPR_OR;
	memcpy(expr->children, children, num_children * sizeof(void*));
	return expr;
}

struct llhd_boolexpr *
llhd_boolexpr_copy(struct llhd_boolexpr *expr) {
	struct llhd_boolexpr *copy;
	unsigned i;
	assert(expr);
	copy = alloc_boolexpr(expr->num_children);
	copy->kind = expr->kind;
	copy->negate = expr->negate;
	if (expr->kind != LLHD_BOOLEXPR_SYMBOL) {
		for (i = 0; i < expr->num_children; ++i)
			copy->children[i] = llhd_boolexpr_copy(expr->children[i]);
	} else {
		copy->children[0] = expr->children[0];
	}
	return copy;
}

void
llhd_boolexpr_free(struct llhd_boolexpr *expr) {
	assert(expr);
	if (expr->kind != LLHD_BOOLEXPR_SYMBOL) {
		unsigned i;
		for (i = 0; i < expr->num_children; ++i)
			if (expr->children[i])
				llhd_boolexpr_free(expr->children[i]);
	}
	llhd_free(expr);
}

void
llhd_boolexpr_negate(struct llhd_boolexpr *expr) {
	assert(expr);
	expr->negate = !expr->negate;
}

static int
cmp_expr_opt(struct llhd_boolexpr *a, struct llhd_boolexpr *b, int ineg) {
	if (a->kind < b->kind) return -1;
	if (a->kind > b->kind) return  1;
	if (!ineg) {
		if (a->negate < b->negate) return -1;
		if (a->negate > b->negate) return  1;
	}
	if (a->num_children < b->num_children) return -1;
	if (a->num_children > b->num_children) return  1;
	if (a->kind != LLHD_BOOLEXPR_SYMBOL) {
		unsigned i,r;
		for (i = 0; i < a->num_children; ++i) {
			r = cmp_expr_opt(a->children[i], b->children[i], 0);
			if (r != 0) return r;
		}
	} else {
		if (a->children[0] < b->children[0]) return -1;
		if (a->children[0] > b->children[0]) return  1;
	}
	return 0;
}

static int
cmp_expr(const void *pa, const void *pb) {
	return cmp_expr_opt(*(void**)pa, *(void**)pb, 0);
}

static int
cmp_expr_ineg(const void *pa, const void *pb) {
	return cmp_expr_opt(*(void**)pa, *(void**)pb, 1);
}

static void
simplify_children(
	struct llhd_boolexpr **expr_ptr,
	enum llhd_boolexpr_kind mask_child,
	enum llhd_boolexpr_kind identity_child
) {
	unsigned i, o;
	struct llhd_boolexpr *expr;

	assert(expr_ptr);
	expr = *expr_ptr;
	assert(expr);

	for (i = 0; i < expr->num_children; ++i) {
		if (expr->children[i]->kind == mask_child) {
			*expr_ptr = expr->children[i];
			expr->children[i] = NULL;
			llhd_boolexpr_free(expr);
			return;
		}
		if (expr->children[i]->kind >= LLHD_BOOLEXPR_SYMBOL)
			break;
	}

	for (i = 0, o = 0; i < expr->num_children; ++i) {
		expr->children[o] = expr->children[i];
		if (expr->children[o]->kind == identity_child || (o > 0 && cmp_expr(&expr->children[o-1], &expr->children[o]) == 0)) {
			llhd_boolexpr_free(expr->children[o]);
			expr->children[o] = NULL;
		} else if (o > 0 && cmp_expr_ineg(&expr->children[o-1], &expr->children[o]) == 0) {
			llhd_boolexpr_free(expr);
			expr = alloc_boolexpr(0);
			expr->kind = mask_child;
			*expr_ptr = expr;
			return;
		} else {
			++o;
		}
	}
	for (i = o; i < expr->num_children; ++i)
		expr->children[i] = NULL;
	expr->num_children = o;

	assert(expr->num_children > 0); // above steps should never remove all children
	if (expr->num_children == 1) {
		*expr_ptr = expr->children[0];
		expr->children[0] = NULL;
		llhd_boolexpr_free(expr);
	}
}

static void
simplify(struct llhd_boolexpr **expr_ptr) {
	struct llhd_boolexpr *expr;
	assert(expr_ptr);
	expr = *expr_ptr;
	assert(expr);

	// Resolve negated constants.
	if (expr->negate) {
		if (expr->kind == LLHD_BOOLEXPR_CONST_0) {
			expr->negate = 0;
			expr->kind = LLHD_BOOLEXPR_CONST_1;
			return;
		}
		if (expr->kind == LLHD_BOOLEXPR_CONST_1) {
			expr->negate = 0;
			expr->kind = LLHD_BOOLEXPR_CONST_0;
			return;
		}
	}

	// Apply De Morgan's law.
	if (expr->negate && (expr->kind == LLHD_BOOLEXPR_AND || expr->kind == LLHD_BOOLEXPR_OR)) {
		unsigned i;
		expr->negate = 0;
		expr->kind = (expr->kind == LLHD_BOOLEXPR_AND ? LLHD_BOOLEXPR_OR : LLHD_BOOLEXPR_AND);
		for (i = 0; i < expr->num_children; ++i) {
			expr->children[i]->negate = !expr->children[i]->negate;
		}
	}

	// Simplify and sort children.
	if (expr->kind != LLHD_BOOLEXPR_SYMBOL) {
		unsigned i;
		for (i = 0; i < expr->num_children; ++i) {
			simplify(expr->children + i);
		}
		qsort(expr->children, expr->num_children, sizeof(struct llhd_boolexpr*), cmp_expr);
	}

	// Remove redundant children and detect identity and masking:
	// - a &  0 = 0
	// - a & ~a = 0
	// - a |  1 = 1
	// - a | ~a = 1
	if (expr->kind == LLHD_BOOLEXPR_AND) {
		simplify_children(expr_ptr, LLHD_BOOLEXPR_CONST_0, LLHD_BOOLEXPR_CONST_1);
		expr = *expr_ptr;
	} else if (expr->kind == LLHD_BOOLEXPR_OR) {
		simplify_children(expr_ptr, LLHD_BOOLEXPR_CONST_1, LLHD_BOOLEXPR_CONST_0);
		expr = *expr_ptr;
	}

	return;
}

void
llhd_boolexpr_disjunctive_cnf(struct llhd_boolexpr **expr_ptr) {
	struct llhd_boolexpr *expr;
	assert(expr_ptr);
	expr = *expr_ptr;
	assert(expr);

	simplify(expr_ptr);
}

static unsigned
hash_ptr(void *ptr) {
	uint64_t v = (uint64_t)ptr;
	v = v ^ (v >> 32);
	v = v ^ (v >> 16);
	return v & 0xFFFF;
}

void
llhd_boolexpr_write(struct llhd_boolexpr *expr, void(*write_fn)(void*,FILE*), FILE *out) {
	unsigned i;

	assert(expr);

	if (expr->negate)
		fputc('~', out);

	switch (expr->kind) {
		case LLHD_BOOLEXPR_CONST_0: fputc('0', out); break;
		case LLHD_BOOLEXPR_CONST_1: fputc('1', out); break;
		case LLHD_BOOLEXPR_SYMBOL:
			if (write_fn)
				write_fn(expr->children[0], out);
			else
				fprintf(out, "<%X>", hash_ptr(expr->children[0]));
			break;
		case LLHD_BOOLEXPR_OR:
			fputc('(', out);
			for (i = 0; i < expr->num_children; ++i) {
				if (i != 0) fputs(" | ", out);
				llhd_boolexpr_write(expr->children[i], write_fn, out);
			}
			fputc(')', out);
			break;
		case LLHD_BOOLEXPR_AND:
			fputc('(', out);
			for (i = 0; i < expr->num_children; ++i) {
				if (i != 0) fputs(" & ", out);
				llhd_boolexpr_write(expr->children[i], write_fn, out);
			}
			fputc(')', out);
			break;
		default:
			assert(0 && "write not implemented for kind");
			break;
	}
}


enum llhd_boolexpr_kind
llhd_boolexpr_get_kind(struct llhd_boolexpr *expr) {
	assert(expr);
	return expr->kind;
}

enum llhd_boolexpr_kind
llhd_boolexpr_is(struct llhd_boolexpr *expr, enum llhd_boolexpr_kind kind) {
	assert(expr);
	return expr->kind == kind;
}

void *
llhd_boolexpr_get_symbol(struct llhd_boolexpr *expr) {
	assert(expr && expr->kind == LLHD_BOOLEXPR_SYMBOL);
	return expr->children[0];
}

unsigned
llhd_boolexpr_get_num_children(struct llhd_boolexpr *expr) {
	assert(expr);
	if (expr->kind == LLHD_BOOLEXPR_SYMBOL)
		return 0;
	else
		return expr->num_children;
}

struct llhd_boolexpr **
llhd_boolexpr_get_children(struct llhd_boolexpr *expr) {
	assert(expr && expr->kind != LLHD_BOOLEXPR_SYMBOL);
	return expr->children;
}
