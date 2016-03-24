// Copyright (c) 2016 Fabian Schuiki
#include "util.h"
#include <llhd.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>

void *
llhd_alloc(size_t sz) {
	void *ptr = malloc(sz);
	assert(ptr);
	return ptr;
}

void *
llhd_zalloc(size_t sz) {
	void *ptr = llhd_alloc(sz);
	memset(ptr, 0, sz);
	return ptr;
}

void
llhd_free(void *ptr) {
	free(ptr);
}

/**
 * Initializes a list. Only call this function on links that represent the list
 * as a whole, not on individual elements.
 *
 * @memberof llhd_list
 */
void
llhd_list_init(struct llhd_list *list) {
	assert(list);
	list->prev = list;
	list->next = list;
}

void
llhd_list_insert(struct llhd_list *list, struct llhd_list *elm) {
	assert(list && elm);
	assert(list->next && list->prev);
	elm->prev = list;
	elm->next = list->next;
	list->next = elm;
	elm->next->prev = elm;
}

void
llhd_list_insert_list(struct llhd_list *list, struct llhd_list *other) {
	assert(list && other);
	assert(list->next && list->prev && other->next && other->prev);
	if (llhd_list_empty(other))
		return;
	other->next->prev = list;
	other->prev->next = list->next;
	list->next->prev = other->prev;
	list->next = other->next;
}

void
llhd_list_remove(struct llhd_list *elm) {
	assert(elm && elm->prev && elm->next);
	elm->prev->next = elm->next;
	elm->next->prev = elm->prev;
	elm->prev = NULL;
	elm->next = NULL;
}

unsigned
llhd_list_length(struct llhd_list *list) {
	struct llhd_list *e;
	unsigned count;

	count = 0;
	e = list->next;
	while (e != list) {
		e = e->next;
		++count;
	}

	return count;
}

/**
 * Checks whether a list is empty.
 *
 * @memberof llhd_list
 */
bool
llhd_list_empty(struct llhd_list *list) {
	assert(list);
	return list->next == list;
}
