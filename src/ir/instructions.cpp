/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/instructions.hpp"
#include "llhd/ir/type.hpp"

namespace llhd {

DriveInst::DriveInst(Value * target, Value * value, BasicBlock * parent):
	Instruction(Type::getVoidType(target->getContext()), parent),
	target(target),
	value(value) {
	llhd_assert_msg(equal(target->getType(), value->getType()), "target and value must be of the same type");
}

BranchInst::BranchInst(Value * ifTrue, Value * ifFalse, Value * cond, BasicBlock * parent):
	Instruction(Type::getVoidType(ifTrue->getContext()), parent),
	ifTrue(ifTrue),
	ifFalse(ifFalse),
	condition(cond) {
}

SwitchInst::SwitchInst(Value * value, Value * otherwise, BasicBlock * parent):
	Instruction(Type::getVoidType(value->getContext()), parent),
	value(value),
	otherwise(otherwise) {
}

void SwitchInst::addDestination(Value * val, Value * dst) {
	llhd_assert(val);
	llhd_assert(dst);
	destinations.push_back(Destination(val,dst));
}

AddInst::AddInst(Value * lhs, Value * rhs, BasicBlock * parent):
	Instruction(lhs->getType(), parent),
	lhs(lhs),
	rhs(rhs) {
	llhd_assert_msg(equal(lhs->getType(), rhs->getType()), "lhs and rhs of add must be of same type");
}

SubInst::SubInst(Value * lhs, Value * rhs, BasicBlock * parent):
	Instruction(lhs->getType(), parent),
	lhs(lhs),
	rhs(rhs) {
	llhd_assert_msg(equal(lhs->getType(), rhs->getType()), "lhs and rhs of sub must be of same type");
}

static Type * getExtractValueType(Type * type, unsigned length) {
	switch (type->getTypeId()) {
		case Type::LogicTypeId:
			return Type::getLogicType(type->getContext(), length);
		default:
			llhd_abort_msg("extract value not supported for type");
	}
}

ExtractValueInst::ExtractValueInst(Value * target, Value * index, unsigned length, BasicBlock * parent):
	Instruction(getExtractValueType(target->getType(), length), parent),
	target(target),
	index(index),
	length(length) {
}

InsertValueInst::InsertValueInst(Value * target, Value * value, Value * index, unsigned length, BasicBlock * parent):
	Instruction(target->getType(), parent),
	target(target),
	value(value),
	index(index),
	length(length) {
}

CompareInst::CompareInst(Op op, Value * lhs, Value * rhs, BasicBlock * parent):
	Instruction(Type::getLogicType(lhs->getContext(),1), parent),
	op(op),
	lhs(lhs),
	rhs(rhs) {
	llhd_assert(lhs);
	llhd_assert(rhs);
}

} // namespace llhd
