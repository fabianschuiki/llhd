/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class SourceBuffer;

namespace vhdl {

class LexerNovum {
	std::vector<Token*> tokens;
public:
	void lex(const SourceBuffer* src, SourceLocation loc);
};

} // namespace vhdl
} // namespace llhd
