/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/value.hpp"

namespace llhd {

class Process;
class Type;

// owned by parent
class Argument : public Value, public OwnedBy<Process> {
	std::string name;
	Type * type;

public:
	Argument(const std::string & name, Type * type, Process * parent);

	const std::string & getName() const { return name; }
	Type * getType() const { return type; }
};

} // namespace llhd
