/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/ir/instruction.hpp"

namespace llhd {

class DriveInst : public Instruction {
public:
	DriveInst(Value * target, Value * value, BasicBlock * parent = nullptr);

	Value * getTarget() { return target; }
	const Value * getTarget() const { return target; }

	Value * getValue() { return value; }
	const Value * getValue() const { return value; }

private:
	Value * target;
	Value * value;
};

} // namespace llhd
