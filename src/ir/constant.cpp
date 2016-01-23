/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/constant.hpp"
#include "llhd/ir/constants.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

Constant * Constant::get(Type * type, const std::string & str) {
	llhd_assert(type);
	switch (type->getTypeId()) {
		case Type::LogicTypeId:
			return ConstantLogic::get(dynamic_cast<LogicType*>(type), str);
		default:
			llhd_abort_msg("cannot construct value from string for type");
			return nullptr;
	}
}

Constant * Constant::getNull(Type * type) {
	llhd_assert(type);
	switch (type->getTypeId()) {
		case Type::LogicTypeId:
			return ConstantLogic::getNull(dynamic_cast<LogicType*>(type));
		default:
			llhd_abort_msg("No corresponding null value for type");
			return nullptr;
	}
}

Constant::Constant(Type * type):
	Value(Value::ConstantId, type) {
}

} // namespace llhd
