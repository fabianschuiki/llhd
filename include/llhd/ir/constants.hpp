/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/constant.hpp"
#include "llhd/ir/types.hpp"

namespace llhd {

// subclasses of Constant are listed here

class ConstantLogic : public Constant {
public:
	static ConstantLogic * getNullValue(LogicType * type);
	static ConstantLogic * get(LogicType * type, const std::string & str);
	unsigned getBitWidth() const { return bits.size(); }
private:
	ConstantLogic(LogicType * type);
	std::vector<char> bits;
};

} // namespace llhd
