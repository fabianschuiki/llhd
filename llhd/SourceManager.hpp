/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/MemoryPool.hpp"
#include <map>

namespace llhd {

class FileEntry;
class SourceBuffer;
class SourceCache;
class SourceManager;

/// An opaque identifier that refers to a source file.
class FileId {
	/// 0 is "invalid", everything else refers to a source file.
	unsigned id;

	/// Returns a FileId with the given ID.
	static FileId make(unsigned id) {
		FileId fid;
		fid.id = id;
		return fid;
	}

	friend class SourceManager;

public:
	FileId(): id(0) {}

	bool isValid() const { return id != 0; }
	bool operator== (const FileId& rhs) const { return id == rhs.id; }
	bool operator!= (const FileId& rhs) const { return id != rhs.id; }
	bool operator<  (const FileId& rhs) const { return id <  rhs.id; }
	bool operator>  (const FileId& rhs) const { return id >  rhs.id; }
	bool operator<= (const FileId& rhs) const { return id <= rhs.id; }
	bool operator>= (const FileId& rhs) const { return id >= rhs.id; }
};

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
	MemoryPool<> cacheAllocator;
	std::map<const FileEntry*, SourceCache> caches;

public:
	FileId createFileId(const FileEntry* fe);
	SourceBuffer* getBuffer(FileId fid);
};

} // namespace llhd
