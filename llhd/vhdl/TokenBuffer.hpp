/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {
namespace vhdl {

class Token;

/// A chunk of memory containing pointers to Tokens. Users of this class may
/// assume that the buffer be null-terminated, i.e. the very last token is a
/// NULL-pointer. This allows for fairly efficient parsing.
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
		end(ptr+length) {

		assert(*(end-1) == NULL && "TokenBuffer not null-terminated!");
	}

	/// Creates a buffer ranging from \a start to \a end. The last token must
	/// be NULL.
	TokenBuffer(Token** start, Token** end):
		start(start),
		end(end) {

		assert(*(end-1) == NULL && "TokenBuffer not null-terminated!");
	}

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

} // namespace vhdl
} // namespace llhd
