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
			std::cout << "emitting " << type << " '" << std::string(bc,c) << "'\n";
			std::cout.flush();
		}
		bc = c;
		loc = nloc;
	};

	while (*c != 0) {
		if (*c <= 0x20 || (*c == 0xc2 && *(c+1) == 0xa0)) {
			c++;
			if (*c == 0xa0) c++;
			while ((*c <= 0x20 || (*c == 0xc2 && *(c+1) == 0xa0)) && *c != 0) {
				c++;
				if (*c == 0xa0) c++;
			}
			emit(skipWhitespaces ? kTokenInvalid : kTokenWhitespace);
		}
		else if (*c == '-' && *(c+1) == '-') {
			c++;
			while (*c != '\n' && *c != 0) c++;
			emit(skipComments ? kTokenInvalid : kTokenComment);
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
