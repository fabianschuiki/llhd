/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/SourceBuffer.hpp"
#include "llhd/SourceLocation.hpp"
#include "llhd/TokenContext.hpp"
#include "llhd/vhdl/Lexer.hpp"
#include <iostream>
#include <fstream>

/// \file
/// Passes one or more files through the llhd::vhdl::Lexer.

using namespace llhd;

int main(int argc, char** argv) {
	try {

		// Make sure we have enough arguments.
		if (argc < 2) {
			std::cerr << "no input files\n";
			std::cerr << "usage: " << argv[0] << " filename ...\n";
			return 1;
		}

		// Lex all the source files.
		TokenContext ctx;
		vhdl::Lexer lexer(ctx);
		for (int i = 1; i < argc; i++) {
			std::ifstream fin(argv[i]);
			if (!fin.good()) {
				std::cerr << "unable to open file " << argv[i] << '\n';
				continue;
			}

			fin.seekg(0, std::ios_base::end);
			size_t length = fin.tellg();
			fin.seekg(0, std::ios_base::beg);

			utf8char data[length+1];
			fin.read((char*)data, length);
			data[length] = 0;
			lexer.lex(
				llhd::SourceBuffer(data, data+length+1),
				llhd::SourceLocation());
		}

		std::cout << "lexed " << ctx.getBuffer().getLength() << " tokens\n";

	} catch (std::exception& e) {
		std::cerr << "exception: " << e.what() << '\n';
		return 1;
	}
	return 0;
}
