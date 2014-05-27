/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"
#include "llhd/allocator/Allocator.hpp"
#include <cstdlib>

namespace llhd {

/// Allocates memory using malloc/free.
class MallocAllocator : public Allocator<MallocAllocator> {
public:
	/// Allocates \a size bytes aligned to \a alignment bytes. Beware that
	/// the actual amount of memory allocated is (size+alignment-1) to
	/// accomodate the worst alignment case. Always try to allocate larger
	/// chunks of aligned memory, or use a MemoryPool to individually allocate
	/// small aligned chunks.
	void* allocate(size_t size, unsigned alignment = 1) {
		// Perform a sanity check on the alignment.
		assert(alignment > 0 && alignment <= 128 && "zero or excessive alignment");

		// To accomodate the alignment, we have to assume the worst case
		// situation where the returned memory location needs to be shifted by
		// (alignment-1).
		size_t paddedSize = size + alignment - 1;
		void* ptr = malloc(paddedSize);

		// Return the aligned pointer.
		return (void*)alignPtr((char*)ptr, alignment);
	}

	/// Deallocates the memory at \a ptr.
	void deallocate(void* ptr) { free(ptr); }
};

} // namespace llhd
