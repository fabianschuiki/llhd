/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/MemoryPool.hpp"

namespace llhd {
namespace vhdl {
namespace ast {

/// A container for a VHDL abstract syntax tree.
class Context {
	mutable MemoryPool<> pool;

public:
	/// Allocates memory for objects associated with this Context. All memory
	/// allocated via this method is freed automatically when the Context is
	/// itself destroyed.
	void* allocate(size_t size) const {
		return pool.allocate(size);
	}
};

} // namespace ast
} // namespace vhdl
} // namespace llhd
