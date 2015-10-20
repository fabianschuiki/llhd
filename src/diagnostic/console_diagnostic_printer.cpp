/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/diagnostic/console_diagnostic_printer.hpp"
#include "llhd/utils/console.hpp"
#include <algorithm>
#include <iostream>

namespace llhd {


void ConsoleDiagnosticPrinter::consume(Diagnostic const& d) {
	unsigned msg_indent = 0;

	for (auto& msg : d.get_messages()) {
		std::cout << std::string(msg_indent, ' ');

		switch (msg.get_severity()) {
			case DIAG_FATAL:   std::cout << kout.format(kout.bold,kout.fg_red)     << "fatal: "   << kout.format(kout.fg_def); break;
			case DIAG_ERROR:   std::cout << kout.format(kout.bold,kout.fg_red)     << "error: "   << kout.format(kout.fg_def); break;
			case DIAG_WARNING: std::cout << kout.format(kout.bold,kout.fg_yellow)  << "warning: " << kout.format(kout.fg_def); break;
			case DIAG_INFO:    std::cout << kout.format(kout.bold,kout.fg_magenta) << "info: "    << kout.format(kout.fg_def); break;
			default: break;
		}
		std::cout << msg.get_text() << kout.format(kout.reset) << " [" << msg.get_main_range() << "]" << '\n';


		// Gather the ranges of the source code that are visible in the output.
		std::vector<SourceRange> ranges;
		if (auto mr = msg.get_main_range())
			ranges.push_back(mr);
		ranges.insert(ranges.end(), msg.get_highlit_ranges().begin(), msg.get_highlit_ranges().end());
		ranges.insert(ranges.end(), msg.get_visible_ranges().begin(), msg.get_visible_ranges().end());


		// Transform the ranges into perceived ranges.
		std::vector<PerceivedRange> perceived_ranges(ranges.size());
		std::transform(ranges.begin(), ranges.end(), perceived_ranges.begin(),
			[&](SourceRange const& rng){
				auto source_layout = lookup_source_layout(rng.get_source_id());
				return source_layout.lookup(rng);
			}
		);


		// Transform the perceived ranges into line ranges.
		struct LineRange {
			unsigned order;
			SourceId sid;
			unsigned offset;
			unsigned length;
		};
		unsigned i = 0;
		std::vector<LineRange> lines(perceived_ranges.size());
		std::transform(perceived_ranges.begin(), perceived_ranges.end(), lines.begin(),
			[&](PerceivedRange const& rng){
				return LineRange {
					i++,
					rng.get_source_id(),
					rng.get_first_line(),
					rng.get_last_line() - rng.get_first_line() + 1
				};
			}
		);


		// Merge adjacent line ranges. To do this, first sort the ranges by
		// their position in the file, merge, and restore the original ordering.
		std::sort(lines.begin(), lines.end(),
			[](LineRange const& a, LineRange const& b){
				return a.sid < b.sid || (a.sid == b.sid && a.offset < b.offset);
			}
		);

		const unsigned LINE_MERGE_THRESHOLD = 3; // distance below which two line ranges are merged
		std::vector<LineRange> merged_lines;
		for (auto& b : lines) {
			if (merged_lines.empty()) {
				merged_lines.push_back(b);
				continue;
			}
			auto& a = merged_lines.back();

			if (a.sid != b.sid) {
				merged_lines.push_back(b);
				continue;
			}

			unsigned a0 = a.offset;
			unsigned a1 = a.offset + a.length;
			unsigned b0 = b.offset;
			unsigned b1 = b.offset + b.length;

			if (a1 > b0 || b0 - a1 < LINE_MERGE_THRESHOLD) {
				// std::cout << "-> merging in " << b.order << '\n';
				a.length = std::max(a.length, b1 - a0);
				a.order  = std::min(a.order, b.order);
			} else {
				// std::cout << "-> keeping " << b.order << '\n';
				merged_lines.push_back(b);
			}
		}

		std::sort(merged_lines.begin(), merged_lines.end(),
			[](LineRange const& a, LineRange const& b){
				return a.order < b.order;
			}
		);


		// Print the lines to the console.
		for (auto& l : merged_lines) {
			assert(l.length > 0);

			// Source* source = sourceRepository.getSource(l.sid);
			auto source_layout = lookup_source_layout(l.sid);

			std::cout << "    " << kout.format(kout.dim) << lookup_source_path(l.sid);
			std::cout << ':' << l.offset+1;
			if (l.length > 1)
				std::cout << '-' << l.offset+l.length;
			std::cout << kout.format(kout.undim) << '\n';

			auto common_indent = source_layout.get_line(l.offset).indent;
			for (unsigned i = 1; i < l.length; ++i) {
				auto& line = source_layout.get_line(l.offset+i);
				if (line.empty)
					continue;
				if (line.indent.spaces < common_indent.spaces)
					common_indent.spaces = line.indent.spaces;
				if (line.indent.tabs < common_indent.tabs)
					common_indent.tabs = line.indent.tabs;
			}

			Range<const char*> content = content_callback(l.sid);
			for (unsigned i = 0; i < l.length; ++i) {
				auto& line = source_layout.get_line(l.offset+i);

				unsigned first = line.offset;
				unsigned last = line.offset + line.length;
				if (!line.empty)
					first += common_indent.tabs + common_indent.spaces;

				auto rng = make_range(
					content.begin() + first,
					content.begin() + last
				);

				std::cout << "    ";
				unsigned chars_written = 0;
				for (auto c : rng) {
					if (c == '\t') {
						std::cout << "    ";
						chars_written += 4;
					} else if (c == '\n') {
						std::cout << ' ';
						chars_written += 1;
					} else {
						std::cout << c;
						++chars_written;
					}
				}
				std::cout << '\n';

				std::vector<char> markers(chars_written+1, ' ');
				bool anything_marked = false;

				auto mark = [&](SourceRange const& r, char m){
					if (!r) return;

					auto a = r.get_offset();
					auto b = r.get_offset() + r.get_length();
					if (a >= last) return;
					if (b < first) return;

					auto ra = first < a ? a - first : 0;
					auto rb = b - first;
					if (rb-ra == 0) ++rb;

					std::replace(markers.begin()+ra, markers.begin()+rb, ' ', m);
					anything_marked = true;
				};

				mark(msg.get_main_range(), '^');
				for (auto& r : msg.get_highlit_ranges())
					mark(r, '`');

				if (anything_marked) {
					std::cout << "    " << kout.format(kout.fg_green);
					auto ic = rng.begin();
					auto im = markers.begin();
					for (; ic != rng.end(); ++ic, ++im) {
						if (*ic == '\t')
							std::cout << std::string(4, *im);
						else
							std::cout << *im;
					}
					for (; im != markers.end(); ++im)
						std::cout << *im;
					std::cout << kout.format(kout.fg_def) << '\n';
				}
			}

			// std::cout << "\n";
		}

		msg_indent = 2;
	}

	// std::cout << '\n';
}


std::string ConsoleDiagnosticPrinter::lookup_source_path(SourceId id) {
	return path_callback(id);
}

SourceLayout const& ConsoleDiagnosticPrinter::lookup_source_layout(SourceId id) {
	auto it = m_source_layout_cache.find(id);
	if (it == m_source_layout_cache.end()) {
		SourceLayout sl = SourceLayout::analyze(content_callback(id));
		it = m_source_layout_cache.insert(
			std::make_pair(id, std::move(sl))).first;
	}
	return it->second;
}


} // namespace llhd
