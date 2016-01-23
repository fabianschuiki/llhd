/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/basicblock.hpp"
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/type.hpp"

namespace llhd {

BasicBlock::BasicBlock(Context & C, const std::string & name, Process * parent):
	Value(Value::BasicBlockId, Type::getVoidType(C)),
	OwnedBy(parent),
	name(name) {
}

BasicBlock::~BasicBlock() {
	for (auto I : instructions)
		delete I;
}

} // namespace llhd
