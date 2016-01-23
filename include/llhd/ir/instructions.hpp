/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/ir/instruction.hpp"

namespace llhd {

class DriveInst : public Instruction {
public:
	DriveInst(Value * target, Value * value);

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
	BranchInst(BasicBlock * ifTrue, BasicBlock * ifFalse, Value * cond);

	BasicBlock * getIfTrue() { return ifTrue; }
	BasicBlock * getIfFalse() { return ifFalse; }
	Value * getCondition() { return condition; }

	const BasicBlock * getIfTrue() const { return ifTrue; }
	const BasicBlock * getIfFalse() const { return ifFalse; }
	const Value * getCondition() const { return condition; }

private:
	BasicBlock * ifTrue;
	BasicBlock * ifFalse;
	Value * condition;
};


class SwitchInst : public Instruction {
public:
	typedef std::pair<Value*,BasicBlock*> Destination;
	typedef std::vector<Destination> DestinationList;

	SwitchInst(Value * value, BasicBlock * otherwise);

	Value * getValue() { return value; }
	DestinationList getDestinationList() { return destinations; }
	BasicBlock * getOtherwise() { return otherwise; }

	const Value * getValue() const { return value; }
	const DestinationList & getDestinationList() const { return destinations; }
	const BasicBlock * getOtherwise() const { return otherwise; }

	void addDestination(Value * val, BasicBlock * dst);

private:
	Value * value;
	DestinationList destinations;
	BasicBlock * otherwise;
};


class BinaryInst : public Instruction {
public:
	BinaryInst(Opcode opc, Value * lhs, Value * rhs);

private:
	Value * lhs;
	Value * rhs;
};


class ExtractValueInst : public Instruction {
public:
	ExtractValueInst(Value * target, Value * index, unsigned length = 0);

private:
	Value * target;
	Value * index;
	unsigned length; // optional
};


class InsertValueInst : public Instruction {
public:
	InsertValueInst(Value * target, Value * value, Value * index, unsigned length = 0);

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

	CompareInst(Op op, Value * lhs, Value * rhs);

private:
	Op op;
	Value * lhs;
	Value * rhs;
};

} // namespace llhd
