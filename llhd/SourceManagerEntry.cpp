/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceManager.hpp"
#include <algorithm>
using llhd::SourceManagerEntry;

/// Writes the offset of all line breaks in the range [\a first,\a last) to the
/// output iterator \a out. Used to build the line offset cache for the
/// SourceManagerEntry class.
template<class InputIterator, class OutputIterator>
void computeLineOffsets(
	InputIterator first,
	InputIterator last,
	OutputIterator out) {

	*out++ = 0; // first line starts at offset 0

	InputIterator p = first;
	while (p != last) {
		if (*p == '\n' || *p == '\r') {
			InputIterator c = p;
			++p;
			if (p != last && (*p == '\n' || *p == '\r') && *p != *c)
				++p;
			*out++ = std::distance(first, p);
		} else {
			++p;
		}
	}
}

/// Builds the lineOffsetCache if necessary.
inline void SourceManagerEntry::ensureLineOffsetCache() const {

	// Nothing to do if the cache already exists.
	if (!lineOffsetCache.empty())
		return;

	// Compute the buffer offsets at which lines start, and accumulate the
	// numbers into an std::vector. We hint the vector at expecting around 256
	// lines, which speeds up insertion for the first 256 lines. A back_inserter
	// iterator is used to fill the vector.
	lineOffsetCache.clear();
	lineOffsetCache.reserve(256);
	computeLineOffsets(
		buffer,
		buffer+size,
		std::back_inserter(lineOffsetCache));
}

/// Returns the line number that contains the \a offset, starting at 1. Upon
/// first call, the line offset cache is built from the buffer contents, which
/// is a fairly expensive operation. Use only for diagnostics.
unsigned SourceManagerEntry::getLineNumberAtOffset(unsigned offset) const {

	// Make sure the line offsets are calculated.
	ensureLineOffsetCache();

	// upper_bound returns the first value in the line offset cache that is
	// larger than offset. E.g. for line offset {0, 10, 20}, it returns an
	// iterator to 10 for offsets 0-9, an iterator to 20 for offsets 10-19, and
	// the end iterator for offsets >= 20.
	auto i = std::upper_bound(
		lineOffsetCache.begin(),
		lineOffsetCache.end(),
		offset);

	// Since i is now an iterator to the line after the one that offset is on,
	// we may simply use the distance of i from the beginning of the cache (i.e.
	// the index into the cache) as the line number, starting at 1.
	return std::distance(lineOffsetCache.begin(), i);
}

/// Returns the column number of \a offset, starting at 1. I.e. the number of
/// characters from the beginning of the line \a offset is on.
unsigned SourceManagerEntry::getColumnNumberAtOffset(unsigned offset) const {
	assert(offset <= size);
	const utf8char* start = buffer+offset;
	const utf8char* p = start;

	// The start of the line should never be at offset directly. This happens
	// if the character at offset is a \n or \r:
	//     awesome stuff\n
	//     something\n
	//              ^ offset
	// In this case we don't want to return 0 as the column offset, but rather
	// the distance to the previous line break. Therefore we may always skip the
	// character directly at offset, as long as we don't leave the buffer.
	if (p != buffer)
		p--;

	// Search backwards until we hit the start of the buffer or a line break.
	// In case we hit a line break and not the start of the buffer, step the
	// pointer forward one step such that it points to the first character of
	// the line rather than the line break;
	while (p != buffer && *p != '\n' && *p != '\r')
		--p;
	if (p != buffer)
		++p; // step back over the \n or \r

	// Calculate the distance between the offset where we started searching and
	// the final position. Since we want the columns to start at 1 we add 1 to
	// the result.
	return start-p+1;
}
