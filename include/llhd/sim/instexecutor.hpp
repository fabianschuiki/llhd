/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

class Constant;
class Instruction;
class Process;
class Value;

class InstExecutor {
public:
	InstExecutor(Process * P);
	void setValue(Value * target, Constant * value);
	void run();
	void step();

private:
	Process * P;
	Instruction * ins;
	unsigned insIdx;

	std::map<Value*,Constant*> valueMap;
	Constant * lookup(Value * value);
};

} // namespace llhd
