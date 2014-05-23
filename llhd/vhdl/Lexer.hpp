/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "Token.hpp"
#include <istream>
#include <string>
#include <vector>

namespace llhd {
namespace vhdl {

struct Lexer
{
	std::istream& input;
	std::string buffer;
	int cursor;
	TokenPosition start;
	TokenPosition pos;
	std::vector<Token> tokens;

	Lexer(std::istream& input);

	void emit(TokenType type);

	void next(int n = 1);
	bool eof();

	bool accept(char c);
	bool accept(const std::string& s);
	bool acceptOneOf(const std::string& s, int stride = 1);

	bool consume(char c);
	bool consume(const std::string& s);
	bool consumeOneOf(const std::string& s, int stride = 1);

private:
	bool ensure(int ahead);
};

} // namespace vhdl
} // namespace llhd
