/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/unicode.hpp"
#include "llhd/allocator/PoolAllocator.hpp"
#include <fstream>
#include <iostream>
#include <sstream>

/// \file Reads the `CaseFolding.txt` file which is part of the Unicode
/// Character database, and digests its information into an efficient mapping
/// structure which is written to disk.

struct MappingGenerator;
struct MappingFragment;

struct MappingFragment {
	MappingFragment* frags[16];
	unsigned value[8];
	unsigned id;

	MappingFragment& operator() (unsigned bits);
	MappingFragment& operator= (const unsigned* v);

	MappingGenerator& gen;
	MappingFragment(MappingGenerator& gen);

	void enumerate(unsigned& i, unsigned& j) {
		i += 16;
		for (unsigned n = 0; n < 16; n++) {
			auto f = frags[n];
			if (f) {
				if (!f->value[0]) {
					f->id = i;
					i += 16;
					f->enumerate(i,j);
				} else {
					f->id = 0x80000000 | (j++);
					for (unsigned k = 0; k < 8 && f->value[k] != 0; k++)
						j++;
				}
			}
		}
	}
};

struct MappingGenerator {
	llhd::PoolAllocator<> alloc;
	MappingFragment root;
	unsigned numNodes, numLeaves;

	MappingGenerator(): root(*this) {}

	MappingFragment& operator() (unsigned bits) {
		return root(bits);
	}

	void enumerate() {
		numNodes = 0;
		numLeaves = 0;
		root.enumerate(numNodes, numLeaves);
	}
};

MappingFragment& MappingFragment::operator() (unsigned bits) {
	bits &= 0xf;
	auto& frag = frags[bits];
	if (!frag)
		frag = gen.alloc.one<MappingFragment>(gen);
	return *frag;
}

MappingFragment& MappingFragment::operator= (const unsigned* v) {
	unsigned k;
	for (k = 0; k < 8 && v[k] != 0; k++)
		value[k] = v[k];
	if (k < 8)
		value[k] = 0;
	return *this;
}

MappingFragment::MappingFragment(MappingGenerator& gen): gen(gen) {
	value[0] = 0;
	for (int i = 0; i < 16; i++)
		frags[i] = NULL;
}


static void synthesizeNodes(std::ostream& out, const MappingFragment& frag) {
	out << "\t/* " << frag.id << " */ ";
	for (unsigned i = 0; i < 16; i++) {
		auto& f = frag.frags[i];
		if (i != 0) out << ' ';
		out << "0x" << (f ? f->id : 0) << ',';
	}
	out << '\n';
	for (unsigned i = 0; i < 16; i++) {
		auto& f = frag.frags[i];
		if (f && !f->value[0]) synthesizeNodes(out, *f);
	}
}

static void synthesizeLeaves(std::ostream& out, const MappingFragment& frag) {
	if (!frag.value[0]) {
		for (unsigned i = 0; i < 16; i++) {
			auto& f = frag.frags[i];
			if (f) synthesizeLeaves(out, *f);
		}
	} else {
		out << "\t/* " << frag.id << " */ ";
		for (unsigned i = 0; i < 8 && frag.value[i] != 0; i++)
			out << "0x" << frag.value[i] << ", ";
		out << "0,\n";
	}
}


int main(int argc, char** argv)
{
	// Verify we have enough arguments.
	if (argc != 3) {
		std::cerr << "usage: " << argv[0] << " <input> <output>\n";
		return 1;
	}

	// Open the file for reading.
	std::ifstream fin(argv[1]);
	if (!fin.good()) {
		std::cerr << "unable to open input file for writing\n";
		return 2;
	}

	// Parse the input file and generate the different table versions.
	MappingGenerator map_utf32_full, map_utf32_simple;
	MappingGenerator map_utf16_full, map_utf16_simple;
	MappingGenerator map_utf8_full, map_utf8_simple;

	while (fin.good()) {

		// Skip empty lines.
		if (fin.peek() == '\n') {
			fin.get();
			continue;
		}

		// Skip comment lines.
		if (fin.peek() == '#') {
			while (fin.get() != '\n' && fin.good());
			continue;
		}

		// Abort in case the peeking caused the stream to reach its end.
		if (!fin.good())
			break;

		unsigned code;
		char status;
		unsigned mapping[16] = {0};

		fin >> std::hex >> code;
		while (fin.peek() == ';' || fin.peek() == ' ')
			fin.get();
		fin >> status;
		while (fin.peek() == ';' || fin.peek() == ' ')
			fin.get();

		unsigned i;
		for (i = 0; i < 15;) {
			fin >> std::hex >> mapping[i++];
			if (fin.peek() == ';')
				break;
			while (fin.peek() == ' ')
				fin.get();
		}
		mapping[i] = 0;
		while (fin.get() != '\n' && fin.good());

		std::cout << "read code point " << std::hex << code << ", status " << status << ", mapping " << mapping[0] << ':' << mapping[1] << ':' << mapping[2] << '\n';

		// Generate the UTF32 code points.
		// std::stringstream utf32v;
		// utf32v << "{ " << std::hex;
		// for (i = 0; i < 16 && mapping[i] != 0; i++) {
		// 	if (i != 0)
		// 		utf32v << ", ";
		// 	utf32v << "0x" << mapping[i];
		// }
		// utf32v << ", 0 }";
		// std::cout << "  = " << utf32v.str() << '\n';

		if (status == 'C' || status == 'F')
			map_utf32_full(code >> 16)(code >> 12)(code >> 8)(code >> 4)(code >> 0) = mapping;// = utf32v.str();
		if (status == 'C' || status == 'S')
			map_utf32_simple(code >> 16)(code >> 12)(code >> 8)(code >> 4)(code >> 0) = mapping;// = utf32v.str();
	}

	// Enumerate the mappings.
	map_utf32_full.enumerate();
	map_utf32_simple.enumerate();
	map_utf16_full.enumerate();
	map_utf16_simple.enumerate();
	map_utf8_full.enumerate();
	map_utf8_simple.enumerate();

	std::cout << "utf32_full has " << map_utf32_full.numNodes << " entries\n";
	std::cout << "utf32_simple has " << map_utf32_simple.numLeaves << " entries\n";

	// Open the file for writing.
	std::ofstream fout(argv[2]);
	if (!fout.good()) {
		std::cerr << "unable to open output file for writing\n";
		return 3;
	}
	fout << "/* This file was automatically generated by unicode-gen-casefolding. DO NOT MODIFY. */\n";
	fout << "#include \"llhd/unicode-internal.hpp\"\n";
	fout << std::hex;

	// UTF32 Full
	fout << "\nconst uint32_t llhd::unicode::utf32::full::nodes[] = {\n";
	synthesizeNodes(fout, map_utf32_full.root);
	fout << "\t0\n};\n";

	fout << "\nconst uint32_t llhd::unicode::utf32::full::leaves[] = {\n";
	synthesizeLeaves(fout, map_utf32_full.root);
	fout << "\t0\n};\n";

	// UTF32 Simple
	fout << "\nconst uint32_t llhd::unicode::utf32::simple::nodes[] = {\n";
	synthesizeNodes(fout, map_utf32_simple.root);
	fout << "\t0\n};\n";

	fout << "\nconst uint32_t llhd::unicode::utf32::simple::leaves[] = {\n";
	synthesizeLeaves(fout, map_utf32_simple.root);
	fout << "\t0\n};\n";

	return 0;
}
