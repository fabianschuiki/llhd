/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class FileEntry;
class SourceBuffer;
class SourceCache;
class SourceManager;


class SourceManagerEntry {
	friend class SourceManager;

	unsigned id;
	unsigned offset;
	unsigned size;

	/// Returns a SourceBuffer with the contents of this entry. Overridden by
	/// subclasses to implement loading the resource they wrap around.
	virtual const SourceBuffer* getBuffer() const = 0;

	/// Returns the name of this entry that may be presented to the user.
	/// Overridden by subclasses to return a useful name for the resource they
	/// wrap around.
	virtual const char* getName() const = 0;
};


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
