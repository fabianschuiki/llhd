/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"
#include "llhd/MallocAllocator.hpp"
#include <vector>

namespace llhd {

/// @brief Allocates memory that is freed when the pool is destructed.
///
/// Implements a very basic form of garbage collection. When a piece of memory
/// is requested by calling allocate, the pool returns a memory location in the
/// current slab, or allocates a new slab of memory. The size of these slabs
/// increase with every slab created, thus reducing the number of allocations
/// requested from the allocator. For excessively large pieces of memory a slab
/// of custom size is allocated and returned.
template <
	typename AllocatorType = MallocAllocator,
    size_t slabSize = 4096,
    size_t thresholdSize = slabSize>
class MemoryPool {

	/// @brief Used internally to allocate slabs.
	AllocatorType allocator;
	/// @brief Points to the next free byte in the current slab.
	char* cur;
	/// @brief Points to the end of the current slab.
	char* end;

	/// @brief The slabs allocated so far.
	std::vector<void*> slabs;
	/// @brief The custom-sized slabs allocated so far.
	std::vector<std::pair<void*,size_t> > customSizedSlabs;

public:

	void* allocate(size_t size, unsigned align) {
		return 0;
	}
};

} // namespace llhd
