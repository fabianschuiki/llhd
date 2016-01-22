/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"
#include "llhd/ir/type.hpp"

namespace llhd {

// subclasses of Type are listed here

class LogicType : public Type {
public:
	static LogicType * get(Context & C, unsigned width);
	unsigned getWidth() const { return width; }
protected:
	LogicType(Context & C, unsigned width);
	unsigned width;
};

} // namespace llhd
