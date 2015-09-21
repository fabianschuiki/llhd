/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/utils/range.hpp"
#include "llhd/location.hpp"

namespace llhd {

class DiagnosticContext;

/// \ingroup assembly
/// \needsdoc
enum AssemblyLexerToken {
	TOKEN_INVALID = 0,
	TOKEN_SOF,
	TOKEN_EOF,

	TOKEN_LBRACE,
	TOKEN_RBRACE,
	TOKEN_LPAREN,
	TOKEN_RPAREN,
	TOKEN_COMMA,
	TOKEN_EQUAL,

	TOKEN_NAME_GLOBAL,
	TOKEN_NAME_LOCAL,
	TOKEN_NUMBER_LITERAL,
	TOKEN_STRING_LITERAL,
	TOKEN_TYPE,

	TOKEN_KW_MOD,
	TOKEN_KW_PROC,
	TOKEN_KW_FUNC,
};

/// \ingroup assembly
/// \needsdoc
class AssemblyLexer {
	/// The input string being analyzed lexically.
	const Range<const char*> m_input;
	/// The source location of the input.
	const SourceLocation m_loc;
	/// The context where diagnostic messages will be sent.
	DiagnosticContext *m_dctx;
	/// The start of the current token.
	const char *m_base;
	/// The end of the current token.
	const char *m_ptr;
	/// The type of the current token.
	AssemblyLexerToken m_token = TOKEN_SOF;

public:
	AssemblyLexer(Range<const char*> input, SourceLocation loc, DiagnosticContext *dctx = nullptr);

	AssemblyLexer& next();
	operator bool() const;
	bool is_at_end() const;
	bool is_invalid() const;

	AssemblyLexerToken current_token() const;
	SourceRange current_range() const;
	Range<const char*> current_text() const;
	std::string current_string() const;

private:
	bool read_name();
};

} // namespace llhd
