/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/unicode/unichar.hpp"
#include "llhd/unicode/utf.hpp"
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>

/// \file
/// Reads the `CaseFolding.txt` file which is part of the unicode character
/// database, and digests its information into an efficient mapping structure
/// which is written to disk.

struct MappingGenerator;
struct MappingFragment;

struct MappingFragment {
	static const unsigned numFrags  = 16;
	static const unsigned maxValues = 32;

	MappingFragment* frags[numFrags];
	unsigned value[maxValues];
	uint16_t id;

	MappingFragment& operator() (unsigned bits);
	MappingFragment& operator= (const unsigned* v);

	MappingGenerator& gen;
	MappingFragment(MappingGenerator& gen);

	void enumerate(unsigned& i, unsigned& j) {
		// i += numFrags;
		for (unsigned n = 0; n < numFrags; n++) {
			auto f = frags[n];
			if (f) {
				if (!f->value[0]) {
					f->id = i;
					i += numFrags;
					f->enumerate(i,j);
				} else {
					assert(j < 0x8000);
					f->id = 0x8000 | (j++);
					for (unsigned k = 0; k < maxValues && f->value[k] != 0; k++)
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
		numNodes = 16;
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
	for (k = 0; k < maxValues && v[k] != 0; k++)
		value[k] = v[k];
	if (k < maxValues)
		value[k] = 0;
	return *this;
}

MappingFragment::MappingFragment(MappingGenerator& gen): gen(gen) {
	id = 0;
	value[0] = 0;
	for (unsigned i = 0; i < numFrags; i++)
		frags[i] = NULL;
}


static void synthesizeNodes(std::ostream& out, const MappingFragment& frag) {
	out << "\t/* " << std::setw(4) << frag.id << " */ ";
	for (unsigned i = 0; i < frag.numFrags; i++) {
		auto& f = frag.frags[i];
		if (i != 0) out << ' ';
		out << "0x" << (f ? f->id : 0) << ',';
	}
	out << '\n';
	for (unsigned i = 0; i < frag.numFrags; i++) {
		auto& f = frag.frags[i];
		if (f && !f->value[0]) synthesizeNodes(out, *f);
	}
}

static void synthesizeLeaves(std::ostream& out, const MappingFragment& frag) {
	if (!frag.value[0]) {
		for (unsigned i = 0; i < frag.numFrags; i++) {
			auto& f = frag.frags[i];
			if (f) synthesizeLeaves(out, *f);
		}
	} else {
		out << "\t/* " << std::setw(4) << frag.id << " */ ";
		for (unsigned i = 0; i < frag.maxValues && frag.value[i] != 0; i++)
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

		// std::cout << "read code point " << std::hex << code << ", status " << status << ", mapping " << mapping[0] << ':' << mapping[1] << ':' << mapping[2] << '\n';

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

		using llhd::unicode::utf8char;
		using llhd::unicode::utf16char;

		// Generate UTF8 and UTF16 sequences for the code and the mapping.
		utf8char  u8c[8],  *p8c = u8c;
		utf16char u16c[8], *p16c = u16c;
		unsigned u8m[32],  *p8m = u8m;
		unsigned u16m[32], *p16m = u16m;

		llhd::unicode::utf8::encode(code, p8c);
		llhd::unicode::utf16::encode(code, p16c);

		for (unsigned* m = mapping; *m != 0; m++) {
			llhd::unicode::utf8::encode(*m, p8m);
			llhd::unicode::utf16::encode(*m, p16m);
		}
		*p8c = 0;
		*p8m = 0;
		*p16c = 0;
		*p16m = 0;

		// Store the mapping for UTF-8, UTF-16, and UTF-32 in the *_full tables
		// if the status indicates a common (C) or full (F) mapping.
		if (status == 'C' || status == 'F') {
			MappingFragment* f;

			f = &map_utf8_full.root;
			for (utf8char* p = u8c; p < p8c; p++)
				f = &(*f)(*p >> 4)(*p >> 0);
			*f = u8m;

			f = &map_utf16_full.root;
			for (utf16char* p = u16c; p < p16c; p++)
				f = &(*f)(*p >> 12)(*p >> 8)(*p >> 4)(*p >> 0);
			*f = u16m;

			map_utf32_full(code >> 16)(code >> 12)(code >> 8)(code >> 4)(code >> 0) = mapping;
		}

		// Store the mapping for UTF-8, UTF-16, and UTF-32 in the *_simple
		// tables if the status indicates a common (C) or simple (S) mapping.
		if (status == 'C' || status == 'S') {
			MappingFragment* f;

			f = &map_utf8_simple.root;
			for (utf8char* p = u8c; p < p8c; p++)
				f = &(*f)(*p >> 4)(*p >> 0);
			*f = u8m;

			f = &map_utf16_simple.root;
			for (utf16char* p = u16c; p < p16c; p++)
				f = &(*f)(*p >> 12)(*p >> 8)(*p >> 4)(*p >> 0);
			*f = u16m;

			map_utf32_simple(code >> 16)(code >> 12)(code >> 8)(code >> 4)(code >> 0) = mapping;
		}
	}

	// Enumerate the mappings. This calculates the layout for the internal table
	// structures.
	map_utf32_full.enumerate();
	map_utf32_simple.enumerate();
	map_utf16_full.enumerate();
	map_utf16_simple.enumerate();
	map_utf8_full.enumerate();
	map_utf8_simple.enumerate();

	// Open the file for writing.
	std::ofstream fout(argv[2]);
	if (!fout.good()) {
		std::cerr << "unable to open output file for writing\n";
		return 3;
	}
	fout << "/* This file was automatically generated by unicode-tblgen-casefolding. DO NOT MODIFY. */\n";
	fout << "#include \"llhd/unicode/casefolding-internal.hpp\"\n";
	fout << std::hex;

	// Synthesize C++ code for the mapping tables.
	auto synth = [&fout](
		const char* type,
		const char* prefix,
		const MappingGenerator& map) {

		fout << "\nconst uint32_t llhd::unicode::" << prefix
		     << "_nodes[] = {\n";
		synthesizeNodes(fout, map.root);
		fout << "\t0\n};\n";

		fout << "\nconst " << type << " llhd::unicode::" << prefix
		     << "_leaves[] = {\n";
		synthesizeLeaves(fout, map.root);
		fout << "\t0\n};\n";
	};

	synth("uint32_t", "utf32_full", map_utf32_full);
	synth("uint32_t", "utf32_simple", map_utf32_simple);
	synth("uint16_t", "utf16_full", map_utf16_full);
	synth("uint16_t", "utf16_simple", map_utf16_simple);
	synth("uint8_t", "utf8_full", map_utf8_full);
	synth("uint8_t", "utf8_simple", map_utf8_simple);

	return 0;
}
