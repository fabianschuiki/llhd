/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/allocator/PoolAllocator.hpp"

namespace llhd {
namespace vhdl {
namespace ast {

/// A container for a VHDL abstract syntax tree.
class Context {
public:
	mutable PoolAllocator<> alloc;
};

} // namespace ast
} // namespace vhdl
} // namespace llhd
