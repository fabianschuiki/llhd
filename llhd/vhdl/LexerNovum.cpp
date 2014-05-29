/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/vhdl/LexerNovum.hpp"
#include "llhd/vhdl/TokenContext.hpp"
#include "llhd/vhdl/Token.hpp"
#include <cstring>
#include <iostream>
using namespace llhd::vhdl;

// inline bool isWhitespace(const char* c) {
// 	return *c == ' ' || *c == '\t' || *c == '\n'
// }

/// Implemented according to IEEE 1076-2000.
void LexerNovum::lex(const SourceBuffer& src, SourceLocation loc) {
	const char* bc = src.getStart();
	const char* c = bc;

	auto emit = [this,&bc,&c,&loc] (unsigned type) {
		if (bc == c)
			return;
		SourceLocation nloc = loc + (c-bc);
		if (type > 0) {
			// Map identifiers that describe keywords to a separate token type
			// which will help parsing. Write a string table matcher for this
			// which can take advantage of common prefixes among the strings.
			// Maybe a handcrafted function would suffice as well.
			if (type == kTokenIdentifier) {
				if (std::equal(bc, c, "entity"))
					type = 1000;
			}
			// Token* tkn = this->ctx.alloc.one<Token>(
			// 	SourceBuffer(bc,c),
			// 	SourceRange(loc,nloc),
			// 	type);
			// this->ctx.addToken(tkn);
			std::cout << "emitting " << std::hex << type << " '" << std::string(bc,c) << "'\n";
			std::cout.flush();
		}
		bc = c;
		loc = nloc;
	};

	while (*c != 0) {
		// Characters 0x01..0x20 are treated as whitespace. This range covers
		// tab, line feed, carraige return, space, and many control characters.
		// This is more general than what the VHDL standard allows. The non-
		// breakable space 0xa0 (UTF-8 0xc2a0) is included as well.
		// 1076-2000 §13.1
		if (*c <= 0x20 || (*c == (char)0xc2 && *(c+1) == (char)0xa0)) {
			c++;
			if (*c == (char)0xa0) c++;
			while ((*c <= 0x20 || (*c == (char)0xc2 && *(c+1) == (char)0xa0)) && *c != 0) {
				c++;
				if (*c == (char)0xa0) c++;
			}
			emit(skipWhitespaces ? kTokenInvalid : kTokenWhitespace);
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
		else if (*c == '\'') { c++; emit(kTokenApostrophe); }
		else if (*c == '(') { c++; emit(kTokenLParen); }
		else if (*c == ')') { c++; emit(kTokenRParen); }
		else if (*c == '+') { c++; emit(kTokenPlus); }
		else if (*c == ',') { c++; emit(kTokenComma); }
		else if (*c == '-') { c++; emit(kTokenMinus); }
		else if (*c == '.') { c++; emit(kTokenPeriod); }
		else if (*c == ';') { c++; emit(kTokenSemicolon); }
		else if (*c == '|') { c++; emit(kTokenPipe); }
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


		else if (*c >= '0' && *c <= '9') {
			c++;
			while (*c >= '0' && *c <= '9') c++;
			emit(kTokenNumber);
		}
		else if (*c < 0x41) {
			c++;
			emit(kTokenSymbol);
		}
		else {
			c++;
			while ((*c >= '0' && *c <= '9') || *c > 0x40) c++;
			emit(kTokenIdentifier);
		}
	}
}
