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
	unsigned num_uses;
	size_t uses_offset;
};

struct llhd_binary_inst {
	struct llhd_inst super;
	int op;
	struct llhd_value *lhs;
	struct llhd_value *rhs;
	struct llhd_value_use uses[2];
};

struct llhd_compare_inst {
	struct llhd_inst super;
	int op;
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
	unsigned num_args;
	struct llhd_value **args;
	struct llhd_value_use *uses;
};

struct llhd_inst_inst {
	struct llhd_inst super;
	struct llhd_value *comp;
	unsigned num_inputs;
	unsigned num_outputs;
	struct llhd_value **params;
	struct llhd_value_use *uses;
};

struct llhd_call_inst {
	struct llhd_inst super;
	struct llhd_value *func;
	unsigned num_args;
	struct llhd_value **args;
	struct llhd_value_use *uses;
};

struct llhd_unary_inst {
	struct llhd_inst super;
	int op;
	struct llhd_value *arg;
	struct llhd_value_use use;
};

struct llhd_extract_inst {
	struct llhd_inst super;
	struct llhd_value *target;
	unsigned index;
	struct llhd_value_use use;
};

struct llhd_insert_inst {
	struct llhd_inst super;
	struct llhd_value *target;
	unsigned index;
	struct llhd_value *value;
	struct llhd_value_use uses[2];
};

struct llhd_reg_inst {
	struct llhd_inst super;
	struct llhd_value *value, *strobe;
	struct llhd_value_use uses[2];
};
