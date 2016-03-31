// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include "util.h"
#include <llhd.h>

struct llhd_value_vtbl {
	int kind;
	size_t name_offset;
	size_t type_offset;
	void (*dispose_fn)(void*);
	void (*substitute_fn)(void*,void*,void*);
	void (*add_inst_fn)(void*,struct llhd_value*,int);
};

struct llhd_value_use {
	struct llhd_list link;
	struct llhd_value *user;
	int arg;
};

struct llhd_value {
	struct llhd_value_vtbl *vtbl;
	/// @todo Make rc atomic.
	int rc;
	struct llhd_list users;
};

void *llhd_alloc_value(size_t,void*);
void llhd_value_use(struct llhd_value*, struct llhd_value_use*);
void llhd_value_unuse(struct llhd_value_use*);
