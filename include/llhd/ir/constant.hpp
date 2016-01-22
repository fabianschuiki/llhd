/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/value.hpp"

namespace llhd {

class Constant : public Value {
public:
	static Constant * getValue(Type * type, const std::string & str);
	static Constant * getNullValue(Type * type);
protected:
	Constant(Type * type);
};

} // namespace llhd
