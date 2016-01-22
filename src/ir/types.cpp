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
}

} // namespace llhd
