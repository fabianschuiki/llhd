/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/unicode/unichar.hpp"
#include <string>
#include <vector>

namespace llhd {

/// An entry for one named buffer in the SourceManager. Instances of this class
/// are generated internally inside the SourceManager to maintain a list of
/// buffers and their metadata. You should never need to interact with this
/// class from outside SourceManager.
class SourceManagerEntry {
	mutable std::vector<unsigned> lineOffsetCache;
	inline void ensureLineOffsetCache() const;

public:
	/// The unique identifier of this entry.
	unsigned id;
	/// Position in the SourceLocation space where this entry starts.
	unsigned offset;
	/// Number of characters in this entry.
	unsigned size;
	/// Position in the SourceLocation space where this entry ends.
	unsigned end;
	/// The buffer's name. Usually the name of the file where the buffer
	/// contents were loaded from.
	std::string name;
	/// Buffer containing the entry's source code. The entry does not take
	/// ownership of the buffer. The SourceManager provides an allocator for
	/// memory that is garbage collected when the manager is destroyed.
	const utf8char* buffer;

	/// Creates a new empty entry.
	SourceManagerEntry(
		unsigned id,
		unsigned offset,
		unsigned size,
		unsigned end):
		id(id),
		offset(offset),
		size(size),
		end(end),
		buffer(nullptr) {}

	unsigned getLineNumberAtOffset(unsigned offset) const;
	unsigned getColumnNumberAtOffset(unsigned offset) const;
};

} // namespace llhd
