// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include "util.h"
#include <llhd.h>
#include <stdint.h>

struct llhd_value_use {
	struct llhd_list link;
	struct llhd_value *user;
	int arg;
};

/// @todo Add a type field.
struct llhd_value {
	struct llhd_value_vtbl *vtbl;
	/// @todo Make rc atomic.
	int rc;
	struct llhd_list users;
};

struct llhd_const_int {
	struct llhd_value super;
	struct llhd_type *type;
	uint64_t value;
};

struct llhd_unit {
	struct llhd_value super;
	unsigned num_inputs;
	unsigned num_outputs;
	struct llhd_param **params;
	struct llhd_list link;
	struct llhd_module *module;
};

struct llhd_entity {
	struct llhd_unit super;
	char *name;
	struct llhd_type *type;
	struct llhd_list insts;
};

struct llhd_proc {
	struct llhd_unit super;
	char *name;
	struct llhd_type *type;
	struct llhd_list blocks;
};

struct llhd_block {
	struct llhd_value super;
	struct llhd_value *parent;
	struct llhd_list link;
	char *name;
	struct llhd_type *type;
	struct llhd_list insts;
};


struct llhd_value_vtbl {
	int kind;
	size_t name_offset;
	size_t type_offset;
	void (*dispose_fn)(void*);
	void (*substitute_fn)(void*, void*, void*);
	void (*add_inst_fn)(void*, struct llhd_value*, int);
	void (*remove_inst_fn)(void*, struct llhd_value*);
	void (*add_block_fn)(void*, struct llhd_block*, int);
	void (*remove_block_fn)(void*, struct llhd_block*);
	void (*unlink_from_parent_fn)(void*);
	void (*unlink_uses_fn)(void*);
};

struct llhd_unit_vtbl {
	struct llhd_value_vtbl super;
	int kind;
	size_t block_list_offset;
};

struct llhd_const_vtbl {
	struct llhd_value_vtbl super;
	int kind;
	char *(*to_string_fn)(void*);
};


void *llhd_alloc_value(size_t,void*);
void llhd_value_use(struct llhd_value*, struct llhd_value_use*);
void llhd_value_unuse(struct llhd_value_use*);

void *llhd_alloc_unit(size_t,void*,unsigned);
