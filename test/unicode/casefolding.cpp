/* Copyright (c) 2014-2015 Fabian Schuiki */
#include "catch.hpp"
#include "llhd/unicode/casefolding.hpp"

using namespace llhd;


/// Asserts that casefolding yields the given outputs under simple and full
/// folding for the given input.
static void check_fold(std::string in, std::string out_simple, std::string out_full) {
	using simple = llhd::unicode::CasefoldIterator<utf8char,false>;
	using full   = llhd::unicode::CasefoldIterator<utf8char,true>;

	REQUIRE(std::string(simple((const utf8char*)in.c_str()), simple()) == out_simple);
	REQUIRE(std::string(full((const utf8char*)in.c_str()), full()) == out_full);
}

/// Asserts that simple and full casefolding yield the given output for the
/// given input.
static void check_fold(std::string in, std::string out) {
	check_fold(in, out, out);
}

/// Asserts that casefolding is an identity operation on the given input.
static void check_fold(std::string in) {
	check_fold(in, in);
}


TEST_CASE("characters") {
	check_fold("a");
	check_fold("ä");
	check_fold("ö");
	check_fold("à");
	check_fold("€");
	check_fold("å");
	check_fold("ø");

	check_fold("A", "a");
	check_fold("Ä", "ä");
	check_fold("Ö", "ö");
	check_fold("À", "à");
	check_fold("Å", "å");
	check_fold("Ø", "ø");

	check_fold("ß", "ß", "ss");
	check_fold("\u0391", "\u03b1");
	check_fold("ὒ", "ὒ", "υ\u0313\u0300");
}


TEST_CASE("strings") {
	check_fold("HeLLo WorlD", "hello world");
	check_fold("GrÜße Sie", "grüße sie", "grüsse sie");
	check_fold("HELLO WORLD", "hello world");
}
