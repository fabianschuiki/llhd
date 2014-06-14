/* Copyright (c) 2014 Fabian Schuiki */
/* Unit tests for unicode case folding algorithms. */

#define BOOST_TEST_MODULE unicode_casefolding
#include <llhd/unicode/casefolding.hpp>
#include <boost/test/unit_test.hpp>

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
		s << ((unsigned)*p & ((1 << sizeof(T)*8)-1));
	}
	s << '"';
	return s.str();
}

template<class InputIterator>
std::string dump(InputIterator first, InputIterator last) {
	std::stringstream s;
	s << std::hex;
	s << '"';
	for (InputIterator p = first; p != last; p++) {
		if (p != first) s << ':';
		s << ((unsigned)*p & ((1 << sizeof(typename InputIterator::value_type)*8)-1));
	}
	s << '"';
	return s.str();
}

template<typename T, typename U>
void check_fold(const U* in, const U* simple, const U* full) {
	const T* ptr = (const T*)in;
	const T* outs = llhd::unicode::casefold<T,false>(ptr);
	const T* outf = llhd::unicode::casefold<T,true>(ptr);
	BOOST_CHECK_MESSAGE(strcmp((const char*)outs, (const char*)simple) == 0,
		"simple(" << dump(ptr) <<
		") = " << dump(outs) <<
		", expected " << dump((const T*)simple));
	BOOST_CHECK_MESSAGE(strcmp((const char*)outf, (const char*)full) == 0,
		"full(" << dump(ptr) <<
		") = " << dump(outs) <<
		", expected " << dump((const T*)full));
}

struct FoldTestCase {
	const char *in;
	const char *outs, *outf;
	FoldTestCase(const char *in = 0):
		in(in),
		outs(in),
		outf(in) {}
	FoldTestCase(const char *in, const char *out):
		in(in),
		outs(out),
		outf(out) {}
	FoldTestCase(const char *in, const char *outs, const char *outf):
		in(in),
		outs(outs),
		outf(outf) {}
};

BOOST_AUTO_TEST_CASE(characters) {
	static const FoldTestCase cases[] = {
		{"a"}, {"ä"}, {"ö"}, {"à"}, {"€"}, {"å"}, {"ø"},
		{"A", "a"},
		{"ß", "ß", "ss"},
		{"Å", "å"},
		{"\u0391", "\u03b1"},
		{"ὒ", "ὒ", "υ\u0313\u0300"},
		{}
	};

	for (const FoldTestCase* c = cases; c->in != 0; c++) {
		check_fold<utf8char>(c->in, c->outs, c->outf);
	}
}

BOOST_AUTO_TEST_CASE(strings) {
	static const FoldTestCase cases[] = {
		{"HeLLo WorlD", "hello world"},
		{"GrÜße Sie", "grüße sie", "grüsse sie"},
		{"HELLO WORLD", "hello world"},
		{}
	};

	for (const FoldTestCase* c = cases; c->in != 0; c++) {
		llhd::unicode::casefold_iterator<utf8char,true>  itf((utf8char*)c->in);
		llhd::unicode::casefold_iterator<utf8char,false> its((utf8char*)c->in);
		llhd::unicode::casefold_iterator<utf8char,true>  endf;
		llhd::unicode::casefold_iterator<utf8char,false> ends;

		BOOST_CHECK_MESSAGE(std::equal(itf, endf, (utf8char*)c->outf),
			"full(\"" << c->in <<
			"\") = \"" << std::string(itf, endf) <<
			"\", expected \"" << c->outf <<
			"\"\n    " << dump(itf, endf) << " !=\n    " << dump(c->outf));
		BOOST_CHECK_MESSAGE(std::equal(its, ends, (utf8char*)c->outs),
			"simple(\"" << c->in <<
			"\") = \"" << std::string(its, ends) <<
			"\", expected \"" << c->outs <<
			"\"\n    " << dump(its, ends) << " !=\n    " << dump(c->outs));
		// check_fold<utf8char>(c->in, c->outs, c->outf);
	}
}
