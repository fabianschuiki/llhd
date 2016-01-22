/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

class Context;

// - types specify a static get() method to construct them within a context
// - Type provides a wrapper for the most commonly used get() method of each type

class Type {
public:
	enum TypeId {
		VoidTypeId,
		LogicTypeId,
		IntegerTypeId,
	};

	static Type * getVoidType(Context & C);
	static Type * getLogicType(Context & C, unsigned width);
	static Type * getIntegerType(Context & C, unsigned width);

	Context & getContext() const { return context; }
	TypeId getTypeId() const { return tid; }

	virtual bool equalTo(Type * other) const;
	friend bool equal(Type * A, Type * B);

protected:
	Type(const Type &) = delete;
	Type & operator=(const Type &) = delete;

	friend class Context;
	Type(Context &C, TypeId tid);
	virtual ~Type() {}

private:
	Context & context;
	TypeId tid;
};

} // namespace llhd
