/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/type.hpp"

namespace llhd {

class ConstValue;
class Context;

class Value {
public:
	enum ValueId {
		EntityId,
		ProcessId,
		FunctionId,
		ArgumentId,
		BasicBlockId,
	};

	Type * getType() const { return type; }
	Context & getContext() const;

	const std::string & getName() const { return name; }
	void setName(const std::string & name) { this->name = name; }

	static Value * getConstNull(Type * type);

protected:
	Value & operator= (const Value &) = delete;
	Value(const Value &) = delete;

	Value(Type * type);

private:
	Type * type;
	std::string name;
};

} // namespace llhd
