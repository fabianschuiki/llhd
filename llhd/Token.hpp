/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class Token {
public:
	SourceRange range;
	unsigned type;

	Token(const SourceRange r, unsigned t):
		range(r),
		type(t) {}
};

} // namespace llhd
