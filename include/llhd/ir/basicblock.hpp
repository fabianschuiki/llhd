/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/value.hpp"

namespace llhd {

class Instruction;
class Process;

// owned by parent
class BasicBlock : public Value, public OwnedBy<Process> {
public:
	typedef std::vector<Instruction*> InstList;

	BasicBlock(Context & C, const std::string & name, Process * parent = nullptr);
	~BasicBlock();

	const std::string & getName() const { return name; }

	InstList & getInstList() { return instructions; }
	const InstList & getInstList() const { return instructions; }

private:
	std::string name;
	InstList instructions;
};

} // namespace llhd
