/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/SourceManager-internal.hpp"
using namespace llhd;


SourceManager::~SourceManager() {
}

/// Puts the file \a fe under management and returns a FileId for it. This id
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
	bootstrapEntry(entry.get());
	entry->size = bfs::file_size(fp);
	entry->path = fp;

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
	bootstrapEntry(entry.get());
	entry->buffer = buffer;
	entry->size = buffer->getBufferSize();

	srcTable.push_back(std::move(entry));
	indexEntry = entry.get();

	return FileId(entry->id);
}

/// Fills in the entry's id and offset. The id is set to be one higher than the
/// the previous table entries, and offset starts where the previous entry left
/// off.
void SourceManager::bootstrapEntry(SourceManagerEntry* entry) {
	unsigned id = srcTable.size()+1;
	entry->id = id;
	entry->offset = id > 1 ? srcTable[id-2]->offset + srcTable[id-2]->size : 0;
}


const SourceBuffer* SourceManager::getBuffer(FileId fid) {
	assert(fid.id < srcTable.size() && "FileId points outside vsrc table!");
	return srcTable[fid.id]->getBuffer();
}


SourceLocation SourceManager::getStartLocation(FileId fid) {
	return SourceLocation();
}

SourceLocation SourceManager::getEndLocation(FileId fid) {
	return SourceLocation();
}


const char* SourceManager::getFilename(SourceLocation loc) {
	return NULL;
}

unsigned SourceManager::getLineNumber(SourceLocation loc) {
	return 0;
}

unsigned SourceManager::getColumnNumber(SourceLocation loc) {
	return 0;
}

