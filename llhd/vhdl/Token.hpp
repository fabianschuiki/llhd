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
	kTokenInvalid = 0x000,
	kTokenEOF,
	kTokenComment,
	kTokenWhitespace,
	kTokenNumber,

	// Identifiers
	kTokenIdentifier = 0x100,
	kTokenBasicIdentifier,
	kTokenExtendedIdentifier,

	// Keywords, which are special identifiers
	kTokenKeywordIdentifier,
	kKeywordAbs,
	kKeywordAccess,
	kKeywordAfter,
	kKeywordAlias,
	kKeywordAll,
	kKeywordAnd,
	kKeywordArchitecture,
	kKeywordArray,
	kKeywordAssert,
	kKeywordAttribute,
	kKeywordBegin,
	kKeywordBlock,
	kKeywordBody,
	kKeywordBuffer,
	kKeywordBus,
	kKeywordCase,
	kKeywordComponent,
	kKeywordConfiguration,
	kKeywordConstant,
	kKeywordLabel,
	kKeywordDisconnect,
	kKeywordDownto,
	kKeywordMap,
	kKeywordElse,
	kKeywordElsif,
	kKeywordEnd,
	kKeywordEntity,
	kKeywordExit,
	kKeywordFile,
	kKeywordFor,
	kKeywordFunction,
	kKeywordGenerate,
	kKeywordGeneric,
	kKeywordGroup,
	kKeywordGuarded,
	kKeywordIf,
	kKeywordImpure,
	kKeywordIn,
	kKeywordInertial,
	kKeywordInout,
	kKeywordIs,
	kKeywordLibrary,
	kKeywordLinkage,
	kKeywordLiteral,
	kKeywordLoop,
	kKeywordMod,
	kKeywordNand,
	kKeywordNew,
	kKeywordNext,
	kKeywordNor,
	kKeywordNot,
	kKeywordNull,
	kKeywordOf,
	kKeywordOn,
	kKeywordOpen,
	kKeywordOr,
	kKeywordOthers,
	kKeywordOut,
	kKeywordPackage,
	kKeywordPort,
	kKeywordPostponed,
	kKeywordProcedural,
	kKeywordProcedure,
	kKeywordProcess,
	kKeywordProtected,
	kKeywordPure,
	kKeywordRange,
	kKeywordRecord,
	kKeywordReference,
	kKeywordRegister,
	kKeywordReject,
	kKeywordRem,
	kKeywordReport,
	kKeywordReturn,
	kKeywordRol,
	kKeywordRor,
	kKeywordSelect,
	kKeywordSeverity,
	kKeywordShared,
	kKeywordSignal,
	kKeywordSla,
	kKeywordSll,
	kKeywordSra,
	kKeywordSrl,
	kKeywordSubtype,
	kKeywordThen,
	kKeywordTo,
	kKeywordTransport,
	kKeywordType,
	kKeywordUnaffected,
	kKeywordUnits,
	kKeywordUntil,
	kKeywordUse,
	kKeywordVariable,
	kKeywordWait,
	kKeywordWhen,
	kKeywordWhile,
	kKeywordWith,
	kKeywordXnor,
	kKeywordXor,

	// Symbols
	kTokenSymbol = 0x200,
	kTokenAmpersand,
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
	kTokenGreater,

	// Literals
	kTokenLiteral = 0x300,
	kTokenAbstractLiteral,
	kTokenCharacterLiteral,
	kTokenStringLiteral,
	kTokenBitStringLiteral,

	kTokenMask = 0x300
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
