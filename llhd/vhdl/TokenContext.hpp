/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/vhdl/Token.hpp"
#include <vector>

namespace llhd {
namespace vhdl {

/// A container for an array of related Tokens. Usually a parser will process
/// an input file an generate tokens, which are allocated and captured by a
/// TokenContext. Multiple files may be tokenized into one TokenContext if that
/// makes sense, e.g. when a file includes other files.
class TokenContext {
	/// Allocator that provides memory for the tokens.
	PoolAllocator<> alloc;
	/// Sequence of tokens, stored as pointers into memory provided by the
	/// allocator.
	std::vector<Token*> tokens;

public:
	/// Allocates \a size bytes aligned to \a alignment. The memory is provided
	/// by the allocator, and is therefore being garbage collected as soon as
	/// the TokenContext is destroyed.
	void* allocate(size_t size, unsigned alignment = 0);

	/// Allocates memory for a new token and constructs it with the given
	/// arguments. The memory is provided by the allocator, thus the token is
	/// garbage collected as soon as the TokenContext is destroyed.
	template <typename... Args> Token* allocate(Args&&... args) {
		// Token* tkn = alloc.one<Token>();
		// new (tkn) Token(&args...);
		// tokens.push_back(tkn);
		// return tkn;
		return 0;
	}
};

} // namespace vhdl
} // namespace llhd
