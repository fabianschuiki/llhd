/* Copyright (c) 2016 Fabian Schuiki */
#include "value.h"
#include "util.h"
#include "module.h"

#include <assert.h>
#include <string.h>


struct llhd_module *
llhd_module_new(const char *name) {
	struct llhd_module *M;
	M = llhd_zalloc(sizeof(*M));
	M->name = name ? strdup(name) : NULL;
	llhd_list_init(&M->units);
	return M;
}

void
llhd_module_free(struct llhd_module *M) {
	assert(M);
	llhd_free(M);
}

struct llhd_value *
llhd_module_get_first_unit(struct llhd_module *M) {
	assert(M);
	if (M->units.next == &M->units)
		return NULL;
	return (struct llhd_value *)llhd_container_of2(M, struct llhd_unit, link);
}

struct llhd_value *
llhd_module_get_last_unit(struct llhd_module *M) {
	assert(M);
	if (M->units.prev == &M->units)
		return NULL;
	return (struct llhd_value *)llhd_container_of2(M, struct llhd_unit, link);
}

struct llhd_list *
llhd_module_get_units(struct llhd_module *M) {
	assert(M);
	return &M->units;
}


const char *
llhd_module_get_name(struct llhd_module *M) {
	assert(M);
	return M->name;
}

