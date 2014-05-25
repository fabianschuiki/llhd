/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class FileEntry;
class SourceBuffer;
class SourceCache;
class SourceManager;

class VirtualSourceEntry {
	friend class SourceManager;

	unsigned id;
	unsigned offset;
	unsigned size;

	const FileEntry* origFile;
	const SourceBuffer* origBuffer;
	bool ownsBuffer;

	VirtualSourceEntry():
		id(0),
		offset(0),
		size(0),
		origFile(NULL),
		origBuffer(NULL),
		ownsBuffer(false) {}
};

class SourceCache {
	friend class SourceManager;
};

} // namespace llhd
