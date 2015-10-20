/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/diagnostic/diagnostic.hpp"
#include "llhd/diagnostic/console_diagnostic_printer.hpp"
#include "llhd/assembly/assembly.hpp"
#include "llhd/assembly/reader.hpp"
#include "llhd/utils/memory.hpp"
#include "llhd/utils/readfile.hpp"
#include "llhd/location.hpp"
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

const char *arg0 = nullptr;

struct Options {
};

struct Source {
	llhd::SourceId sid;
	std::string path;
	std::vector<char> content;
};


void parse_options(int &argc, char **argv, Options &opts) {
	// not yet implemented
}


int main(int argc, char **argv) {
	using namespace llhd;

	arg0 = *argv;
	++argv;
	--argc;

	Options opts;
	parse_options(argc, argv, opts);

	if (argc == 0) {
		std::cerr << "usage: " << arg0 << " INPUT...\n";
		return 1;
	}

	DiagnosticContext dctx;
	Assembly as;
	AssemblyReader rd(as);
	std::vector<std::unique_ptr<Source>> sources;

	for (unsigned i = 0; i < (unsigned)argc; ++i) {
		sources.push_back(make_unique<Source>());
		auto &src = *sources.back();
		src.sid = SourceId(sources.size());
		src.path = argv[i];
		if (!readfile(argv[i], src.content)) {
			std::cerr << "unable to read file " << argv[i] << '\n';
			return 1;
		}
		rd(make_range(&*src.content.begin(), &*src.content.end()), SourceLocation(src.sid), &dctx);
	}

	ConsoleDiagnosticPrinter printer;
	printer.path_callback = [&](SourceId sid) -> std::string {
		unsigned id = sid.get_id()-1;
		if (id < sources.size())
			return sources[id]->path;
		return "<unknown>";
	};
	printer.content_callback = [&](SourceId sid) -> Range<const char*> {
		unsigned id = sid.get_id()-1;
		if (id < sources.size())
			return make_range(&*sources[id]->content.begin(), &*sources[id]->content.end());
		return make_range(nullptr, nullptr);
	};
	for (auto &d : dctx.get_diagnostics())
		printer.consume(d);

	if (dctx.is_error())
		return 1;

	std::cout << to_string(as) << "\n";

	return 0;
}
