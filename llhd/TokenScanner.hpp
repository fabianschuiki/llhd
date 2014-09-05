/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/Token.hpp"
#include "llhd/TokenBuffer.hpp"
#include <stdexcept>
#include <string>

namespace llhd {

class TokenScanner {
	TokenScanner* parent;
	Token** start;
	Token** current;
	Token** end;

public:
	TokenScanner(Token** start, Token** end, TokenScanner* parent = nullptr):
		parent(parent),
		start(start),
		current(start),
		end(end) {}

	Token** getStart() const { return start; }
	Token** getEnd() const { return end; }
	Token** getCurrent() const { return current; }

	bool isAtEnd() const { return current == end; }

	SourceRange getRange() const {
		return SourceRange((*start)->range.s, (*(end-1))->range.e);
	}

	SourceRange getRangeToHere() const {
		if (current == start)
			return SourceRange((*start)->range.s, (*start)->range.s+1);
		return SourceRange((*start)->range.s, (*(current-1))->range.e);
	}

	SourceRange getCurrentRange() const {
		if (current == end) {
			auto l = (*(end-1))->range.e;
			return SourceRange(l-1,l);
		}
		return (*current)->range;
	}

	bool accept(unsigned type, Token** into = nullptr) {
		if (current != end && (*current)->type == type) {
			if (into)
				*into = *current;
			advance();
			return true;
		}
		return false;
	}

	bool find(unsigned type, Token** into = nullptr) {
		bool found = (current != end && (*current)->type == type);
		if (found && into)
			*into = *current;
		advance();
		return found;
	}

	TokenScanner branch() {
		return TokenScanner(current, end, this);
	}

	TokenScanner slice(unsigned startOffset, unsigned currentOffset) {
		return TokenScanner(start+startOffset, current-currentOffset, this);
	}

	void commit() {
		if (parent)
			parent->current = current;
	}

private:
	void advance() {
		if (current == end) {
			throw std::runtime_error("read past the end of the input");
		}
		current++;
	}
};

} // namespace llhd
