/* Copyright (c) 2014 Fabian Schuiki */
#define BOOST_TEST_MODULE asm_lexer_module
#include "llhd/AssemblyLexer.hpp"
#include "llhd/SourceLocation.hpp"
#include <boost/test/unit_test.hpp>
using namespace llhd;

const char* src = "\
# a comment\n\
define @marx_tb {\n\
	%0 = inst @st\\02uff.asd_a23\n\
	process initial storage\n\
	$32b12997851234 /\n\
}";

BOOST_AUTO_TEST_CASE(asm_lexer_case) {
	AssemblyLexer a(SourceLocation(), Buffer<const char>(src), nullptr);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kKeywordDefine);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kIdentifierGlobal);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kSymbolLBrace);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kIdentifierLocal);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kSymbolEqual);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kIdentifierReserved);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kIdentifierGlobal);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kKeywordProcess);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kKeywordInitial);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kKeywordStorage);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kNumberLiteral);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kSymbolSlash);
	BOOST_CHECK_EQUAL(a.next().getToken(), AssemblyLexer::kSymbolRBrace);
	BOOST_CHECK(!a.next());
}
