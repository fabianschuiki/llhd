/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"

namespace llhd {


/// Base class for all source table entries maintained by SourceManager. Other
/// classes derive from this to implement a specific kind of resource that may
/// added to the SourceManager's table of sources. Currently supported are:
///
/// - FileSourceManagerEntry, which points at a file on disk; and
/// - BufferSourceManagerEntry, which points at an arbitrary memory location.
class SourceManagerEntry {
public:
	unsigned id;
	unsigned offset;
	unsigned size;
	unsigned end;

	std::string name;
	const utf8char* buffer;
	bool bufferAutodelete;

	unsigned getLineNumberAtOffset(unsigned offset) {
		return 1;
	}
	unsigned getColumnNumberAtOffset(unsigned offset) {
		return 1;
	}
};


} // namespace llhd
