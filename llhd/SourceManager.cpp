/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceManager.hpp"
// #include "llhd/SourceManager-internal.hpp"
using namespace llhd;


FileId SourceManager::addBuffer(
	const SourceBuffer& buffer,
	const std::string& name) {

	SourceManagerEntry& entry = makeEntry(buffer.getSize());
	entry.name = name;
	entry.buffer = buffer.getStart();

	return FileId(entry.id);
}

FileId SourceManager::addBufferCopy(
	const SourceBuffer& buffer,
	const std::string& name) {

	utf8char* copy = (utf8char*)alloc.allocate(buffer.getSize());
	std::copy(buffer.getStart(), buffer.getEnd(), copy);

	SourceManagerEntry& entry = makeEntry(buffer.getSize());
	entry.name = name;
	entry.buffer = copy;

	return FileId(entry.id);
}


SourceManagerEntry& SourceManager::makeEntry(unsigned size) {
	unsigned offset = srcTable.empty() ? 0 : srcTable.back().end;
	auto i = srcTable.emplace(
		srcTable.end(),
		srcTable.size() + 1, // id
		offset,              // offset
		size,                // size
		offset + size);      // end
	return *i;
}

/// Returns a SourceBuffer containing the contents of the file \a fid. If the
/// \a fid refers to a real file on disk, the file is loaded the first time
/// this function is called.
SourceBuffer SourceManager::getBuffer(FileId fid) {
	assert(fid.id > 0 && "FileId is invalid");
	assert(fid.id-1 < srcTable.size() && "FileId points outside source table!");
	auto e = srcTable[fid.id-1];
	return SourceBuffer(e.buffer, e.buffer + e.size);
}


/// Returns a location that points at the beginning of file \a fid. I.e. the
/// very first byte in the file.
SourceLocation SourceManager::getStartLocation(FileId fid) {
	assert(fid.id > 0 && "FileId is invalid");
	assert(fid.id-1 < srcTable.size() && "FileId points outside source table!");
	return SourceLocation(srcTable[fid.id-1].offset);
}

/// Returns a location that points at the end of file \a fid. I.e. the position
/// after the last byte of the file.
SourceLocation SourceManager::getEndLocation(FileId fid) {
	assert(fid.id > 0 && "FileId is invalid");
	assert(fid.id-1 < srcTable.size() && "FileId points outside source table!");
	return SourceLocation(srcTable[fid.id-1].end);
}

/// Returns the FileId which the location \a loc points at.
FileId SourceManager::getFileIdForLocation(SourceLocation loc) {
	assert(!srcTable.empty() && "source table is empty, nowhere SourceLocation could point!");

	// Make sure we're within the virtual location space.
	if (loc.id >= srcTable.back().end)
		return FileId(); // invalid id

	// Perform a binary search over the source table.
	unsigned lowIndex = 0;
	unsigned highIndex = srcTable.size();

	while (lowIndex != highIndex) {
		unsigned middleIndex = (lowIndex+highIndex)/2;
		const SourceManagerEntry& entry = srcTable[middleIndex];

		if (entry.offset > loc.id) {
			highIndex = middleIndex;
		} else if (entry.end <= loc.id) {
			lowIndex = middleIndex;
		} else {
			return FileId(middleIndex+1);
		}
	}

	return FileId(lowIndex+1);
}

/// Converts the location \a loc into a human-readable PresumedLocation. The
/// result contains filename, offset, line, and column information decoded from
/// the SourceLocation passed to the function.
PresumedLocation SourceManager::getPresumedLocation(SourceLocation loc) {

	// Look up the id of the file this location points into. Invalid values
	// are propagated back to the caller.
	FileId fid = getFileIdForLocation(loc);
	if (!fid.isValid())
		return PresumedLocation(); // invalid location

	// Look up the source table entry for the file and fill in the presumed
	// location details.
	const SourceManagerEntry& entry = srcTable[fid.id-1];
	unsigned offset = loc.id - entry.offset;

	PresumedLocation r;
	r.filename = entry.name;
	r.offset = offset;
	r.line = entry.getLineNumberAtOffset(offset);
	r.column = entry.getColumnNumberAtOffset(offset);
	return r;
}
