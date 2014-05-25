/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/SourceManager-internal.hpp"
using namespace llhd;


SourceManager::~SourceManager()
{
	// Deallocate the buffers for which this manager has taken ownership.
	for (auto& pairs : bufferVsrcIndex)
		if (pairs.second->ownsBuffer)
			delete[] pairs.second->origBuffer;
}

/// Puts the file \a fe under management and returns a FileId for it. This id
/// is a highly efficient, unique identifier for this source file, and should
/// be used in all subsequent interaction with the SourceManager to refer to
/// this file.
///
/// Note that the file is not loaded immediately. Rather, room in the virtual
/// source space is allocated for it. The file is loaded lazily as soon as it
/// is required.
FileId SourceManager::createFileId(const FileEntry* fe) {
	// Lookup the entry for this file. If it is already loaded, immediately
	// return that entry's FileId.
	VirtualSourceEntry*& entry = fileVsrcIndex[fe];
	if (entry)
		return FileId(entry->id);

	// Create a new entry for this file.
	unsigned id = vsrcTable.size()+1;
	entry = new VirtualSourceEntry();
	entry->id = id;
	entry->offset = id > 1 ? vsrcTable[id-2]->offset + vsrcTable[id-2]->size : 0;
	// entry->size = fe->getSize();
	entry->origFile = fe;
	vsrcTable.push_back(entry);

	return FileId(id);
}

/// Puts the \a buffer under management and returns a FileId for it. This id
/// is a highly efficient, unique identifier for this buffer, and should be
/// used in all subsequent interaction with the SourceManager.
///
/// This function provides a mechanism of adding any chunk of memory to the
/// SourceManager as if it was a file. If \a takeOwnership is set to true, the
/// \a buffer will be deallocated when the manager itself is deallocated.
FileId SourceManager::createFileId(const SourceBuffer* buffer, bool takeOwnership) {
	// Lookup the entry for this buffer. If it is already loaded, immediately
	// return that buffer's FileId.
	VirtualSourceEntry*& entry = bufferVsrcIndex[buffer];
	if (entry) {
		entry->ownsBuffer = entry->ownsBuffer || takeOwnership; // in case the caller decided to hand us buffer ownership upon the second call
		return FileId(entry->id);
	}

	// Create a new entry for this buffer.
	unsigned id = vsrcTable.size()+1;
	entry = new VirtualSourceEntry();
	entry->id = id;
	entry->offset = id > 1 ? vsrcTable[id-2]->offset + vsrcTable[id-2]->size : 0;
	entry->size = buffer->getBufferSize();
	entry->origBuffer = buffer;
	entry->ownsBuffer = takeOwnership;
	vsrcTable.push_back(entry);

	return FileId(id);
}


SourceBuffer* SourceManager::getBuffer(FileId fid) {
	assert(fid.id < vsrcTable.size() && "FileId points outside vsrc table!");
	VirtualSourceEntry* entry = vsrcTable[fid.id];

	// If the vsrc entry wraps a SourceBuffer, return that immediately.
	if (entry->origBuffer)
		return entry->origBuffer;

	// Otherwise lookup the cache for the wrapped file.
	return getSourceCache(entry->origFile).getBuffer(*this);
}


SourceLocation SourceManager::getStartLocation(FileId fid) {

}

SourceLocation SourceManager::getEndLocation(FileId fid) {

}


const char* SourceManager::getFilename(SourceLocation loc) {

}

unsigned SourceManager::getLineNumber(SourceLocation loc) {

}

unsigned SourceManager::getColumnNumber(SourceLocation loc) {

}

