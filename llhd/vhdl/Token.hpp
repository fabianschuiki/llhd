/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"
#include <ostream>

namespace llhd {
namespace vhdl {

class Token {
public:
	SourceRange range;
	unsigned type;

	Token(const SourceRange r, unsigned t):
		range(r),
		type(t) {}
};

} // namespace vhdl
} // namespace llhd
