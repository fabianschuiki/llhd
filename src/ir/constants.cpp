/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/context.hpp"
#include "llhd/ir/constants.hpp"

namespace llhd {

ConstantLogic * ConstantLogic::getNullValue(LogicType * type) {
	llhd_assert(type);

	auto * C = new ConstantLogic(type);
	type->getContext().values.push_back(C);
	std::fill(C->bits.begin(), C->bits.end(), '0');

	return C;
}

ConstantLogic * ConstantLogic::get(LogicType * type, const std::string & str) {
	llhd_assert(type);
	llhd_assert(type->getWidth() == str.size());

	auto * C = new ConstantLogic(type);
	type->getContext().values.push_back(C);
	std::copy(str.begin(), str.end(), C->bits.begin());

	return C;
}

ConstantLogic::ConstantLogic(LogicType * type):
	Constant(type) {
	llhd_assert(type);
	bits.resize(type->getWidth());
}

} // namespace llhd
