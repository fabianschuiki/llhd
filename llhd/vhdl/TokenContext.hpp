/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/MemoryPool.hpp"
#include <vector>

namespace llhd {
namespace vhdl {

class TokenContext {
	MemoryPool<> allocator;
	std::vector<Token*> tokens;

public:
	void* allocate(size_t size, unsigned alignment = 0);
	template <typename Args> Token* allocate(Args&&... args) {
		Token* tkn = allocator.allocate<Token>();
		new (tkn) Token(args);
		tokens.push_back(tkn);
		return tkn;
	}
};

} // namespace vhdl
} // namespace llhd
