// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <llhd.h>

struct llhd_inst {
	struct llhd_value super;
	struct llhd_value *parent;
	struct llhd_list link;
	struct llhd_type *type;
};

struct llhd_inst_vtbl {
	struct llhd_value_vtbl super;
	int kind;
};

struct llhd_binary_inst {
	struct llhd_inst super;
	enum llhd_binary_op op;
	struct llhd_value *lhs;
	struct llhd_value *rhs;
	struct llhd_value_use uses[2];
};
