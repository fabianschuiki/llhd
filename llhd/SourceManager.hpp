/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/MemoryPool.hpp"
#include "llhd/SourceLocation.hpp"
#include <map>

namespace llhd {

class FileEntry;
class SourceBuffer;
class SourceCache;
class SourceManager;
class VirtualSourceEntry;

/// Loads and maintains source files, and creates a continuous location space.
///
/// The SourceManager is used to load files from disk into memory which is
/// garbage collected as soon as the manager is destructed. All loaded files
/// are concatenated into a continuous virtual space, which allows a single
/// integer to specify an exact location within all open files. This location
/// may be decoded by the SourceManager to obtain the actual file, line and
/// column information.
///
/// Files cannot be unloaded and will reside in memory as long as the manager
/// instance lives.
class SourceManager {
	std::map<const FileEntry*, SourceCache> caches;

	std::vector<VirtualSourceEntry*> vsrcTable;
	std::map<const FileEntry*, VirtualSourceEntry*> fileVsrcIndex;
	std::map<const SourceBuffer*, VirtualSourceEntry*> bufferVsrcIndex;

public:
	~SourceManager();

	FileId createFileId(const FileEntry* fe);
	FileId createFileId(const SourceBuffer* buffer, bool takeOwnership = true);

	SourceBuffer* getBuffer(FileId fid);
	SourceBuffer* getBufferForFile(const FileEntry* fe);

	SourceLocation getStartLocation(FileId fid);
	SourceLocation getEndLocation(FileId fid);

	const char* getFilename(SourceLocation loc);
	unsigned getLineNumber(SourceLocation loc);
	unsigned getColumnNumber(SourceLocation loc);
};

} // namespace llhd
