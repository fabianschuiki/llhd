/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace llhd {

class AssemblyType {
public:
	virtual ~AssemblyType() {}
};

class AssemblyTypeLogic : public AssemblyType {};

class AssemblyTypeWord : public AssemblyType {
public:
	unsigned width;
	std::shared_ptr<AssemblyType> type;
};

class AssemblySignal {
public:
	enum Direction {
		kSignal,
		kRegister,
		kPortIn,
		kPortOut
	};

	Direction dir;
	std::string name;
	std::shared_ptr<AssemblyType> type;
	// AssemblyExpr* assignment;
};

class AssemblyModule {
public:
	std::string name;
	std::map<std::string, std::shared_ptr<const AssemblySignal>> signals;
};

class Assembly {
public:
	std::map<std::string, std::shared_ptr<const AssemblyModule>> modules;
};

} // namespace llhd
