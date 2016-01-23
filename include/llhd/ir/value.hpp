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
		ConstantId,
		InstructionId,
	};

	virtual ~Value() {}

	Type * getType() const { return type; }
	Context & getContext() const;
	ValueId getValueId() const { return vid; }

	const std::string & getName() const { return name; }
	void setName(const std::string & name) { this->name = name; }

	static Value * getConst(Type * type, const std::string & str);
	static Value * getConstNull(Type * type);

protected:
	Value & operator= (const Value &) = delete;
	Value(const Value &) = delete;

	Value(ValueId vid, Type * type);

private:
	ValueId vid;
	Type * type;
	std::string name;
};

} // namespace llhd
