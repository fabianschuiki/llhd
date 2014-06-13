/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/unicode/casefolding.hpp"
#include "llhd/unicode/casefolding-internal.hpp"

using llhd::unicode::casefolding;
using llhd::unicode::utf8char;
using llhd::unicode::utf16char;
using llhd::unicode::utf32char;


inline const utf8char* fold(
	const utf8char* c,
	const uint32_t* nodes,
	const uint8_t* leaves,
	unsigned* shift) {

	const utf8char* c0 = c;
	unsigned base = 0;
	do {
		base = nodes[base + ((*c >> 4) & 0xF)];
		if (base == 0 || (base & 0x8000))
			break;
		base = nodes[base + (*c & 0xF)];
		c++;
		if (base & 0x8000) {
			if (shift)
				*shift = c-c0;
			return leaves + (base ^ 0x8000);
		}
	} while (base != 0 && *c != 0);

	return c0;
}

inline const utf16char* fold(
	const utf16char* c,
	const uint32_t* nodes,
	const uint16_t* leaves,
	unsigned* shift) {

	return c;
}

inline const utf32char* fold(
	const utf32char* c,
	const uint32_t* nodes,
	const uint32_t* leaves,
	unsigned* shift) {

	return c;
}


const utf8char* casefolding::full(const utf8char* c, unsigned* shift) {
	return fold(c, utf8_full_nodes, utf8_full_leaves, shift);
}

const utf8char* casefolding::simple(const utf8char* c, unsigned* shift) {
	return fold(c, utf8_simple_nodes, utf8_simple_leaves, shift);
}


const utf16char* casefolding::full(const utf16char* c, unsigned* shift) {
	return fold(c, utf16_full_nodes, utf16_full_leaves, shift);
}

const utf16char* casefolding::simple(const utf16char* c, unsigned* shift) {
	return fold(c, utf16_simple_nodes, utf16_simple_leaves, shift);
}


const utf32char* casefolding::full(const utf32char* c, unsigned* shift) {
	return fold(c, utf32_full_nodes, utf32_full_leaves, shift);
}

const utf32char* casefolding::simple(const utf32char* c, unsigned* shift) {
	return fold(c, utf32_simple_nodes, utf32_simple_leaves, shift);
}

