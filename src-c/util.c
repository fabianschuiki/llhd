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

void *
llhd_realloc(void *ptr, size_t sz) {
	return realloc(ptr, sz);
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

void
llhd_buffer_init(struct llhd_buffer *buf, size_t cap) {
	memset(buf, 0, sizeof(struct llhd_buffer));
	if (cap < 16)
		cap = 16;
	buf->cap = cap;
	buf->data = llhd_alloc(cap);
}

void
llhd_buffer_dispose(struct llhd_buffer *buf) {
	if (buf->data)
		llhd_free(buf->data);
	memset(buf, 0, sizeof(struct llhd_buffer));
}

void *
llhd_buffer_append(struct llhd_buffer *buf, size_t size, void *data) {
	void *ptr = buf->data + buf->size;
	size_t req = buf->size + size;

	if (req > buf->cap) {
		buf->cap *= 2;
		if (buf->cap < req)
			buf->cap = req;
		buf->data = llhd_realloc(buf->data, buf->cap);
	}

	buf->size += size;
	if (data) {
		memcpy(ptr, data, size);
	}
	return ptr;
}



void
llhd_ptrset_init(struct llhd_ptrset *ps, size_t cap) {
	assert(ps);
	memset(ps, 0, sizeof(*ps));
	ps->cap = cap;
	if (cap > 0) {
		ps->data = llhd_alloc(ps->cap * sizeof(void*));
	}
}

void
llhd_ptrset_dispose(struct llhd_ptrset *ps) {
	assert(ps);
	if (ps->data) {
		llhd_free(ps->data);
	}
	memset(ps, 0, sizeof(*ps));
}

/**
 * Based on Linux' implementation of bsearch, see [1].
 *
 * [1]: http://lxr.free-electrons.com/source/lib/bsearch.c
 */
static unsigned
bsearch_ptr(void **haystack, unsigned num, void *needle) {
	unsigned start = 0, end = num;
	while (start < end) {
		unsigned mid = start + (end - start) / 2;
		if (needle < haystack[mid]) {
			end = mid;
		} else if (needle > haystack[mid]) {
			start = mid + 1;
		} else {
			return mid;
		}
	}
	return start;
}

bool
llhd_ptrset_insert(struct llhd_ptrset *ps, void *ptr) {
	unsigned idx, i;
	assert(ps);

	idx = bsearch_ptr(ps->data, ps->num, ptr);
	if (idx < ps->num && ps->data[idx] == ptr) {
		return false;
	}

	if (ps->num == ps->cap) {
		ps->cap *= 2;
		ps->data = llhd_realloc(ps->data, ps->cap * sizeof(void*));
	}

	for (i = ps->num; i > idx; --i) {
		ps->data[i] = ps->data[i-1];
	}
	ps->data[idx] = ptr;
	++ps->num;
	return true;
}

bool
llhd_ptrset_remove(struct llhd_ptrset *ps, void *ptr) {
	unsigned idx, i;
	assert(ps);

	idx = bsearch_ptr(ps->data, ps->num, ptr);
	if (idx == ps->num || ps->data[idx] != ptr) {
		return false;
	}

	--ps->num;
	for (i = idx; i < ps->num; ++i) {
		ps->data[i] = ps->data[i+1];
	}
	return true;
}

bool
llhd_ptrset_has(struct llhd_ptrset *ps, void *ptr) {
	unsigned idx;
	assert(ps);

	idx = bsearch_ptr(ps->data, ps->num, ptr);
	return idx < ps->num && ps->data[idx] == ptr;
}


void
llhd_ptrmap_init(struct llhd_ptrmap *pm, size_t cap) {
	assert(pm);
	memset(pm, 0, sizeof(*pm));
	pm->cap = cap;
	if (pm->cap) {
		size_t sz = pm->cap * sizeof(void*);
		pm->keys = llhd_alloc(sz);
		pm->values = llhd_alloc(sz);
	}
}

void
llhd_ptrmap_dispose(struct llhd_ptrmap *pm) {
	assert(pm);
	if (pm->keys) {
		llhd_free(pm->keys);
	}
	if (pm->values) {
		llhd_free(pm->values);
	}
	memset(pm, 0, sizeof(*pm));
}

/**
 * Get a pointer to a value with a given key. If no such key exists in the map,
 * one is created and its value set to @c NULL.
 *
 * @warning The returned pointer is only valid until the next call to
 *          llhd_ptrmap_expand, llhd_ptrmap_set, or llhd_ptrmap_remove.
 *
 * @return A pointer to the value for the given key. The pointed-to value is
 *         @c NULL if the key did not exist in the map prior to calling this
 *         function, or its value was set to @c NULL explicitly.
 */
void **
llhd_ptrmap_expand(struct llhd_ptrmap *pm, void *key) {
	unsigned idx, i;
	assert(pm);

	idx = bsearch_ptr(pm->keys, pm->num, key);
	if (idx < pm->num && pm->keys[idx] == key) {
		return pm->values + idx;
	}

	if (pm->num == pm->cap) {
		size_t sz;
		pm->cap *= 2;
		sz = pm->cap * sizeof(void*);
		pm->keys = llhd_realloc(pm->keys, sz);
		pm->values = llhd_realloc(pm->values, sz);
	}

	for (i = pm->num; i > idx; --i) {
		pm->keys[i] = pm->keys[i-1];
		pm->values[i] = pm->values[i-1];
	}
	pm->keys[idx] = key;
	pm->values[idx] = NULL;
	++pm->num;
	return pm->values + idx;
}

/**
 * Insert a value into a ptrmap.
 *
 * @return The value that was replaced, or @c NULL if no value existed for the
 *         given key.
 */
void *
llhd_ptrmap_set(struct llhd_ptrmap *pm, void *key, void *value) {
	void **slot, *rem;
	slot = llhd_ptrmap_expand(pm, key);
	rem = *slot;
	*slot = value;
	return rem;
}

/**
 * Lookup the value for a given key in a ptrmap.
 *
 * @return The value for the given key if it exists, or @c NULL otherwise.
 */
void *
llhd_ptrmap_get(struct llhd_ptrmap *pm, void *key) {
	unsigned idx;
	assert(pm);
	idx = bsearch_ptr(pm->keys, pm->num, key);
	return (idx < pm->num && pm->keys[idx] == key) ? pm->values[idx] : NULL;
}

/**
 * Remove a value from a ptrmap.
 *
 * @return The value that was removed, or @c NULL if no value existed for the
 *         given key.
 */
void *
llhd_ptrmap_remove(struct llhd_ptrmap *pm, void *key) {
	unsigned idx, i;
	void *rem;
	assert(pm);

	idx = bsearch_ptr(pm->keys, pm->num, key);
	if (idx == pm->num || pm->keys[idx] != key) {
		return NULL;
	}

	rem = pm->values[idx];
	--pm->num;
	for (i = idx; i < pm->num; ++i) {
		pm->keys[i] = pm->keys[i+1];
		pm->values[i] = pm->values[i+1];
	}
	return rem;
}
