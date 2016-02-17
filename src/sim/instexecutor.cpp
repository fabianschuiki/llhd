/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/basicblock.hpp"
#include "llhd/ir/instruction.hpp"
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/value.hpp"
#include "llhd/ir/constant.hpp"
#include "llhd/ir/constants.hpp"
#include "llhd/ir/process.hpp"
#include "llhd/sim/instexecutor.hpp"

// TODO:
// - implement all instructions
//   - compare
//   - sub
//   - mul
//   - div
//   - and
//   - or
//   - xor
// - switch to multiple value maps, one for each value type

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
	llhd_abort_msg("invalid arguments to valuesEqual");
	return false;
}

void InstExecutor::step() {
	if (!ins)
		return;
	BasicBlock * nextBB = nullptr;

	tfm::printf("executing ins %d\n", ins->getOpcode());

	if (ins->getOpcode() >= Instruction::BinaryFirst &&
		ins->getOpcode() <= Instruction::BinaryLast) {
		execBinary(static_cast<BinaryInst*>(ins));
	} else switch (ins->getOpcode()) {
		case Instruction::Drive: execDrive(static_cast<DriveInst*>(ins)); break;
		case Instruction::Switch: nextBB = execSwitch(static_cast<SwitchInst*>(ins)); break;
		case Instruction::InsertValue: execInsertValue(static_cast<InsertValueInst*>(ins)); break;
		case Instruction::ExtractValue: execExtractValue(static_cast<ExtractValueInst*>(ins)); break;
		case Instruction::Branch: nextBB = execBranch(static_cast<BranchInst*>(ins)); break;

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


void InstExecutor::execDrive(DriveInst * I) {
	tfm::printf("driving %p\n", I->getTarget());
}

BasicBlock * InstExecutor::execSwitch(SwitchInst * I) {
	tfm::printf("switching on %p\n", I->getValue());
	for (auto p : I->getDestinationList())
		if (valuesEqual(lookup(I->getValue()), lookup(p.first)))
			return p.second;
	if (I->getOtherwise())
		return I->getOtherwise();
	llhd_abort_msg("SwitchInst must cover all input values or provide otherwise destination");
	return nullptr;
}

void InstExecutor::execInsertValue(InsertValueInst * I) {
	tfm::printf("inserting value %p into %p\n", I->getValue(), I->getTarget());
	auto kind = I->getType()->getTypeId();
	if (kind == Type::LogicTypeId) {
		auto type = static_cast<LogicType*>(I->getType());
		// TODO: assert that the looked-up values are actually instances of ConstantLogic
		Logic target = static_cast<ConstantLogic*>(lookup(I->getTarget()))->getValue();
		Logic value  = static_cast<ConstantLogic*>(lookup(I->getValue()))->getValue();
		unsigned index = static_cast<ConstantInteger*>(lookup(I->getIndex()))->getValue();
		unsigned len = I->getLength();
		tfm::printf("insertvalue '%s' '%s' %d %d = ", target.toString(), value.toString(), index, len);
		llhd_assert(index+len <= target.getWidth());
		for (unsigned i = 0; i < len; ++i)
			target.set(index+i, value.get(i));
		tfm::printf("%s\n", target.toString());
		valueMap[I] = ConstantLogic::get(type, target);
	} else {
		llhd_abort_msg("invalid type for InsertValueInst");
	}
}

void InstExecutor::execExtractValue(ExtractValueInst * I) {
	auto kind = I->getType()->getTypeId();
	if (kind == Type::LogicTypeId) {
		auto type = static_cast<LogicType*>(I->getType());
		// TODO: assert that the looked-up values are actually instances of ConstantLogic
		Logic target = static_cast<ConstantLogic*>(lookup(I->getTarget()))->getValue();
		unsigned index = static_cast<ConstantInteger*>(lookup(I->getIndex()))->getValue();
		unsigned len = I->getLength();
		llhd_assert(index+len <= target.getWidth());
		Logic result(len);
		for (unsigned i = 0; i < len; ++i)
			result.set(i, target.get(index+i));
		tfm::printf("extractvalue '%s' %d %d = %s\n", target.toString(), index, len, result.toString());
		valueMap[I] = ConstantLogic::get(type, result);
	} else {
		llhd_abort_msg("invalid type for ExtractValueInst");
	}
}

static Logic evalBinary(Instruction::Opcode opcode, const Logic & lhs, const Logic & rhs) {
	llhd_assert(lhs.getWidth() == rhs.getWidth());
	switch (opcode) {
		case Instruction::Add: {
			Logic result(lhs.getWidth(), Logic::X);
			int carry = 0;
			for (unsigned i = 0; i < lhs.getWidth(); ++i) {
				auto blhs = lhs.get(i);
				auto brhs = rhs.get(i);
				int vlhs, vrhs;

				if (blhs == Logic::O || blhs == Logic::L)
					vlhs = 0;
				else if (blhs == Logic::I || blhs == Logic::H)
					vlhs = 1;
				else
					break;

				if (brhs == Logic::O || brhs == Logic::L)
					vrhs = 0;
				else if (brhs == Logic::I || brhs == Logic::H)
					vrhs = 1;
				else
					break;

				int s = vlhs + vrhs + carry;
				result.set(i, s & 0x1 ? Logic::I : Logic::O);
				carry = s & 0x2 ? 1 : 0;
			}
			return result;
		} break;
		default:
			llhd_abort_msg("BinaryInst opcode not supported for logic");
			return Logic(0);
	}
}

void InstExecutor::execBinary(BinaryInst * I) {
	auto kind = I->getType()->getTypeId();
	if (kind == Type::LogicTypeId) {
		auto type = static_cast<LogicType*>(I->getType());
		// TODO: assert that the looked-up values are actually instances of ConstantLogic
		const Logic & lhs = static_cast<ConstantLogic*>(lookup(I->getLhs()))->getValue();
		const Logic & rhs = static_cast<ConstantLogic*>(lookup(I->getRhs()))->getValue();
		Logic result = evalBinary(I->getOpcode(), lhs, rhs);
		tfm::printf("binary %d '%s' '%s' = %s\n", I->getOpcode(), lhs.toString(), rhs.toString(), result.toString());
		valueMap[I] = ConstantLogic::get(type, result);
	} else {
		llhd_abort_msg("invalid type for BinaryInst");
	}
}

BasicBlock * InstExecutor::execBranch(BranchInst * I) {
	if (!I->getCondition())
		return I->getIfTrue();
	unsigned cond = static_cast<ConstantInteger*>(lookup(I->getCondition()))->getValue();
	return cond ? I->getIfTrue() : I->getIfFalse();
}

} // namespace llhd
