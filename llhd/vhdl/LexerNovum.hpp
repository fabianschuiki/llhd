/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class SourceBuffer;

namespace vhdl {

class TokenContext;

class LexerNovum {
	TokenContext& ctx;
public:
	LexerNovum(TokenContext& ctx): ctx(ctx) {}
	void lex(const SourceBuffer* src, SourceLocation loc);
};

} // namespace vhdl
} // namespace llhd
