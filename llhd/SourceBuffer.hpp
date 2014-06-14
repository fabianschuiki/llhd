/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"
#include "llhd/types.hpp"
#include "llhd/unicode/unichar.hpp"

namespace llhd {

/// A chunk of memory containing the contents of a source file. The memory is
/// not owned by the SourceBuffer. Users of the SourceBuffer class may expect
/// the memory to be null-terminated, which makes it very efficient to read the
/// contents since no end-of-file checks need to be performed.
class SourceBuffer {
	typedef unicode::utf8char utf8char;

	const utf8char* start;
	const utf8char* end;

public:
	/// Creates an empty buffer.
	SourceBuffer(): start(NULL), end(NULL) {}

	/// Creates a new buffer ranging from \a ptr to \a ptr + \a length.
	SourceBuffer(utf8char* ptr, size_t length):
		start(ptr),
		end(ptr+length) {

		assert(*(end-1) == 0 && "SourceBuffer not null-terminated!");
	}

	/// Creates a new buffer rangin from \a start to \a end.
	SourceBuffer(utf8char* start, utf8char* end):
		start(start),
		end(end) {

		assert(*(end-1) == 0 && "SourceBuffer not null-terminated!");
	}

	/// Returns a pointer to the first byte in the buffer.
	const utf8char* getStart() const { return start; }
	/// Returns a pointer to the location just after the last byte in the
	//// buffer.
	const utf8char* getEnd() const { return end; }
	/// Returns the size of the buffer.
	const size_t getSize() const { return end-start; }
	/// Returns true if the buffer is empty.
	bool isEmpty() const { return start == end; }
};

} // namespace llhd
