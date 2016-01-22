/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/instruction.hpp"

namespace llhd {

Instruction::Instruction(Type * type, BasicBlock * parent):
	Value(type),
	OwnedBy(parent) {
}

Instruction::~Instruction() {
}

} // namespace llhd
