/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/unicode/casefolding.hpp"
#include "llhd/unicode/casefolding-internal.hpp"
#include "llhd/unicode/utf.hpp"
using namespace llhd;


inline unsigned resolveBase(const uint32_t* nodes, unsigned base, utf8char c) {
	base = nodes[base + ((c >> 4) & 0xF)];
	if (base == 0)
		return 0;
	return nodes[base + (c & 0xF)];
}

inline unsigned resolveBase(const uint32_t* nodes, unsigned base, utf16char c) {
	base = resolveBase(nodes, base, (utf8char)(c >> 8));
	if (base == 0)
		return 0;
	return resolveBase(nodes, base, (utf8char)c);
}

inline unsigned resolveBase(const uint32_t* nodes, unsigned base, utf32char c) {
	base = resolveBase(nodes, base, (utf16char)(c >> 16));
	if (base == 0)
		return 0;
	return resolveBase(nodes, base, (utf16char)c);
}

template<typename T, typename L>
const T* fold(
	const T* c,
	const uint32_t* nodes,
	const L* leaves,
	unsigned* shift) {

	const T* c0 = c;
	unsigned base = 0;
	do {
		base = resolveBase(nodes, base, *c);
		c++;
		if (base & 0x8000) {
			if (shift)
				*shift = c-c0;
			return leaves + (base ^ 0x8000);
		}
	} while (base != 0 && *c != 0);

	return c0;
}


template<> const utf8char*
llhd::unicode::casefold<utf8char,true>(const utf8char* c, unsigned* shift) {
	if (llhd::unicode::utf8::isTrail(*c))
		return c;
	return fold(c, utf8_full_nodes, utf8_full_leaves, shift);
}

template<> const utf8char*
llhd::unicode::casefold<utf8char,false>(const utf8char* c, unsigned* shift) {
	if (llhd::unicode::utf8::isTrail(*c))
		return c;
	return fold(c, utf8_simple_nodes, utf8_simple_leaves, shift);
}


template<> const utf16char*
llhd::unicode::casefold<utf16char,true>(const utf16char* c, unsigned* shift) {
	if (llhd::unicode::utf16::isTrail(*c))
		return c;
	return fold(c, utf16_full_nodes, utf16_full_leaves, shift);
}

template<> const utf16char*
llhd::unicode::casefold<utf16char,false>(const utf16char* c, unsigned* shift) {
	if (llhd::unicode::utf16::isTrail(*c))
		return c;
	return fold(c, utf16_simple_nodes, utf16_simple_leaves, shift);
}


template<> const utf32char*
llhd::unicode::casefold<utf32char,true>(const utf32char* c, unsigned* shift) {
	return fold(c, utf32_full_nodes, utf32_full_leaves, shift);
}

template<> const utf32char*
llhd::unicode::casefold<utf32char,false>(const utf32char* c, unsigned* shift) {
	return fold(c, utf32_simple_nodes, utf32_simple_leaves, shift);
}
