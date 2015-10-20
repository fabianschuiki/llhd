/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/diagnostic/source_layout.hpp"
#include <algorithm>
#include <iostream>

namespace llhd {


/// \needsdoc
SourceLayout SourceLayout::analyze(Range<const char*> content) {
	SourceLayout result;

	Line current_line;
	bool count_indent = true;
	auto line_start = content.begin();

	for (auto it = content.begin(); it != content.end();) {
		auto c = *it;
		++it;

		if (c == '\n') {
			current_line.length = std::distance(line_start, it);
			result.lines.push_back(current_line);

			line_start = it;
			current_line.offset += current_line.length;
			current_line.indent.spaces = 0;
			current_line.indent.tabs = 0;
			current_line.empty = true;
			count_indent = true;

		} else if (count_indent) {
			if (c == '\t') {
				++current_line.indent.tabs;
			} else if (c == ' ') {
				++current_line.indent.spaces;
			} else {
				count_indent = false;
				current_line.empty = false;
			}
		}
	}

	if (line_start != content.end()) {
		current_line.length = std::distance(line_start, content.end());
		current_line.includes_newline = false;
		result.lines.push_back(current_line);
	}

	return result;
}


/// \needsdoc
unsigned SourceLayout::find_line_index_at_offset(unsigned offset) const {
	auto it = std::lower_bound(
		lines.begin(),
		lines.end(),
		offset,
		[&](Line const& line, unsigned offset){
			return line.offset + line.length <= offset;
		}
	);
	if (it == lines.end())
		return lines.size()-1;
	return std::distance(lines.begin(), it);
}


/// \needsdoc
SourceLayout::Line const& SourceLayout::get_line(unsigned index) const {
	assert(index < lines.size());
	return lines[index];
}


/// \needsdoc
PerceivedLocation SourceLayout::lookup(SourceLocation const& l) const {
	// assert(l.get_source_id() == source->getId());
	unsigned idx = find_line_index_at_offset(l.get_offset());
	assert(idx < lines.size());
	return PerceivedLocation(
		l.get_source_id(), idx, l.get_offset() - lines[idx].offset
	);
}


/// \needsdoc
PerceivedRange SourceLayout::lookup(SourceRange const& r) const {
	// assert(r.get_source_id() == source->getId());
	unsigned idx_start = find_line_index_at_offset(r.get_offset());
	unsigned idx_end   = find_line_index_at_offset(r.get_offset() + r.get_length());
	assert(idx_start < lines.size() && idx_end < lines.size());
	return PerceivedRange(
		r.get_source_id(),
		idx_start, r.get_offset() - lines[idx_start].offset,
		idx_end, r.get_offset() + r.get_length() - lines[idx_end].offset
	);
}


} // namespace llhd
