/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"
#include "llhd/SourceManager.hpp"
#include "llhd/TokenContext.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticFormatterConsole.hpp"
#include "llhd/vhdl/Lexer.hpp"
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/ast/Context.hpp"
#include <iostream>
#include <fstream>

using namespace llhd;

int main(int argc, char** argv) {
	try {

		// Make sure we have enough arguments.
		if (argc < 2) {
			std::cerr << "no input files\n";
			std::cerr << "usage: " << argv[0] << " filename ...\n";
			return 1;
		}

		// Read and lex all the source files.
		SourceManager manager;
		DiagnosticContext diag;
		DiagnosticFormatterConsole fmt(std::cout, manager);
		TokenContext ctx;
		vhdl::Lexer lexer(ctx, diag);
		for (int i = 1; i < argc; i++) {
			std::ifstream fin(argv[i]);
			if (!fin.good()) {
				std::cerr << "unable to open file " << argv[i] << '\n';
				continue;
			}

			fin.seekg(0, std::ios_base::end);
			size_t length = fin.tellg();
			fin.seekg(0, std::ios_base::beg);

			utf8char* data = (utf8char*)manager.alloc.allocate(length);
			fin.read((char*)data, length);

			SourceBuffer buffer(data,length);
			auto fid = manager.addBuffer(buffer, argv[i]);
			lexer.lex(buffer, manager.getStartLocation(fid));
		}

		// Abort if the lexer failed.
		if (diag.isFatal()) {
			fmt << diag;
			return 1;
		}

		// Parse the tokens.
		vhdl::ast::Context astctx;
		vhdl::Parser parser(astctx, diag);
		parser.parse(ctx.getBuffer());

		// Format the diagnostics to the console.
		fmt << diag;
		return diag.isFatal() ? 1 : 0;

	} catch (std::exception& e) {
		std::cerr << "exception: " << e.what() << '\n';
		return 1;
	}
	return 0;
}
