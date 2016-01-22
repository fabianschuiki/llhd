/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/context.hpp"
#include "llhd/ir/type.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

Type::Type(Context & C, TypeId tid):
	context(C),
	tid(tid) {
}

Type * Type::getVoidType(Context & C) { return &C.voidType; }
Type * Type::getLogicType(Context & C, unsigned width) { return LogicType::get(C,width); }
Type * Type::getIntegerType(Context & C, unsigned width) { return IntegerType::get(C,width); }

bool Type::equalTo(Type * other) const {
	llhd_assert(other);
	return tid == other->tid;
}

bool equal(Type * A, Type * B) {
	llhd_assert(A);
	llhd_assert(B);
	return A->equalTo(B);
}

} // namespace llhd
