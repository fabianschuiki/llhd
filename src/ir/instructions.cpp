/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/type.hpp"

namespace llhd {

DriveInst::DriveInst(Value * target, Value * value, BasicBlock * parent):
	Instruction(Type::getVoidType(target->getContext()), parent),
	target(target),
	value(value) {
}

} // namespace llhd
