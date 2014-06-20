/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Diagnostic.hpp"
#include "llhd/DiagnosticFormatterConsole.hpp"
#include "llhd/DiagnosticMessage.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/SourceRangeSet.hpp"
using namespace llhd;


template<class InputIterator>
InputIterator findEnclosingRange(
	InputIterator first,
	InputIterator last,
	SourceRange r) {

	while (first != last) {
		if (first->s <= r.s && first->e >= r.e)
			break;
		++first;
	}
	return first;
}


DiagnosticFormatterConsole& DiagnosticFormatterConsole::operator<<(
	const Diagnostic* diag) {

	std::vector<SourceRange> printedRanges;

	for (unsigned i = 0; i < diag->getNumMessages(); i++) {
		const DiagnosticMessage* msg = diag->getMessage(i);
		std::string pad(2, ' ');

		if (i == 0) {
			auto mr = msg->getMainRange();
			if (mr.isValid()) {
				PresumedRange pr = manager.getPresumedRange(mr);
				output << manager.getBufferName(pr.s.fid) << ":" << pr << ": ";
			}
		} else {
			output << pad;
		}
		switch (msg->getType()) {
			case kFatal: output << "\033[31;1mfatal error:\033[0m"; break;
			case kError: output << "\033[31;1merror:\033[0m"; break;
			case kWarning: output << "\033[33;1mwarning:\033[0m"; break;
			case kNote: output << "\033[1mnote:\033[0m"; break;
			case kFixit: output << "\033[1mfixit:\033[0m"; break;
			default: output << "unspecified:"; break;
		}
		output << " ";
		// output << "\033[1m";

		// Calculate the source code snippets to show in the console.
		SourceRangeSet rngs;
		if (msg->getMainRange().isValid()) {
			rngs.insert(msg->getMainRange());
		}
		rngs.insert(msg->getHighlightedRanges());
		rngs.insert(msg->getRelevantRanges());

		// Also include the source locations set as arguments of the message.
		// If one of the ranges listed as arguments is already fully covered by
		// one of the ranges previously printed there is no need to add it
		// again.
		for (auto arg : msg->getArguments()) {
			if (arg.type == DiagnosticMessageArgument::kSourceRange) {
				auto i = findEnclosingRange(
					printedRanges.begin(),
					printedRanges.end(),
					arg.r);
				if (i == printedRanges.end())
					rngs.insert(arg.r);
			}
		}

		// Add the ranges of the set to the list of printed ranges such that the
		// message may refer to them by index.
		printedRanges.insert(printedRanges.end(), rngs.begin(), rngs.end());

		const char* p = msg->getMessage();
		unsigned line = 0;
		while (*p != 0) {
			if (*p == '\n') {
				if (line++ == 0)
					output << "\033[0m";
				output << '\n' << pad << "  ";
			} else if (*p == '$') {
				p++;
				assert(*p >= '0' && *p <= '9');
				const DiagnosticMessageArgument& arg = msg->getArgument(*p-'0');

				switch (arg.type) {
					case DiagnosticMessageArgument::kSignedInt:
						output << arg.i; break;
					case DiagnosticMessageArgument::kUnsignedInt:
						output << arg.u; break;
					case DiagnosticMessageArgument::kString:
						output << arg.s; break;
					case DiagnosticMessageArgument::kSourceRange: {
						auto i = findEnclosingRange(
							printedRanges.begin(),
							printedRanges.end(),
							arg.r);
						if (i == printedRanges.end()) {
							PresumedRange rng = manager.getPresumedRange(arg.r);
							output << "(" << manager.getBufferName(rng.s.fid)
							       << ':' << rng << ')';
						} else {
							auto id = std::distance(printedRanges.begin(), i)+1;
							output << '(' << id << ')';
						}
					} break;
					default:
						output << "<unknown arg " << *p << '>'; break;
				}
			} else {
				output.put(*p);
			}
			p++;
		}
		// if (i == 0) output << "\033[1m";
		// output << msg->getMessage();
		// if (i == 0) output << "\033[0m";
		output << '\n';

		// Print the source code snippets.
		for (SourceRangeSet::ConstIterator i = rngs.begin();
			i != rngs.end(); ++i) {

			SourceRange sr = *i;
			PresumedRange pr = manager.getPresumedRange(sr);

			output << "  (" << std::distance(rngs.begin(), i)+1 << ") ";
			output << manager.getBufferName(pr.s.fid) << ':' << pr << ":\n";
			output << "    ... source would go here ...\n";
		}


		// if (msg->getMainRange().isValid()) {
		// 	PresumedRange rng = manager.getPresumedRange(msg->getMainRange());
		// 	output << pad << "  (main " << manager.getBufferName(rng.s.fid)
		// 	       << ':' << rng << ")\n";
		// }

		// for (auto r : msg->getHighlightedRanges()) {
		// 	PresumedRange rng = manager.getPresumedRange(r);
		// 	output << pad << "  (highlight " << manager.getBufferName(rng.s.fid)
		// 	       << ':' << rng << ")\n";
		// }

		// for (auto r : msg->getRelevantRanges()) {
		// 	PresumedRange rng = manager.getPresumedRange(r);
		// 	output << pad << "  (relevant " << manager.getBufferName(rng.s.fid)
		// 	       << ':' << rng << ")\n";
		// }

		// output << "- message " << msg->getMessage() << '\n';
		output << '\n';
	}

	return *this;
}
