/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyLexer.hpp"
#include "llhd/AssemblyParser.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticFormatterConsole.hpp"
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceManager.hpp"
#include <fstream>
using namespace llhd;

/// \file
/// A simple simulator for hardware described in LLHD assembly. Absolutely
/// suboptimal in performance and feature set, as this tool is intended to be a
/// proof of concept and LLHD usage template.

int main(int argc, char** argv) {
	--argc;
	++argv;

	if (!argc) {
		std::cerr << "no input files\n";
		return 1;
	}

	// Read the input assembly files.
	Assembly as;
	SourceManager sm;
	DiagnosticContext diag;
	DiagnosticFormatterConsole diagfmt(std::cerr, sm);
	while (argc) {
		auto path = *argv;
		--argc;
		++argv;

		// Read the entire file into a buffer maintained by the source manager.
		std::ifstream fin(path);
		if (!fin.good()) {
			std::cerr << "unable to open file " << path << '\n';
			continue;
		}

		fin.seekg(0, std::ios_base::end);
		size_t length = fin.tellg();
		fin.seekg(0, std::ios_base::beg);

		utf8char* data = (utf8char*)sm.alloc.allocate(length);
		fin.read((char*)data, length);

		SourceBuffer buffer(data,length);
		auto fid = sm.addBuffer(buffer, path);

		// Parse the file into the assembly structure.
		AssemblyLexer lex(sm.getStartLocation(fid), buffer, &diag);
		AssemblyParser(as, lex, &diag);

		// Abort if parsing failed failed.
		if (diag.isErrorSeverity()) {
			diagfmt << diag;
			return 1;
		}
	}

	// Dump the issues.
	diagfmt << diag;

	return 0;
}
