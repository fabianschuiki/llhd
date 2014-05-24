/* Copyright (c) 2014 Fabian Schuiki */
#include "Lexer.hpp"
using namespace llhd::vhdl;

Lexer::Lexer(std::istream& input): input(input), cursor(0) {}

void Lexer::emit(TokenType type) {
	Token tkn;
	tkn.type = type;
	tkn.value = buffer.substr(0, cursor);
	tkn.range = TokenRange(start, pos);
	tokens.push_back(tkn);
	start = pos;
	buffer.erase(buffer.begin(), buffer.begin() + cursor);
	cursor = 0;
}


void Lexer::next(int n) {
	for (int i = 0; i < n; i++) {
		if (buffer[cursor+i] == '\n') {
			pos.column = 0;
			pos.line++;
		} else {
			pos.column++;
		}
	}
	cursor += n;
}

bool Lexer::eof() {
	return cursor == (int)buffer.length() && !input.good();
}


bool Lexer::accept(char c) {
	if (!ensure(1))
		return false;
	return (buffer[cursor] == c);
}

bool Lexer::accept(const std::string& s) {
	if (!ensure(s.length()))
		return false;
	return std::equal(s.begin(), s.end(), buffer.begin() + cursor);
}

bool Lexer::acceptOneOf(const std::string& s, int stride) {
	if (!ensure(stride))
		return false;
	for (std::string::const_iterator i = s.begin(); i != s.end(); i += stride) {
		if (std::equal(i, i+stride, buffer.begin() + cursor))
			return true;
	}
	return false;
}


bool Lexer::consume(char c) {
	if (accept(c)) {
		next();
		return true;
	} else {
		return false;
	}
}

bool Lexer::consume(const std::string& s) {
	if (accept(s)) {
		next(s.length());
		return true;
	} else {
		return false;
	}
}

bool Lexer::consumeOneOf(const std::string& s, int stride) {
	if (acceptOneOf(s, stride)) {
		next(stride);
		return true;
	} else {
		return false;
	}
}


bool Lexer::ensure(int ahead) {
	int n = cursor + ahead - buffer.length();
	while (n > 0) {
		int c = input.get();
		if (c < 0)
			return false;
		buffer += c;
	}
	return true;
}
