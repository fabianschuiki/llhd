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
	virtual bool equalTo(Type * type) const override;
	virtual bool isLogic(unsigned width) const override;
protected:
	LogicType(Context & C, unsigned width);
	unsigned width;
};


class IntegerType : public Type {
public:
	static IntegerType * get(Context & C, unsigned width);
	unsigned getWidth() const { return width; }
	virtual bool equalTo(Type * type) const override;
	virtual bool isInteger(unsigned width) const override;
protected:
	IntegerType(Context & C, unsigned width);
	unsigned width;
};

} // namespace llhd
