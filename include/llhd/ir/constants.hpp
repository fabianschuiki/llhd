/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/constant.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

// subclasses of Constant are listed here

class ConstantLogic : public Constant {
public:
	static ConstantLogic * getNull(LogicType * type);
	static ConstantLogic * get(LogicType * type, const std::string & str);
private:
	ConstantLogic(LogicType * type);
	std::vector<char> bits;
};


class ConstantInteger : public Constant {
public:
	static ConstantInteger * getNull(IntegerType * type);
	static ConstantInteger * get(IntegerType * type, const std::string & str);
	static ConstantInteger * get(IntegerType * type, std::intmax_t value);
private:
	ConstantInteger(IntegerType * type);
	std::intmax_t value;
};

} // namespace llhd
