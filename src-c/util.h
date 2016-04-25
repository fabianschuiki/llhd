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
void llhd_buffer_free(struct llhd_buffer*);
void *llhd_buffer_append(struct llhd_buffer*, size_t, void*);
