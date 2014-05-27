/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/SourceManager-internal.hpp"
using namespace llhd;


SourceManager::~SourceManager() {
}

/// Puts the file \a fp under management and returns a FileId for it. This id
/// is a highly efficient, unique identifier for this source file, and should
/// be used in all subsequent interaction with the SourceManager to refer to
/// this file.
///
/// Note that the file is not loaded immediately. Rather, room in the virtual
/// source space is allocated for it. The file is loaded lazily as soon as it
/// is required.
FileId SourceManager::createFileId(
	/// File to be loaded into the SourceManager.
	const bfs::path& fp) {

	// Lookup the entry for this file. If it is already loaded, immediately
	// return that entry's FileId.
	SourceManagerEntry*& indexEntry = fileSrcIndex[fp];
	if (indexEntry)
		return FileId(indexEntry->id);

	// Create a new entry for this file.
	std::unique_ptr<FileSourceManagerEntry> entry(new FileSourceManagerEntry());
	entry->path = fp;
	entry->size = bfs::file_size(fp);
	bootstrapEntry(entry.get());

	srcTable.push_back(std::move(entry));
	indexEntry = entry.get();

	return FileId(entry->id);
}

/// Puts the \a buffer under management and returns a FileId for it. This id
/// is a highly efficient, unique identifier for this buffer, and should be
/// used in all subsequent interaction with the SourceManager.
///
/// This function provides a mechanism of adding any chunk of memory to the
/// SourceManager as if it was a file. If \a takeOwnership is set to true, the
/// \a buffer will be deallocated when the manager itself is deallocated.
FileId SourceManager::createFileId(
	/// Chunk of source code to be added to the manager.
	const SourceBuffer* buffer) {

	// Lookup the entry for this buffer. If it is already loaded, immediately
	// return that buffer's FileId.
	SourceManagerEntry*& indexEntry = bufferSrcIndex[buffer];
	if (indexEntry)
		return FileId(indexEntry->id);

	// Create a new entry for this buffer.
	std::unique_ptr<BufferSourceManagerEntry> entry(new BufferSourceManagerEntry());
	entry->buffer = *buffer;
	entry->size = buffer->getSize();
	bootstrapEntry(entry.get());

	srcTable.push_back(std::move(entry));
	indexEntry = entry.get();

	return FileId(entry->id);
}

/// Fills in the entry's id and offset. The id is set to be one higher than the
/// the previous table entries, and offset starts where the previous entry left
/// off.
///
/// This function requires that \c size of the entry be set.
void SourceManager::bootstrapEntry(SourceManagerEntry* entry) {
	unsigned id = srcTable.size()+1;
	entry->id = id;
	entry->offset = id > 1 ? srcTable[id-2]->end : 0;
	entry->end = entry->offset + entry->size;
}

/// Returns a SourceBuffer containing the contents of the file \a fid. If the
/// \a fid refers to a real file on disk, the file is loaded the first time
/// this function is called.
const SourceBuffer* SourceManager::getBuffer(FileId fid) {
	assert(fid.id < srcTable.size() && "FileId points outside source table!");
	return srcTable[fid.id]->getBuffer();
}


/// Returns a location that points at the beginning of file \a fid. I.e. the
/// very first byte in the file.
SourceLocation SourceManager::getStartLocation(FileId fid) {
	assert(fid.id < srcTable.size() && "FileId points outside source table!");
	return SourceLocation(srcTable[fid.id]->offset);
}

/// Returns a location that points at the end of file \a fid. I.e. the position
/// after the last byte of the file.
SourceLocation SourceManager::getEndLocation(FileId fid) {
	assert(fid.id < srcTable.size() && "FileId points outside source table!");
	return SourceLocation(srcTable[fid.id]->end);
}

/// Returns the FileId which the location \a loc points at.
FileId SourceManager::getFileIdForLocation(SourceLocation loc) {
	assert(!srcTable.empty() && "source table is empty, nowhere SourceLocation could point!");

	// Make sure we're within the virtual location space.
	if (loc.id >= srcTable.back()->end)
		return FileId(); // invalid id

	// Perform a binary search over the source table.
	unsigned lowIndex = 0;
	unsigned highIndex = srcTable.size();

	while (lowIndex != highIndex) {
		unsigned middleIndex = (lowIndex+highIndex)/2;
		SourceManagerEntry* entry = srcTable[middleIndex].get();

		if (entry->offset > loc.id) {
			highIndex = middleIndex;
		} else if (entry->end <= loc.id) {
			lowIndex = middleIndex;
		} else {
			return FileId(middleIndex);
		}
	}

	return FileId(lowIndex);
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
	SourceManagerEntry* entry = srcTable[fid.id].get();
	unsigned offset = loc.id - entry->offset;

	PresumedLocation r;
	r.filename = entry->getName();
	r.offset = offset;
	r.line = entry->getLineNumberAtOffset(offset);
	r.column = entry->getColumnNumberAtOffset(offset);
	return r;
}
