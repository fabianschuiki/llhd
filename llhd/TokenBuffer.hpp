/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {

class Token;

/// A chunk of memory containing pointers to Tokens.
class TokenBuffer {
	Token** start;
	Token** end;

public:
	/// Creates an empty buffer.
	TokenBuffer(): start(NULL), end(NULL) {}

	/// Creates a buffer ranging from \a ptr to \a ptr + \a length. The last
	/// token must be NULL.
	TokenBuffer(Token** ptr, size_t length):
		start(ptr),
		end(ptr+length) {}

	/// Creates a buffer ranging from \a start to \a end. The last token must
	/// be NULL.
	TokenBuffer(Token** start, Token** end):
		start(start),
		end(end) {}

	/// Returns a pointer to the first token in the buffer.
	Token** getStart() const { return start; }
	/// Returns a pointer to the position just after the last token in the
	/// buffer.
	Token** getEnd() const { return end; }
	/// Returns the number of tokens in the buffer.
	size_t getLength() const { return end-start; }
	/// Returns true if the buffer is empty.
	bool isEmpty() const { return start == end; }
};

} // namespace llhd
