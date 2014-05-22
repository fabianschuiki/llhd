/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <istream>
#include <set>
#include <string>

namespace llhd {

struct Token
{
	int type;
	std::string text;
};

class TokenizerState
{
	enum MatchResult {
		kDiscard, // state does not apply to the presented buffer
		kReduce,  // state can convert the presented buffer
		kNeedMore // state requires more characters to decide
	}
	virtual MatchResult match(const std::string& buffer, Token &token) = 0;
};

class Tokenizer
{
public:
	std::istream &is;
	std::string backlog;
	TokenizerState *state;

	typedef std::set<TokenizerState *> StateSet;
	StateSet states;

	Tokenizer(std::istream &is): is(is) {}
	~Tokenizer();

	bool read(Token &token);
};

} // namespace llhd