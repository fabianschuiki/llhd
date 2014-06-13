/* Copyright (c) 2014 Fabian Schuiki */
/* Unit tests for unicode case folding algorithms. */

#define BOOST_TEST_MODULE unicode_test
#include <llhd/unicode/casefolding.hpp>
#include <boost/test/unit_test.hpp>

using llhd::unicode::casefolding;
using llhd::unicode::utf8char;
using llhd::unicode::utf16char;
using llhd::unicode::utf32char;

template<typename T>
std::string dump(const T* in) {
	std::stringstream s;
	s << std::hex;
	s << '"';
	for (const T* p = in; *p != 0; p++) {
		if (p != in) s << ':';
		s << (unsigned)*p;
	}
	s << '"';
	return s.str();
}

template<typename T, typename U>
void check_fold(const U* in, const U* simple, const U* full) {
	const T* ptr = (const T*)in;
	const T* outs = casefolding::simple(ptr);
	const T* outf = casefolding::full(ptr);
	BOOST_CHECK_MESSAGE(strcmp((const char*)outs, (const char*)simple) == 0,
		"simple(" << dump(ptr) <<
		") = " << dump(outs) <<
		", expected " << dump((const T*)simple));
	BOOST_CHECK_MESSAGE(strcmp((const char*)outf, (const char*)full) == 0,
		"full(" << dump(ptr) <<
		") = " << dump(outs) <<
		", expected " << dump((const T*)full));
}

BOOST_AUTO_TEST_CASE(unicode_casefolding) {

	static const char* nofolds_utf8[] = {
		"a", "ä", "ö", "à", "€", "å", "ø", 0
	};
	static const struct { const char *i, *os, *of; } folds_utf8[] = {
		{"A", "a", "a"},
		{"ß", "ß", "ss"},
		{"Å", "å", "å"},
		{"\u0391", "\u03b1", "\u03b1"},
		{"ὒ", "ὒ", "υ\u0313\u0300"},
		{0}
	};

	for (unsigned i = 0; nofolds_utf8[i] != 0; i++) {
		check_fold<utf8char>(
			nofolds_utf8[i],
			nofolds_utf8[i],
			nofolds_utf8[i]);
	}
	for (unsigned i = 0; folds_utf8[i].i != 0; i++) {
		check_fold<utf8char>(
			folds_utf8[i].i,
			folds_utf8[i].os,
			folds_utf8[i].of);
	}
}
