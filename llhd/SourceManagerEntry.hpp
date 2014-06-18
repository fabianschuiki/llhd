/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/unicode/unichar.hpp"
#include <string>
#include <vector>

namespace llhd {

class SourceManagerEntry {
	mutable std::vector<unsigned> lineOffsetCache;
	inline void ensureLineOffsetCache() const;

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

	unsigned getLineNumberAtOffset(unsigned offset) const;
	unsigned getColumnNumberAtOffset(unsigned offset) const;
};

} // namespace llhd
