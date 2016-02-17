/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/value.hpp"

namespace llhd {

class BasicBlock;

class Instruction : public Value, public OwnedBy<BasicBlock> {
public:
	enum Opcode {
		Drive,
		Branch,
		Switch,
		Add,
		Sub,
		Mul,
		Div,
		And,
		Or,
		Xor,
		ExtractValue,
		InsertValue,
		Compare,

		BinaryFirst = Add,
		BinaryLast = Xor,
	};

	virtual ~Instruction();

	void insertAtBegin(BasicBlock * BB);
	void insertAtEnd(BasicBlock * BB);
	void insertBefore(Instruction * I);
	void insertAfter(Instruction * I);
	void removeFromParent();
	void eraseFromParent();

	Opcode getOpcode() const { return opcode; }

protected:
	Instruction(Opcode opc, Type * type, BasicBlock * parent = nullptr);

private:
	Opcode opcode;
};

} // namespace llhd
