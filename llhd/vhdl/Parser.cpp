/* Copyright (c) 2014 Fabian Schuiki */
#include "Parser.hpp"
#include <iostream>
#include <string>
#include <vector>
using namespace llhd::vhdl;

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

static std::ostream& operator<< (std::ostream& o, const TokenRange& r) {
	o << r.start.line << "." << r.start.column << "-" << r.end.line << "." << r.end.column;
	return o;
}

enum TokenType {
	kTokenComment,
	kTokenWhitespace,
	kTokenIdentifier,
	kTokenSymbol,
	kTokenEOF
};

inline const char* tokenTypeToString(TokenType t) {
	switch (t) {
		case kTokenComment: return "comment";
		case kTokenWhitespace: return "whitespace";
		case kTokenIdentifier: return "identifier";
		case kTokenSymbol: return "symbol";
		case kTokenEOF: return "EOF";
		default: return "unknown";
	}
}

struct Token
{
	TokenType type;
	std::string value;
	TokenRange range;
};

std::ostream& operator<< (std::ostream& o, const Token& tkn) {
	o << tokenTypeToString(tkn.type) << " '";
	for (std::string::const_iterator i = tkn.value.begin(); i != tkn.value.end(); i++) {
		if (*i == '\n') {
			o << "\\n";
		} else {
			o << *i;
		}
	}
	o << "' " << tkn.range;
	return o;
}

static int getUnichar(std::istream& stream, unsigned& width)
{
	// Read the first byte. If it is below 128, the byte is returned as it is.
	int c = stream.get();
	if (c < 0x80)
		return c;

	// Otherwise inspect the leading ones of the byte which indicate the length
	// of the encoded code point. Two implementations are provided, one relying
	// on special CPU instructions.
#if 1
	width = __builtin_clz(~c);
	if (width < 2 || width > 6)
		return -1;
	int mask = (0x80 >> width) - 1;
	int uc = c & mask;
#else
	int uc = 0;
	if (c & 0xe0 == 0xc0) {
		width = 2;
		uc = c & 0x1f;
	} else if (c & 0xf0 == 0xe0) {
		width = 3;
		uc = c & 0x0f;
	} else if (c & 0xf8 == 0xf0) {
		width = 4;
		uc = c & 0x07;
	} else if (c & 0xfc == 0xf8) {
		width = 5;
		uc = c & 0x03;
	} else if (c & 0xfe == 0xfc) {
		width = 6;
		uc = c & 0x01;
	} else {
		return -1;
	}
#endif

	for (int i = 1; i < width; i++) {
		c = stream.get();
		if ((c & 0xc0) != 0x80)
			return -1;
		uc = uc << 6 | (c & 0x3f);
	}

	return uc;
}

struct Lexer
{
	std::istream& input;
	std::string buffer;
	int cursor;
	int width;
	TokenPosition start;
	TokenPosition pos;

	Lexer(std::istream& input): input(input), cursor(0) {}

	void emit(TokenType type) {
		Token tkn;
		tkn.type = type;
		tkn.value = buffer.substr(0, cursor);
		tkn.range = TokenRange(start, pos);
		// std::cout << "emitting " << tkn << '\n';
		start = pos;
		buffer.erase(buffer.begin(), buffer.begin() + cursor);
		cursor = 0;
	}

	void next(int n = 1) {
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

	bool accept(char c) {
		if (!ensure(1))
			return false;
		return (buffer[cursor] == c);
	}

	bool accept(const std::string& s) {
		if (!ensure(s.length()))
			return false;
		return std::equal(s.begin(), s.end(), buffer.begin() + cursor);
	}

	bool acceptOneOf(const std::string& s, int stride = 1) {
		if (!ensure(stride))
			return false;
		for (std::string::const_iterator i = s.begin(); i != s.end(); i += stride) {
			if (std::equal(i, i+stride, buffer.begin() + cursor))
				return true;
		}
		return false;
	}

	bool consume(char c) {
		if (accept(c)) {
			next();
			return true;
		} else {
			return false;
		}
	}

	bool consume(const std::string& s) {
		if (accept(s)) {
			next(s.length());
			return true;
		} else {
			return false;
		}
	}

	bool consumeOneOf(const std::string& s, int stride = 1) {
		if (acceptOneOf(s, stride)) {
			next(stride);
			return true;
		} else {
			return false;
		}
	}

	bool eof() { return cursor == buffer.length() && !input.good(); }

	// int next() {
	// 	if (cursor == buffer.length()) {
	// 		int c = getUnicode(input, width);
	// 		if (c < 0)
	// 			return -1;
	// 		buffer += c;
	// 		cursor += width;
	// 	}
	// 	return buffer[cursor-1];
	// }

	// int peek() {
	// 	int c = next();
	// 	backup();
	// 	return c;
	// }

	// void backup() {
	// 	cursor--;
	// }

private:
	bool ensure(int ahead) {
		int n = cursor + ahead - buffer.length();
		while (n > 0) {
			int c = input.get();
			if (c < 0)
				return false;
			buffer += c;
		}
		return true;
	}
};

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
}

// void Parser::parse(std::istream& input)
// {
// 	// In a first pass, tokenize the input stream.
// 	enum Context {
// 		kInvalid = 0,
// 		kIdentifier,
// 		kComment,
// 		kWhitespace
// 	};
// 	Context context = kInvalid;

// 	std::vector<char> buffer;
// 	TokenPosition l0;
// 	TokenPosition l1;

// 	while (input.good()) {
// 		int pc = input.get();
// 		int nc = input.peek();

// 		// Determine the context of the current character.
// 		Context nextContext = context;
// 		switch (context) {
// 			case kIdentifier:
// 			case kWhitespace:
// 			case kInvalid: {
// 				if (pc == '-' && nc == '-')
// 					nextContext = kComment;
// 				else if (pc == ' ' || pc == '\n' || pc == '\r' || pc == '\t')
// 					nextContext = kWhitespace;
// 				else
// 					nextContext = kIdentifier;
// 			} break;
// 			case kComment: {
// 				if (pc == '\n')
// 					nextContext = kWhitespace;
// 			}
// 		}

// 		if (context != nextContext) {
// 			if (!buffer.empty()) {
// 				buffer.push_back(0);
// 				Token tkn;
// 				tkn.type = kTokenComment;
// 				tkn.value = &buffer[0];
// 				tkn.range.s = l0;
// 				tkn.range.e = l1;
// 				std::cout << "found token '" << tkn.value << "', range " << tkn.range.s.l << "." << tkn.range.s.c << "-" << tkn.range.e.l << "." << tkn.range.e.c << '\n';
// 			}
// 			l0 = l1;
// 			context = nextContext;
// 			buffer.clear();
// 		}

// 		buffer.push_back(pc);
// 		if (pc == '\n') {
// 			l1.c = 0;
// 			l1.l++;
// 		} else {
// 			l1.c++;
// 		}
// 	}
// }
