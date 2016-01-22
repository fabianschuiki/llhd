/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/context.hpp"
#include "llhd/ir/value.hpp"

namespace llhd {

Context::Context():
	voidType(*this, Type::VoidTypeId) {
}

Context::~Context() {
	for (auto p : logicTypeMap)
		delete p.second;
	for (auto p : values)
		delete p;
}

} // namespace llhd
