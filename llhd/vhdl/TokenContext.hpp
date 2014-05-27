/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/vhdl/TokenBuffer.hpp"
#include <vector>

namespace llhd {
namespace vhdl {

/// A container for an array of related Tokens. Usually a parser will process
/// an input file an generate tokens, which are allocated and captured by a
/// TokenContext. Multiple files may be tokenized into one TokenContext if that
/// makes sense, e.g. when a file includes other files.
class TokenContext {
	/// Sequence of tokens, stored as pointers into memory provided by the
	/// allocator.
	std::vector<Token*> tokens;

public:
	/// Allocator that provides garbage collected memory for the tokens. May
	/// also be used for other things which ought to be deallocated when this
	/// TokenContext is destroyed.
	PoolAllocator<> alloc;

	/// Adds the Token \a tkn to this context. The whole of all calls to this
	/// function forms a sequence of Token, which may be accessed by calling
	/// the getBuffer() function.
	void addToken(Token* tkn) {
		tokens.push_back(tkn);
	}

	TokenBuffer getBuffer() {
		return TokenBuffer(&tokens[0], tokens.size());
	}
};

} // namespace vhdl
} // namespace llhd
