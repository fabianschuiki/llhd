/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"
#include "llhd/types.hpp"

namespace llhd {

/// A chunk of memory containing the contents of a source file. The memory is
/// not owned by the SourceBuffer. Users of the SourceBuffer class may expect
/// the memory to be null-terminated, which makes it very efficient to read the
/// contents since no end-of-file checks need to be performed.
class SourceBuffer {
	const char* start;
	const char* end;

public:
	/// Creates a new buffer ranging from \a ptr to \a ptr + \a length.
	SourceBuffer(char* ptr, size_t length):
		start(ptr),
		end(ptr+length) {

		assert(*(end-1) == 0 && "SourceBuffer not null-terminated!");
	}

	/// Creates a new buffer rangin from \a start to \a end.
	SourceBuffer(char* start, char* end):
		start(start),
		end(end) {

		assert(*(end-1) == 0 && "SourceBuffer not null-terminated!");
	}

	/// Returns a pointer to the first byte in the buffer.
	const char* getStart() const { return start; }
	/// Returns a pointer to the location just after the last byte in the
	//// buffer.
	const char* getEnd() const { return end; }
	/// Returns the size of the buffer.
	const size_t getSize() const { return end-start; }
};

} // namespace llhd
