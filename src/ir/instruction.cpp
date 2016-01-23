/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/basicblock.hpp"

namespace llhd {

Instruction::Instruction(Opcode opc, Type * type, BasicBlock * parent):
	Value(Value::InstructionId, type),
	OwnedBy(parent),
	opcode(opc) {
}

Instruction::~Instruction() {
}

void Instruction::insertAtBegin(BasicBlock * BB) {
	llhd_assert(!parent);
	llhd_assert(BB);
	BB->getInstList().insert(BB->getInstList().begin(), this);
	parent = BB;
}

void Instruction::insertAtEnd(BasicBlock * BB) {
	llhd_assert(!parent);
	llhd_assert(BB);
	BB->getInstList().push_back(this);
	parent = BB;
}

void Instruction::insertBefore(Instruction * I) {
	llhd_assert(!parent);
	llhd_assert(I);
	auto * BB = I->getParent();
	auto & IL = BB->getInstList();
	auto it = std::find(IL.begin(), IL.end(), I);
	llhd_assert(it != IL.end());
	IL.insert(it, this);
	parent = BB;
}

void Instruction::insertAfter(Instruction * I) {
	llhd_assert(!parent);
	llhd_assert(I);
	auto * BB = I->getParent();
	auto & IL = BB->getInstList();
	auto it = std::find(IL.begin(), IL.end(), I);
	llhd_assert(it != IL.end());
	++it;
	IL.insert(it, this);
	parent = BB;
}

void Instruction::removeFromParent() {
	llhd_assert(parent);
	auto & IL = parent->getInstList();
	auto it = std::find(IL.begin(), IL.end(), this);
	llhd_assert(it != IL.end());
	IL.erase(it);
}

void Instruction::eraseFromParent() {
	llhd_assert(parent);
	removeFromParent();
	delete this;
}

} // namespace llhd
