/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/basicblock.hpp"
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/value.hpp"
#include "llhd/ir/constant.hpp"
#include "llhd/ir/constants.hpp"
#include "llhd/ir/process.hpp"
#include "llhd/sim/instexecutor.hpp"

namespace llhd {

InstExecutor::InstExecutor(Process * P):
	P(P),
	ins(nullptr),
	insIdx(0) {

	llhd_assert(P);
	if (!P->getBasicBlockList().empty()) {
		BasicBlock * BB = P->getBasicBlockList().front();
		if (!BB->getInstList().empty())
			ins = BB->getInstList().front();
	}
}

void InstExecutor::setValue(Value * target, Constant * value) {
	valueMap[target] = value;
}

Constant * InstExecutor::lookup(Value * value) {
	if (value->getValueId() == Value::ConstantId)
		return static_cast<Constant*>(value);
	// llhd_assert_msg(value->getValueId() == Value::InstructionId, "can only lookup instruction values");
	auto it = valueMap.find(value);
	llhd_assert_msg(it != valueMap.end(), "no value calculated");
	return it->second;
}

void InstExecutor::run() {
	while (ins)
		step();
}

static bool valuesEqual(Constant * A, Constant * B) {
	if (auto AA = dynamic_cast<ConstantLogic*>(A)) {
		auto BB = dynamic_cast<ConstantLogic*>(B);
		if (!BB) return false;
		auto r = AA->getValue() == BB->getValue();
		tfm::printf("comparing \"%s\" == \"%s\" (%s)\n", AA->getValue().toString(), BB->getValue().toString(), r ? "true" : "false");
		return r;
	}
	if (auto AA = dynamic_cast<ConstantInteger*>(A)) {
		auto BB = dynamic_cast<ConstantInteger*>(B);
		if (!BB) return false;
		llhd_abort_msg("valuesEqual for ConstantInteger not implemented");
		return true;
	}
}

void InstExecutor::step() {
	if (!ins)
		return;
	BasicBlock * nextBB = nullptr;

	tfm::printf("executing ins %d\n", ins->getOpcode());

	switch (ins->getOpcode()) {
		case Instruction::Drive: {
			tfm::printf("driving %p\n", dynamic_cast<DriveInst*>(ins)->getTarget());
		} break;

		case Instruction::Switch: {
			auto * I = dynamic_cast<SwitchInst*>(ins);
			tfm::printf("switching on %p\n", I->getValue());
			for (auto p : I->getDestinationList()) {
				if (valuesEqual(lookup(I->getValue()), lookup(p.first))) {
					nextBB = p.second;
					break;
				}
			}
			if (!nextBB && I->getOtherwise())
				nextBB = I->getOtherwise();
			llhd_assert_msg(nextBB, "switch must contain a destination for the input value");
		} break;

		default:
			llhd_abort_msg("invalid opcode");
	}

	if (nextBB) {
		insIdx = 0;
		if (!nextBB->getInstList().empty())
			ins = nextBB->getInstList().front();
		else
			ins = nullptr;
	} else {
		++insIdx;
		BasicBlock * BB = ins->getParent();
		if (insIdx >= BB->getInstList().size())
			llhd_abort_msg("basic block is missing a terminator");
		ins = BB->getInstList()[insIdx];
	}
}

} // namespace llhd
