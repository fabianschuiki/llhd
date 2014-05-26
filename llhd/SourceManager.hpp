/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/filesystem.hpp"
#include "llhd/MemoryPool.hpp"
#include "llhd/SourceLocation.hpp"
#include <map>
#include <memory>

namespace llhd {

class FileEntry;
class SourceBuffer;
class SourceCache;
class SourceManager;
class SourceManagerEntry;

/// Loads and maintains source files, and creates a continuous location space.
///
/// The basic usage of SourceManager is as follow:
/// - Source files may be loaded by calling createFileId(), which returns a
///   FileId to be used in subsequent calls to other functions.
/// - The content of a source file may be accessed by calling getBuffer().
/// - Use SourceLocation objects to point locations in a loaded file.
/// - Call getFilename(), getLineNumber(), or getColumnNumber() to convert such
///   a location to a human-readable form.
///
/// Internally, files are loaded lazily when getBuffer() is called for the
/// first time for the corresponding file. The buffers containing the file
/// contents valid as long as the SourceManager exists.
///
/// All loaded files are concatenated into a continuous virtual space, which
/// allows the SourceLocation class to specify an exact location within any
/// open files through only 32 bits, making them highly efficient.
///
/// Some of the concepts are borrowed from llvm::SourceManager.
class SourceManager {
public:
	std::map<bfs::path, SourceCache> caches;

	typedef std::unique_ptr<SourceManagerEntry> TableSlot;
	std::vector<TableSlot> srcTable;
	std::map<bfs::path, SourceManagerEntry*> fileSrcIndex;
	std::map<const SourceBuffer*, SourceManagerEntry*> bufferSrcIndex;

	void bootstrapEntry(SourceManagerEntry* entry);

public:
	~SourceManager();

	FileId createFileId(const bfs::path& fp);
	FileId createFileId(const SourceBuffer* buffer);

	const SourceBuffer* getBuffer(FileId fid);

	SourceLocation getStartLocation(FileId fid);
	SourceLocation getEndLocation(FileId fid);

	const char* getFilename(SourceLocation loc);
	unsigned getLineNumber(SourceLocation loc);
	unsigned getColumnNumber(SourceLocation loc);
};

} // namespace llhd
