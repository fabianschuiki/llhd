/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/context.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

LogicType * LogicType::get(Context & C, unsigned width) {
	auto & slot = C.logicTypeMap[width];
	if (!slot)
		slot = new LogicType(C,width);
	return slot;
}

LogicType::LogicType(Context & C, unsigned width):
	Type(C, Type::LogicTypeId),
	width(width) {
	llhd_assert_msg(width > 0, "logic type must be at least 1 bit");
}

bool LogicType::equalTo(Type * other) const {
	if (!Type::equalTo(other))
		return false;
	auto * o = static_cast<LogicType*>(other);
	return width == o->width;
}

bool LogicType::isLogic(unsigned width) const {
	return this->width == width;
}

IntegerType * IntegerType::get(Context & C, unsigned width) {
	auto & slot = C.intTypeMap[width];
	if (!slot)
		slot = new IntegerType(C,width);
	return slot;
}

IntegerType::IntegerType(Context & C, unsigned width):
	Type(C, Type::IntegerTypeId),
	width(width) {
	llhd_assert_msg(width > 0, "integer type must be at least 1 bit");
}

bool IntegerType::equalTo(Type * other) const {
	if (!Type::equalTo(other))
		return false;
	auto * o = static_cast<IntegerType*>(other);
	return width == o->width;
}

bool IntegerType::isInteger(unsigned width) const {
	return this->width == width;
}

} // namespace llhd
