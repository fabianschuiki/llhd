/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class SourceBuffer;
class SourceCache;
class SourceManager;


/// Base class for all source table entries maintained by SourceManager. Other
/// classes derive from this to implement a specific kind of resource that may
/// added to the SourceManager's table of sources. Currently supported are:
///
/// - FileSourceManagerEntry, which points at a file on disk; and
/// - BufferSourceManagerEntry, which points at an arbitrary memory location.
class SourceManagerEntry {
	friend class SourceManager;

	unsigned id;
	unsigned offset;
	unsigned size;
	unsigned end;

	/// Returns a SourceBuffer with the contents of this entry. Overridden by
	/// subclasses to implement loading the resource they wrap around.
	virtual const SourceBuffer* getBuffer() const = 0;

	/// Returns the name of this entry that may be presented to the user.
	/// Overridden by subclasses to return a useful name for the resource they
	/// wrap around.
	virtual const char* getName() const = 0;

	unsigned getLineNumberAtOffset(unsigned offset);
	unsigned getColumnNumberAtOffset(unsigned offset);
};


/// SourceManager table entry for files on disk. Identified by their path, this
/// class loads files lazily as soon as getBuffer() is called for the first
/// time. The entry's name corresponds to the file's path.
class FileSourceManagerEntry : public SourceManagerEntry {
	friend class SourceManager;

	/// Path to the file wrapped by this entry.
	bfs::path path;
	/// Cached buffer
	std::unique_ptr<SourceBuffer> buffer;

	virtual const SourceBuffer* getBuffer() const {
		if (!buffer) {
			// TODO: load buffer here ...
		}
		return buffer.get();
	}

	virtual const char* getName() const {
		return path.c_str();
	}
};


/// SourceManager table entry for arbitrary chunks of memory. This is the most
/// lightweight of all table entries, allowing for arbitrary strings to be used
/// as proper source files. The entry name is fixed to "buffer".
class BufferSourceManagerEntry : public SourceManagerEntry {
	friend class SourceManager;

	/// Buffer wrapped by this entry. This entry does not take any form of
	/// ownership for this buffer.
	const SourceBuffer* buffer;

	virtual const SourceBuffer* getBuffer() const { return buffer; }
	virtual const char* getName() const { return "buffer"; }
};


class SourceCache {
	friend class SourceManager;
};

} // namespace llhd
