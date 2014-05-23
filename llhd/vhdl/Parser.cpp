/* Copyright (c) 2014 Fabian Schuiki */
#include "Lexer.hpp"
#include "Parser.hpp"
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using namespace llhd::vhdl;

struct StateFn
{
	typedef StateFn (*FnType)(Lexer& l);
	FnType fn;

	StateFn(): fn(0) {}
	StateFn(FnType fn): fn(fn) {}
	StateFn operator()(Lexer& l) const { return fn(l); }
};

Parser::Parser()
{
}

Parser::~Parser()
{
}

inline static bool consumeWhitespace(Lexer& l) {
	return
		l.consume(' ')  ||
		l.consume('\t') ||
		l.consume('\n') ||
		l.consume('\r') ||
		l.consume("\u00A0");
}

inline static bool acceptWhitespace(Lexer& l) {
	return
		l.accept(' ')  ||
		l.accept('\t') ||
		l.accept('\n') ||
		l.accept('\r') ||
		l.accept("\u00A0");
}

inline static bool consumeSymbol(Lexer& l) {
	return
		l.consumeOneOf("?/=" "?<=" "?>=", 3) ||
		l.consumeOneOf("=>" "**" ":=" "/=" ">=" "<=" "<>" "??" "?=" "?<" "?>" "<<" ">>", 2) ||
		l.consumeOneOf("&'()*+,-./:;<=>`|[]?@");
}

inline static bool acceptSymbol(Lexer& l) {
	return
		l.acceptOneOf("?/=" "?<=" "?>=", 3) ||
		l.acceptOneOf("=>" "**" ":=" "/=" ">=" "<=" "<>" "??" "?=" "?<" "?>" "<<" ">>", 2) ||
		l.acceptOneOf("&'()*+,-./:;<=>`|[]?@");
}

static StateFn lexRoot(Lexer& l);
static StateFn lexComment(Lexer& l);
static StateFn lexWhitespace(Lexer& l);
static StateFn lexIdentifier(Lexer& l);

static StateFn lexComment(Lexer& l)
{
	if (l.eof() || l.accept('\n')) {
		l.emit(kTokenComment);
		return lexRoot;
	}
	l.next();
	return lexComment;
}

static StateFn lexWhitespace(Lexer& l)
{
	if (l.eof() || !consumeWhitespace(l)) {
		l.emit(kTokenWhitespace);
		return lexRoot;
	}
	return lexWhitespace;
}

static StateFn lexIdentifier(Lexer& l)
{
	if (l.eof() || acceptWhitespace(l) || acceptSymbol(l)) {
		l.emit(kTokenIdentifier);
		return lexRoot;
	}
	l.next();
	return lexIdentifier;
}

static StateFn lexRoot(Lexer& l)
{
	if (consumeWhitespace(l))
		return lexWhitespace;
	if (l.consume("--"))
		return lexComment;
	if (consumeSymbol(l)) {
		l.emit(kTokenSymbol);
		return lexRoot;
	}
	if (l.eof())
		return 0;
	else
		return lexIdentifier;

	throw std::runtime_error("garbage at end of file");
}

/** Parses the given input stream. */
void Parser::parse(std::istream& input)
{
	Lexer l(input);
	for (StateFn state = lexRoot; state.fn != 0;) {
		state = state(l);
	}
	std::cout << "parsed " << l.tokens.size() << " tokens\n";
}
