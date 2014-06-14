/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/unicode.hpp"
#include "llhd/vhdl/keywords.hpp"
#include "llhd/vhdl/KeywordMapper.hpp"
#include "llhd/vhdl/LexerNovum.hpp"
#include "llhd/vhdl/TokenContext.hpp"
#include "llhd/vhdl/Token.hpp"
#include <algorithm>
#include <cstring>
#include <iostream>
using namespace llhd::vhdl;

// inline bool isWhitespace(const char* c) {
// 	return *c == ' ' || *c == '\t' || *c == '\n'
// }

/// Implemented according to IEEE 1076-2000.
void LexerNovum::lex(const SourceBuffer& src, SourceLocation loc) {
	const utf8char* bc = src.getStart();
	const utf8char* c = bc;

	auto emit = [this,&bc,&c,&loc] (unsigned type) {
		if (bc == c)
			return;
		SourceLocation nloc = loc + (c-bc);
		if (type > 0) {
			// Token* tkn = this->ctx.alloc.one<Token>(
			// 	SourceBuffer(bc,c),
			// 	SourceRange(loc,nloc),
			// 	type);
			// this->ctx.addToken(tkn);
			std::cout << "emitting " << std::hex << type << std::dec << " " << std::string(bc,c) << "\n";
			std::cout.flush();
		}
		bc = c;
		loc = nloc;
	};

	KeywordMapper<> keywords; keywords
		("abs", kKeywordAbs)
		("access", kKeywordAccess)
		("after", kKeywordAfter)
		("alias", kKeywordAlias)
		("all", kKeywordAll)
		("and", kKeywordAnd)
		("architecture", kKeywordArchitecture)
		("array", kKeywordArray)
		("assert", kKeywordAssert)
		("attribute", kKeywordAttribute)
		("begin", kKeywordBegin)
		("block", kKeywordBlock)
		("body", kKeywordBody)
		("buffer", kKeywordBuffer)
		("bus", kKeywordBus)
		("case", kKeywordCase)
		("component", kKeywordComponent)
		("configuration", kKeywordConfiguration)
		("constant", kKeywordConstant)
		("label", kKeywordLabel)
		("disconnect", kKeywordDisconnect)
		("downto", kKeywordDownto)
		("map", kKeywordMap)
		("else", kKeywordElse)
		("elsif", kKeywordElsif)
		("end", kKeywordEnd)
		("entity", kKeywordEntity)
		("exit", kKeywordExit)
		("file", kKeywordFile)
		("for", kKeywordFor)
		("function", kKeywordFunction)
		("generate", kKeywordGenerate)
		("generic", kKeywordGeneric)
		("group", kKeywordGroup)
		("guarded", kKeywordGuarded)
		("if", kKeywordIf)
		("impure", kKeywordImpure)
		("in", kKeywordIn)
		("inertial", kKeywordInertial)
		("inout", kKeywordInout)
		("is", kKeywordIs)
		("library", kKeywordLibrary)
		("linkage", kKeywordLinkage)
		("literal", kKeywordLiteral)
		("loop", kKeywordLoop)
		("mod", kKeywordMod)
		("nand", kKeywordNand)
		("new", kKeywordNew)
		("next", kKeywordNext)
		("nor", kKeywordNor)
		("not", kKeywordNot)
		("null", kKeywordNull)
		("of", kKeywordOf)
		("on", kKeywordOn)
		("open", kKeywordOpen)
		("or", kKeywordOr)
		("others", kKeywordOthers)
		("out", kKeywordOut)
		("package", kKeywordPackage)
		("port", kKeywordPort)
		("postponed", kKeywordPostponed)
		("procedural", kKeywordProcedural)
		("procedure", kKeywordProcedure)
		("process", kKeywordProcess)
		("protected", kKeywordProtected)
		("pure", kKeywordPure)
		("range", kKeywordRange)
		("record", kKeywordRecord)
		("reference", kKeywordReference)
		("register", kKeywordRegister)
		("reject", kKeywordReject)
		("rem", kKeywordRem)
		("report", kKeywordReport)
		("return", kKeywordReturn)
		("rol", kKeywordRol)
		("ror", kKeywordRor)
		("select", kKeywordSelect)
		("severity", kKeywordSeverity)
		("shared", kKeywordShared)
		("signal", kKeywordSignal)
		("sla", kKeywordSla)
		("sll", kKeywordSll)
		("sra", kKeywordSra)
		("srl", kKeywordSrl)
		("subtype", kKeywordSubtype)
		("then", kKeywordThen)
		("to", kKeywordTo)
		("transport", kKeywordTransport)
		("type", kKeywordType)
		("unaffected", kKeywordUnaffected)
		("units", kKeywordUnits)
		("until", kKeywordUntil)
		("use", kKeywordUse)
		("variable", kKeywordVariable)
		("wait", kKeywordWait)
		("when", kKeywordWhen)
		("while", kKeywordWhile)
		("with", kKeywordWith)
		("xnor", kKeywordXnor)
		("xor", kKeywordXor)
		.compile();

	while (*c != 0) {
		// Characters 0x01..0x20 are treated as whitespace. This range covers
		// tab, line feed, carraige return, space, and many control characters.
		// This is more general than what the VHDL standard allows. The non-
		// breakable space 0xa0 (UTF-8 0xc2a0) is included as well.
		// 1076-2000 §13.1
		if (*c <= 0x20 || (*c == 0xc2 && *(c+1) == 0xa0)) {
			c++;
			if (*c == 0xa0) c++;
			while ((*c <= 0x20 || (*c == 0xc2 && *(c+1) == 0xa0)) && *c != 0) {
				c++;
				if (*c == 0xa0) c++;
			}
			emit(skipWhitespaces ? kTokenInvalid : kTokenWhitespace);
			// TODO: Add pedantic mode which emits warnings if the the
			// character read is not strictly a whitespace as defined by the
			// standard.
		}

		// Comments start with a double hyphen and proceed until the end of the
		// line.
		// 1076-2000 §13.8
		else if (*c == '-' && *(c+1) == '-') {
			c++;
			while (*c != '\n' && *c != 0) c++;
			emit(skipComments ? kTokenInvalid : kTokenComment);
		}

		// Delimiters in VHDL are the special characters "&'()*+,-./:;<=>|[]".
		// 1076-2000 §13.2
		else if (*c == '&') { c++; emit(kTokenAmpersand); }
		else if (*c == '(') { c++; emit(kTokenLParen); }
		else if (*c == ')') { c++; emit(kTokenRParen); }
		else if (*c == '+') { c++; emit(kTokenPlus); }
		else if (*c == ',') { c++; emit(kTokenComma); }
		else if (*c == '-') { c++; emit(kTokenMinus); }
		else if (*c == '.') { c++; emit(kTokenPeriod); }
		else if (*c == ';') { c++; emit(kTokenSemicolon); }
		else if (*c == '|' || *c == '!') { c++; emit(kTokenPipe); } // 1076-2000 §13.10
		else if (*c == '[') { c++; emit(kTokenLBrack); }
		else if (*c == ']') { c++; emit(kTokenRBrack); }

		// Compound delimiters in VHDL are "=> ** := /= >= <= <>".
		// 1076-2000 §13.2
		else if (*c == '*') { c++;
			if (*c == '*') { c++; emit(kTokenDoubleStar); }
			else emit(kTokenStar);
		}
		else if (*c == '/') { c++;
			if (*c == '=') { c++; emit(kTokenNotEqual); }
			else emit(kTokenSlash);
		}
		else if (*c == ':') { c++;
			if (*c == '=') { c++; emit(kTokenVarAssign); }
			else emit(kTokenColon);
		}
		else if (*c == '<') { c++;
			if (*c == '=') { c++; emit(kTokenLessEqual); }
			else if (*c == '>') { c++; emit(kTokenBox); }
			else emit(kTokenLess);
		}
		else if (*c == '=') { c++;
			if (*c == '>') { c++; emit(kTokenArrow); }
			else emit(kTokenEqual);
		}
		else if (*c == '>') { c++;
			if (*c == '=') { c++; emit(kTokenGreaterEqual); }
			else emit(kTokenGreater);
		}

		// Extended identifiers are encapsulated in backslashes \ and may
		// contain any character.
		// 1076-2000 §13.3.2
		else if (*c == '\\') {
			c++;
			while (!(*c == '\\' && *(c+1) != '\\') && *c != 0) {
				if (*c == '\\') c++; // allow \\ escaping
				c++;

				// TODO: Add pedantic mode which checks whether the characters
				// actually fall into VHDL's "191 graphic characters". By
				// default the parser is forgiving and allows virtually every
				// character here.
			}
			if (*c != '\\') {
				// TODO: Emit error indicating that the extended identifier is
				// not terminated and abort.
			}
			c++;
			emit(kTokenExtendedIdentifier);
		}

		// Abstract literals cover to notion of numbers in VHDL. An exact parse
		// is fairly involved, which is why during lexical analysis we simply
		// group characters that look like a number. It is the parser's duty
		// to perform a thorough interpretation of numbers.
		// 1076-2000 §13.4
		else if (*c >= '0' && *c <= '9') {
			c++;
			while ((*c >= '0' && *c <= '9') ||
			       (*c >= 'A' && *c <= 'Z') ||
			       (*c >= 'a' && *c <= 'z') ||
			       *c == '_' || *c == '#' || *c == ':' || *c == '.') c++;
			if (*(c-1) == 'e' || *(c-1) == 'E') {
				if (*c == '+' || *c == '-') c++;
				while (*c >= '0' && *c <= '9') c++;
			}
			emit(kTokenAbstractLiteral);
		}

		// Character literals encapsulate a single character in apostrophes.
		// Interestingly, they may contain the apostrophe itself, which results
		// in the interesting token '''.
		// 1076-2000 §13.5
		else if (*c == '\'') {
			c++; // consume the apostrophe
			c++; // consume the character
			while (*c & 0x80) c++; // consume longer unicode characters
			if (*c != '\'') {
				// TODO: emit error indicating that the character literal
				// contains more than one character. Fast-forward to the
				// next apostrophe to suggest a fix.
			}
			c++; // consume apostrophe
			// TODO: Add pedantic mode which checks whether the encapsulated
			// character actually is among the "191 graphic characters"
			// mentioned in §13.5.
			emit(kTokenCharacterLiteral);
		}

		// String literals encapsulate basically every character. Note that
		// §13.10 allows the perent sign % as a replacement for the double
		// quote character, as long as the literal starts and ends with the
		// same. This also influences escaping: In a %-delimited string, a
		// % must be written as %%; in a "-delimited string, a " must be
		// written as "".
		// 1076-2000 §13.6
		else if (*c == '"' || *c == '%') {
			char end = *c; // terminating character, " or %
			c++;
			while (!(*c == end && *(c+1) != end) && *c != 0) {
				if (*c == end) c++; // allow %% and "" escaping
				if (*c == '\\') {
					c++; // tolerate backspace escaping
					// TODO: Add pedantic mode which would emit an error here
					// indicating that backspace escaping is not part of the
					// standard.
				}
				c++;
			}
			if (*c != end) {
				// TODO: Emit error indicating that the string literal is not
				// terminated and abort.
			}
			c++; // consume terminating character
			// TODO: Add pedantic mode which checks whether the encapsulated
			// string actually contains only the allowed graphical characters.
			emit(kTokenStringLiteral);
		}

		// Bit string literals consist of a base specifier and a literal string
		// containing the values. The standard is fairly strict when it comes
		// to the things allowed inside the literal. We generously gather all
		// the characters that look like a bit string literal, assuming that
		// the later interpretation of the literal will generate errors where
		// appropriate. This allows the lexer to read over obvious errors.
		// 1076-2000 §13.7
		else if ((*c == 'b' || *c == 'B' ||
			      *c == 'o' || *c == 'O' ||
			      *c == 'x' || *c == 'X') &&
		         (*(c+1) == '"' || *(c+1) == '%')) {
			c++; // base
			char end = *c; // terminating character, " or %
			c++;
			while (!(*c == end && *(c+1) != end) && *c != 0) {
				if (*c == end) c++; // allow %% and "" escaping
				if (*c == '\\') {
					c++; // tolerate backspace escaping
					// TODO: Add pedantic mode which would emit an error here
					// indicating that backspace scaping is not part of the
					// standard.
				}
				c++;
			}
			if (*c != end) {
				// TODO: Emit error indicating that the bit string literal is
				// not terminated and abort.
			}
			c++; // consume terminating character.
			emit(kTokenBitStringLiteral);
		}

		// Basic identifiers are fairly limited in the standard, allowing only
		// a small set of characters. This lexer tries to be very forgivin when
		// it comes to identifiers, allowing virtually every character not
		// covered by some other rule to be treated as an identifier. Basically
		// 0-9, a-z, A-Z, _ and all higher unicode code points are considered
		// valid.
		// 1076-2000 §13.3.1
		else if ((*c >= 'A' && *c <= 'Z') ||
		         (*c >= 'a' && *c <= 'z') ||
		         ((*c & 0x80) && !(*c == 0xc2 && *(c+1) == 0xa0))) {

			while ((*c >= '0' && *c <= '9') ||
			       (*c >= 'A' && *c <= 'Z') ||
			       (*c >= 'a' && *c <= 'z') ||
			       ((*c & 0x80) && !(*c == 0xc2 && *(c+1) == 0xa0)) ||
			       *c == '_') c++;

			// TODO: Add pedantic mode which checks whether the characters used
			// in the identifier belong to the "191 graphic characters" defined
			// by the standard.

			if (*c == *bc) {
				// TODO: Emit error indicating that this is a garbage character
				// in the file.
				c++;
				emit(kTokenInvalid);
			} else {
				unsigned mapped = lookupKeyword(
					unicode::casefold_iterator<utf8char>(bc),
					unicode::casefold_iterator<utf8char>(c));
				emit(mapped > 0 ? mapped : kTokenBasicIdentifier);
			}
		}
	}
}
