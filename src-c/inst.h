// Copyright (c) 2016 Fabian Schuiki
#pragma once
#include <llhd.h>

struct llhd_inst {
	struct llhd_value super;
	struct llhd_value *parent;
	struct llhd_list link;
	struct llhd_type *type;
	char *name;
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

struct llhd_compare_inst {
	struct llhd_inst super;
	enum llhd_compare_op op;
	struct llhd_value *lhs;
	struct llhd_value *rhs;
	struct llhd_value_use uses[2];
};

struct llhd_branch_inst {
	struct llhd_inst super;
	struct llhd_value *cond;
	struct llhd_block *dst1;
	struct llhd_block *dst0;
	struct llhd_value_use uses[3];
};

struct llhd_drive_inst {
	struct llhd_inst super;
	struct llhd_value *sig;
	struct llhd_value *val;
	struct llhd_value_use uses[2];
};

struct llhd_ret_inst {
	struct llhd_inst super;
};
