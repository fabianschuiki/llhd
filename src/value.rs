// Copyright (c) 2017 Fabian Schuiki

/// A reference to a value in a module.
pub enum ValueRef {
	Instruction,
	BasicBlock,
	Argument,
	Global,
	Constant,
}
