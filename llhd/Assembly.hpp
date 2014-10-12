/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace llhd {

class AssemblySlot {
public:
	enum Direction {
		kSignal,
		kRegister,
		kPortIn,
		kPortOut
	};

	Direction dir;
	std::string name;
	std::string type;
	// AssemblyExpr* assignment;
};

class AssemblyModule {
public:
	std::string name;
	std::map<std::string, std::shared_ptr<const AssemblySlot>> slots;
};

class Assembly {
public:
	std::map<std::string, std::shared_ptr<const AssemblyModule>> modules;
};

} // namespace llhd
