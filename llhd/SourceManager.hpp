/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"
#include "llhd/SourceManagerEntry.hpp"
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/unicode/unichar.hpp"
#include <vector>

namespace llhd {

/// Loads and maintains source files, and creates a continuous location space.
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
	/// The table of maintained entries.
	std::vector<SourceManagerEntry> srcTable;

	SourceManagerEntry& makeEntry(unsigned size);
	inline SourceManagerEntry& getEntry(FileId fid);

	/// Cache for the getFileIdForLocation() function.
	struct {
		unsigned offset, end;
		unsigned id;
	} lastFileIdForLocation;

public:
	SourceManager();

	/// Allocator that provides garbage collected memory for objects whose
	/// existence should be tied to the SourceManager.
	PoolAllocator<> alloc;

	FileId addBuffer(const SourceBuffer& buffer, const std::string& name);
	FileId addBufferCopy(const SourceBuffer& buffer, const std::string& name);

	SourceBuffer getBuffer(FileId fid);
	const std::string& getBufferName(FileId fid);

	SourceLocation getStartLocation(FileId fid);
	SourceLocation getEndLocation(FileId fid);

	FileId getFileIdForLocation(SourceLocation loc);
	PresumedLocation getPresumedLocation(SourceLocation loc);
	PresumedRange getPresumedRange(SourceRange rng);
};

} // namespace llhd
