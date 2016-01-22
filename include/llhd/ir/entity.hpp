/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

class Module;

class Entity {
	Module * M;
	std::string name;

public:
	Entity(Module * parent, const std::string & name);

	const std::string & getName() const { return name; }
};

} // namespace llhd
