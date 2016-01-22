/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/type.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

class Value;

// manages memory for types and values

class Context {
	Context(const Context &) = delete;
	Context & operator=(const Context &) = delete;

public:
	Context();
	~Context();

	Type voidType;
	std::map<unsigned,LogicType*> logicTypeMap;
	std::vector<Value*> values;

	// TODO: add register/unregister module functions

private:
	// TODO: add module list and free all modules upon d'tor
};

} // namespace llhd
