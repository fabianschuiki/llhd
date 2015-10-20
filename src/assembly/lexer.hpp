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
	TOKEN_MINUS,

	TOKEN_NAME_GLOBAL,
	TOKEN_NAME_LOCAL,
	TOKEN_NUMBER_LITERAL,
	TOKEN_STRING_LITERAL,
	TOKEN_TIME_LITERAL,
	TOKEN_INTEGER_LITERAL,
	TOKEN_REAL_LITERAL,
	TOKEN_TYPE,
	TOKEN_LABEL,

	TOKEN_KW_ABS,
	TOKEN_KW_ADD,
	TOKEN_KW_ALLOC,
	TOKEN_KW_AND,
	TOKEN_KW_BR,
	TOKEN_KW_CALL,
	TOKEN_KW_CAT,
	TOKEN_KW_CLEAR,
	TOKEN_KW_CMP,
	TOKEN_KW_COND,
	TOKEN_KW_DIV,
	TOKEN_KW_DRV,
	TOKEN_KW_EQ,
	TOKEN_KW_EXT,
	TOKEN_KW_FUNC,
	TOKEN_KW_INST,
	TOKEN_KW_LD,
	TOKEN_KW_LMAP,
	TOKEN_KW_MOD,
	TOKEN_KW_MUL,
	TOKEN_KW_NE,
	TOKEN_KW_NOT,
	TOKEN_KW_OR,
	TOKEN_KW_PROC,
	TOKEN_KW_REM,
	TOKEN_KW_RET,
	TOKEN_KW_SEL,
	TOKEN_KW_SGE,
	TOKEN_KW_SGT,
	TOKEN_KW_SIG,
	TOKEN_KW_SIGNED,
	TOKEN_KW_SLE,
	TOKEN_KW_SLT,
	TOKEN_KW_ST,
	TOKEN_KW_SUB,
	TOKEN_KW_TRUNC,
	TOKEN_KW_UGE,
	TOKEN_KW_UGT,
	TOKEN_KW_ULE,
	TOKEN_KW_ULT,
	TOKEN_KW_UNSIGNED,
	TOKEN_KW_WAIT,
	TOKEN_KW_XOR,
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
