/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/filesystem.hpp"
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/unicode/unichar.hpp"
#include <vector>

namespace llhd {

class SourceManagerEntry {
public:
	unsigned id;
	unsigned offset;
	unsigned size;
	unsigned end;

	std::string name;
	const utf8char* buffer;

	SourceManagerEntry(
		unsigned id,
		unsigned offset,
		unsigned size,
		unsigned end):
		id(id),
		offset(offset),
		size(size),
		end(end) {}

	unsigned getLineNumberAtOffset(unsigned offset) const {
		return 1;
	}
	unsigned getColumnNumberAtOffset(unsigned offset) const {
		return 1;
	}
};

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
	std::vector<SourceManagerEntry> srcTable;
	SourceManagerEntry& makeEntry(unsigned size);

public:
	/// Allocator that provides garbage collected memory for objects whose
	/// existence should be tied to the SourceManager.
	PoolAllocator<> alloc;

	FileId addBuffer(const SourceBuffer& buffer, const std::string& name);
	FileId addBufferCopy(const SourceBuffer& buffer, const std::string& name);

	SourceBuffer getBuffer(FileId fid);

	SourceLocation getStartLocation(FileId fid);
	SourceLocation getEndLocation(FileId fid);

	FileId getFileIdForLocation(SourceLocation loc);
	PresumedLocation getPresumedLocation(SourceLocation loc);
};

} // namespace llhd
