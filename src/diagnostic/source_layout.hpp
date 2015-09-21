/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/location.hpp"
#include "llhd/utils/range.hpp"
#include <vector>

namespace llhd {


/// \needsdoc
/// \ingroup diagnostic
class SourceLayout {
public:
	struct Line {
		unsigned offset = 0;
		unsigned length = 0;
		struct {
			unsigned spaces = 0;
			unsigned tabs   = 0;
		} indent;
		bool includes_newline = true;
		bool empty = true;
	};

	static SourceLayout analyze(Range<const char*> content);
	unsigned find_line_index_at_offset(unsigned offset) const;
	Line const& get_line(unsigned index) const;
	std::vector<Line> const& get_lines() const { return lines; }

	PerceivedLocation lookup(SourceLocation const& l) const;
	PerceivedRange lookup(SourceRange const& r) const;

private:
	std::vector<Line> lines;
	SourceLayout() {}
};


} // namespace llhd
