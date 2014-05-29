/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <ostream>

namespace llhd {
namespace vhdl {

struct TokenPosition
{
	int line;
	int column;
	TokenPosition(): line(0), column(0) {}
};

struct TokenRange
{
	TokenPosition start;
	TokenPosition end;
	TokenRange() {}
	TokenRange(TokenPosition& s, TokenPosition& e): start(s), end(e) {}
};

enum TokenType {
	kTokenInvalid = 0x00,
	kTokenEOF = 0x01,
	kTokenComment = 0x20,
	kTokenWhitespace,
	kTokenIdentifier,
	kTokenNumber,

	// Symbols
	kTokenSymbol = 0x40,
	kTokenSymbolMask = 0xe0,

	kTokenAmpersand = 0x41,
	kTokenApostrophe,
	kTokenLParen,
	kTokenRParen,
	kTokenPlus,
	kTokenComma,
	kTokenMinus,
	kTokenPeriod,
	kTokenSemicolon,
	kTokenPipe,
	kTokenLBrack,
	kTokenRBrack,
	kTokenDoubleStar,
	kTokenStar,
	kTokenNotEqual,
	kTokenSlash,
	kTokenVarAssign,
	kTokenColon,
	kTokenLessEqual,
	kTokenBox,
	kTokenLess,
	kTokenArrow,
	kTokenEqual,
	kTokenGreaterEqual,
	kTokenGreater
};

struct Token
{
	TokenType type;
	std::string value;
	TokenRange range;
};

const char* tokenTypeToString(TokenType t);

std::ostream& operator<< (std::ostream& o, const TokenPosition& p);
std::ostream& operator<< (std::ostream& o, const TokenRange& r);
std::ostream& operator<< (std::ostream& o, const Token& tkn);

} // namespace vhdl
} // namespace llhd
