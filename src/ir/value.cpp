/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/type.hpp"
#include "llhd/ir/value.hpp"
#include "llhd/ir/constant.hpp"

namespace llhd {

Value::Value(Type * type):
	type(type) {
	llhd_assert_msg(type, "Value must have a non-null type");
}

Context & Value::getContext() const {
	llhd_assert_msg(type, "Value must have a non-null type");
	return type->getContext();
}

Value * Value::getConst(Type * type, const std::string & str) {
	llhd_assert(type);
	return Constant::getValue(type,str);
}

Value * Value::getConstNull(Type * type) {
	llhd_assert(type);
	return Constant::getNullValue(type);
}

} // namespace llhd
