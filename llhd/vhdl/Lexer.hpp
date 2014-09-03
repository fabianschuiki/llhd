/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"

namespace llhd {

class DiagnosticContext;
class TokenContext;

namespace vhdl {

/// Groups the bytes of a source file into individual tokens. See the TokenType
/// enum for an overview over the types of tokens that are generated. The tokens
/// are emitted into a TokenContext, and may later be passed to the Parser to be
/// turned into an abstract syntax tree.
class Lexer {
	TokenContext& ctx;
	DiagnosticContext& diag;
public:
	bool skipWhitespaces;
	bool skipComments;

	Lexer(TokenContext& ctx, DiagnosticContext& diag): ctx(ctx), diag(diag) {
		skipWhitespaces = true;
		skipComments = true;
	}
	void lex(const SourceBuffer& src, SourceLocation loc);
};

} // namespace vhdl
} // namespace llhd
