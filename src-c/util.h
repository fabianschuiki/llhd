// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdbool.h>
#include <stddef.h>

struct llhd_list {
	struct llhd_list *prev;
	struct llhd_list *next;
};

struct llhd_array {
	void *data;
	unsigned item_size;
	unsigned size;
	unsigned cap;
};

struct llhd_buffer {
	void *data;
	size_t size;
	size_t cap;
};

struct llhd_ptrset {
	void **data;
	unsigned num;
	unsigned cap;
};

struct llhd_ptrmap {
	void **keys, **values;
	unsigned num;
	unsigned cap;
};

void llhd_list_init(struct llhd_list*);
void llhd_list_insert(struct llhd_list*, struct llhd_list*);
void llhd_list_insert_list(struct llhd_list*, struct llhd_list*);
void llhd_list_remove(struct llhd_list*);
unsigned llhd_list_length(struct llhd_list*);
bool llhd_list_empty(struct llhd_list*);

#define llhd_container_of(ptr, sample, member) \
	(__typeof__(sample))((void*)(ptr) - offsetof(__typeof__(*sample), member))
#define llhd_container_of2(ptr, type, member) \
	(type*)((void*)(ptr) - offsetof(type, member))

void llhd_buffer_init(struct llhd_buffer*, size_t);
void llhd_buffer_dispose(struct llhd_buffer*);
void *llhd_buffer_append(struct llhd_buffer*, size_t, void*);

void llhd_ptrset_init(struct llhd_ptrset*, size_t);
void llhd_ptrset_dispose(struct llhd_ptrset*);
bool llhd_ptrset_insert(struct llhd_ptrset*, void*);
bool llhd_ptrset_remove(struct llhd_ptrset*, void*);
bool llhd_ptrset_has(struct llhd_ptrset*, void*);

void llhd_ptrmap_init(struct llhd_ptrmap*, size_t);
void llhd_ptrmap_dispose(struct llhd_ptrmap*);
void **llhd_ptrmap_expand(struct llhd_ptrmap*, void*);
void *llhd_ptrmap_set(struct llhd_ptrmap*, void*, void*);
void *llhd_ptrmap_get(struct llhd_ptrmap*, void*);
void *llhd_ptrmap_remove(struct llhd_ptrmap*, void*);
