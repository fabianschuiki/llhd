/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/config.hpp"
#include "llhd/types.hpp"
#include "llhd/utils/assert.hpp"

#include <tinyformat.h>

#include <map>
#include <set>
#include <string>
#include <vector>


template <class P>
class OwnedBy {
public:
	typedef P ParentType;
	OwnedBy(P * parent = nullptr) : parent(parent) {}
	void setParent(P * parent) { this->parent = parent; }
	P * getParent() { return parent; }
	const P * getParent() const { return parent; }
protected:
	P * parent;
};
