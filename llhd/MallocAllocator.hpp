/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <cstdlib>

namespace llhd {

class MallocAllocator {
public:
	void* allocate(size_t size) { return malloc(size); }
	void deallocate(void* ptr, size_t size) { free(ptr); }
};

} // namespace llhd
