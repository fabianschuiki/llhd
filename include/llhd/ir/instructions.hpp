/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/ir/instruction.hpp"

namespace llhd {

class DriveInst : public Instruction {
public:
	DriveInst(Value * target, Value * value, BasicBlock * parent = nullptr);

	Value * getTarget() { return target; }
	Value * getValue() { return value; }

	const Value * getTarget() const { return target; }
	const Value * getValue() const { return value; }

private:
	Value * target;
	Value * value;
};


class BranchInst : public Instruction {
public:
	BranchInst(Value * ifTrue, Value * ifFalse, Value * cond, BasicBlock * parent = nullptr);

	Value * getIfTrue() { return ifTrue; }
	Value * getIfFalse() { return ifFalse; }
	Value * getCondition() { return condition; }

	const Value * getIfTrue() const { return ifTrue; }
	const Value * getIfFalse() const { return ifFalse; }
	const Value * getCondition() const { return condition; }

private:
	Value * ifTrue;
	Value * ifFalse;
	Value * condition;
};


class SwitchInst : public Instruction {
public:
	typedef std::pair<Value*,Value*> Destination;
	typedef std::vector<Destination> DestinationList;

	SwitchInst(Value * value, Value * otherwise, BasicBlock * parent = nullptr);

	Value * getValue() { return value; }
	DestinationList getDestinationList() { return destinations; }
	Value * getOtherwise() { return otherwise; }

	const Value * getValue() const { return value; }
	const DestinationList & getDestinationList() const { return destinations; }
	const Value * getOtherwise() const { return otherwise; }

	void addDestination(Value * val, Value * dst);

private:
	Value * value;
	DestinationList destinations;
	Value * otherwise;
};


class AddInst : public Instruction {
public:
	AddInst(Value * lhs, Value * rhs, BasicBlock * parent = nullptr);

private:
	Value * lhs;
	Value * rhs;
};


class SubInst : public Instruction {
public:
	SubInst(Value * lhs, Value * rhs, BasicBlock * parent = nullptr);

private:
	Value * lhs;
	Value * rhs;
};


class ExtractValueInst : public Instruction {
public:
	ExtractValueInst(Value * target, Value * index, unsigned length = 0, BasicBlock * parent = nullptr);

private:
	Value * target;
	Value * index;
	unsigned length; // optional
};


class InsertValueInst : public Instruction {
public:
	InsertValueInst(Value * target, Value * value, Value * index, unsigned length = 0, BasicBlock * parent = nullptr);

private:
	Value * target;
	Value * value;
	Value * index;
	unsigned length; // optional
};


class CompareInst : public Instruction {
public:
	enum Op {
		EQ,  // equal
		NE,  // not equal
		UGT, // unsigned greater
		ULT, // unsigned less
		UGE, // unsigned greater or equal
		ULE, // unsigned less or equal
		SGT, // signed greater
		SLT, // signed less
		SGE, // signed greater or equal
		SLE, // signed less or equal
	};

	CompareInst(Op op, Value * lhs, Value * rhs, BasicBlock * parent = nullptr);

private:
	Op op;
	Value * lhs;
	Value * rhs;
};

} // namespace llhd
