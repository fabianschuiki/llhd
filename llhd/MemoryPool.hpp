/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"
#include "llhd/types.hpp"
#include "llhd/MallocAllocator.hpp"
#include <vector>

namespace llhd {

/// Allocates memory that is freed when the pool is destructed.
///
/// Implements a very basic form of garbage collection. When a piece of memory
/// is requested by calling allocate, the pool returns a memory location in the
/// current slab, or allocates a new slab of memory. The size of these slabs
/// increase with every slab created, thus reducing the number of allocations
/// requested from the allocator. For excessively large pieces of memory a slab
/// of custom size is allocated and returned.
///
/// The implementation borrows heavily from llvm::BumpPtrAllocatorImpl.
template <typename AllocatorType = MallocAllocator,
          size_t slabSize = 4096,
          size_t thresholdSize = slabSize>
class MemoryPool
{
	/// Used internally to allocate slabs.
	AllocatorType allocator;
	/// Points to the next free byte in the current slab.
	char* cur;
	/// Points to the end of the current slab.
	char* end;

	/// The slabs allocated so far.
	std::vector<void*> slabs;
	/// The custom-sized slabs allocated so far.
	std::vector<std::pair<void*,size_t> > customSizedSlabs;

	/// Total memory allocated from the underlying allocator.
	size_t totalSlabSize;
	/// Total memory allocated through the allocate function.
	size_t totalAllocatedSize;

	// Make the copy-constructor private to prevent pools being copied, which
	// would make no sense.
	MemoryPool(const MemoryPool&) {}

public:
	/// Creates an empty memory pool.
	MemoryPool():
		cur(NULL),
		end(NULL),
		totalSlabSize(0),
		totalAllocatedSize(0) {}

	/// Moves the allocated memory from \p old to a new pool. The old pool is
	/// is cleared in the process, thus effectively moving the responsibility
	/// to clean up the allocated memory to the new pool.
	MemoryPool(MemoryPool&& old):
		allocator(std::move(old.allocator)),
		cur(old.cur),
		end(old.end),
		slabs(std::move(old.slabs)),
		customSizedSlabs(std::move(old.customSizedSlabs)),
		totalSlabSize(old.totalSlabSize),
		totalAllocatedSize(old.totalAllocatedSize) {

		old.clear();
	}

	/// Also deallocates all memory that was allocated through this pool.
	~MemoryPool() {
		deallocateSlabs();
		deallocateCustomSizedSlabs();
	}

	/// Moves all allocated memory from the \p old pool to this. The old pool
	/// is cleared in the process.
	MemoryPool& operator= (MemoryPool&& old) {
		deallocateSlabs();
		deallocateCustomSizedSlabs();

		cur = old.cur;
		end = old.end;
		totalSlabSize = old.totalSlabSize;
		totalAllocatedSize = old.totalAllocatedSize;
		slabs = std::move(old.slabs);
		customSizedSlabs = std::move(old.customSizedSlabs);
		allocator = std::move(old.allocator);

		old.clear();

		return *this;
	}

	/// Deallocates all memory and resets the pool to its initial state.
	void reset() {
		deallocateSlabs();
		deallocateCustomSizedSlabs();
		clear();
	}

	/// Allocates memory of size \a size, aligned to \a alignment bytes. The
	/// memory either comes from the current slab; or, if the size exceeds the
	/// \c thresholdSize, from a custom-sized slab.
	void* allocate(size_t size, unsigned alignment = 0) {
		// Start a new slab if there is none, and keep track of the memory
		// allocated through this pool.
		if (!cur)
			startNewSlab();
		totalAllocatedSize += size;

		// Align the cur pointer according to the caller's request. An
		// alignment of 0 is interpreted as alignment to 1 byte.
		if (alignment == 0)
			alignment = 1;
		char* ptr = alignPtr(cur, alignment);

		// Take the memory from the current slab, if it is large enough.
		if (ptr + size <= end) {
			cur = ptr + size;
			return ptr;
		}

		// If size exceeds what we can fit into a slab, allocate a custom sized
		// slab. Note that we assume the worst case alignment situation where
		// the pointer is shifted (alignment-1) bytes, thus requiring equally
		// more memory to be allocated to accomodate the shift.
		size_t paddedSize = size + alignment - 1;
		if (paddedSize > thresholdSize) {
			void* slab = allocator.allocate(paddedSize);
			customSizedSlabs.push_back(std::make_pair(slab, paddedSize));

			ptr = alignPtr((char*)slab, alignment);
			assert(ptr + size < (char*)slab + paddedSize);
			return ptr;
		}

		// Otherwise start a new slab and try again.
		startNewSlab();
		ptr = alignPtr(cur, alignment);
		cur = ptr + size;
		assert(cur <= end && "Unable to allocate memory!");
		return ptr;
	}

	/// Allocates memory for \a num objects of type \c T. The allocated objects
	/// are not constructed, hence the responsibility to do so lies with the
	/// caller.
	template <typename T> T* allocate(unsigned num = 1) {
		return (T*)allocate(sizeof(T) * num, alignOf<T>::alignment);
	}

private:
	/// Clears this pool but does not deallocate any memory. The resulting pool
	/// is empty, as if it was just allocated with the default allocator.
	void clear() {
		cur = NULL;
		end = NULL;
		totalSlabSize = 0;
		totalAllocatedSize = 0;
		slabs.clear();
		customSizedSlabs.clear();
	}

	/// Allocates a new slab and sets the \c cur pointer to its start.
	void startNewSlab() {
		size_t size = computeSlabSize(slabs.size());
		totalSlabSize += size;
		void* slab = allocator.allocate(size);
		slabs.push_back(slab);
		cur = (char*)slab;
		end = cur + size;
	}

	/// Deallocates all slabs.
	void deallocateSlabs() {
		for (int i = 0; i < slabs.size(); i++) {
			allocator.deallocate(slabs[i], computeSlabSize(i));
		}
	}

	/// Deallocates all custom-sized slabs.
	void deallocateCustomSizedSlabs() {
		for (auto& i : customSizedSlabs) {
			allocator.deallocate(i.first, i.second);
		}
	}

	/// Computes the size of slab \p slabIndex. The returned size grows
	/// exponentially with the given index to reduce the frequency of
	/// allocations.
	size_t computeSlabSize(unsigned slabIndex) {
		const size_t saturation = 16; // prevents sizes higher than slabSize*2^saturation.
		const size_t period = 32;     // size doubles every period slabs
		return slabSize * ((size_t)1 << std::min<size_t>(saturation, slabIndex / period));
	}
};

} // namespace llhd
