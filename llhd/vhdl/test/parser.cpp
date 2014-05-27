/* Copyright (c) 2014 Fabian Schuiki */
#include <llhd/vhdl/Parser.hpp>
#include <llhd/vhdl/ast/Context.hpp>
#include <iostream>
#include <fstream>


int main(int argc, char** argv)
{
	try {

		// Make sure we have enough arguments.
		if (argc < 2) {
			std::cerr << "no input files\n";
			std::cerr << "usage: " << argv[0] << " filename ...\n";
			return 1;
		}

		// Create an instance of the parser and feed it each file sequentially.
		llhd::vhdl::Parser parser;
		for (int i = 1; i < argc; i++) {
			std::ifstream fin(argv[i]);
			parser.parse(fin);
		}

		llhd::vhdl::ast::Context ctx;
		ctx.alloc.allocate(128);

	} catch (std::exception& e) {
		std::cerr << "exception: " << e.what() << '\n';
		return 1;
	}
	return 0;
}
