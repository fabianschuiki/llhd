/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

class Entity;
class Process;
class Function;

class Module {
public:
	typedef std::vector<Entity*>   EntityList;
	typedef std::vector<Process*>  ProcessList;
	typedef std::vector<Function*> FunctionList;

private:
	EntityList   entities;
	ProcessList  processes;
	FunctionList functions;

public:
	~Module();

	EntityList &   getEntityList()   { return entities; }
	ProcessList &  getProcessList()  { return processes; }
	FunctionList & getFunctionList() { return functions; }

	const EntityList &   getEntityList()   const { return entities; }
	const ProcessList &  getProcessList()  const { return processes; }
	const FunctionList & getFunctionList() const { return functions; }
};

} // namespace llhd
