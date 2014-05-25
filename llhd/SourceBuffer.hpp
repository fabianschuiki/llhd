/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {

/// A chunk of memory containing the contents of a source file. The memory is
/// not owned by the SourceBuffer. Users of the SourceBuffer class may expect
/// the memory to be null-terminated, which makes it very efficient to read the
/// contents since no end-of-file checks need to be performed.
class SourceBuffer {
	const char* bufferStart;
	const char* bufferEnd;

public:
	SourceBuffer(char* ptr, size_t length):
		bufferStart(ptr),
		bufferEnd(ptr+length) {}

	const char* getBufferStart() const { return bufferStart; }
	const char* getBufferEnd() const { return bufferEnd; }
	const size_t getBufferSize() const { return bufferEnd-bufferStart; }
};

} // namespace llhd
