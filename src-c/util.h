// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <stdbool.h>

struct llhd_list {
	struct llhd_list *prev;
	struct llhd_list *next;
};

void llhd_list_init(struct llhd_list*);
void llhd_list_insert(struct llhd_list*, struct llhd_list*);
void llhd_list_insert_list(struct llhd_list*, struct llhd_list*);
void llhd_list_remove(struct llhd_list*);
unsigned llhd_list_length(struct llhd_list*);
bool llhd_list_empty(struct llhd_list*);

#define llhd_container_of(ptr, sample, member) \
	(__typeof__(sample))((char*)(ptr) - offsetof(__typeof__(*sample), member))
