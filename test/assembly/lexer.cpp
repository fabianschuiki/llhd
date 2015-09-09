/* Copyright (c) 2015 Fabian Schuiki */
#include "catch.hpp"
#include "llhd/assembly/lexer.hpp"

using namespace llhd;


TEST_CASE("empty input") {
	AssemblyLexer lex(make_range(""), SourceLocation(), nullptr);
	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE_FALSE(lex);
}

TEST_CASE("global names") {
	AssemblyLexer lex(make_range("@hello @elaborate\\uC2\\uA9name @"), SourceLocation(), nullptr);
	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_NAME_GLOBAL);
	REQUIRE(lex.current_string() == "@hello");
	REQUIRE(lex.current_range() == SourceRange(SourceId(),0,6));
	REQUIRE(lex.next().current_token() == TOKEN_NAME_GLOBAL);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE_FALSE(lex);
}

TEST_CASE("local names") {
	AssemblyLexer lex(make_range("%hello %elaborate\\uC2\\uA9name %"), SourceLocation(), nullptr);
	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_NAME_LOCAL);
	REQUIRE(lex.current_string() == "%hello");
	REQUIRE(lex.current_range() == SourceRange(SourceId(),0,6));
	REQUIRE(lex.next().current_token() == TOKEN_NAME_LOCAL);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE_FALSE(lex);
}

TEST_CASE("invalid names") {
	{
		AssemblyLexer lex(make_range("@a\\ #blah"), SourceLocation(), nullptr);
		REQUIRE(lex);
		REQUIRE(lex.current_token() == TOKEN_SOF);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE_FALSE(lex);
	}
	{
		AssemblyLexer lex(make_range("@a\\uA #blah"), SourceLocation(), nullptr);
		REQUIRE(lex);
		REQUIRE(lex.current_token() == TOKEN_SOF);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE_FALSE(lex);
	}
}

TEST_CASE("number literals") {
	{
		AssemblyLexer lex(make_range(
			"i1'4 l1'A ls24'DEADBEEF\n" // binary number literals
			"i2'd1023 l523'hDEADBEEF\n" // decimal and hexadecimal number literals
			"i123'" // invalid number literal
		), SourceLocation(), nullptr);
		REQUIRE(lex);
		REQUIRE(lex.current_token() == TOKEN_SOF);
		REQUIRE(lex.next().current_token() == TOKEN_NUMBER_LITERAL);
		REQUIRE(lex.next().current_token() == TOKEN_NUMBER_LITERAL);
		REQUIRE(lex.next().current_token() == TOKEN_NUMBER_LITERAL);
		REQUIRE(lex.next().current_token() == TOKEN_NUMBER_LITERAL);
		REQUIRE(lex.next().current_token() == TOKEN_NUMBER_LITERAL);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE_FALSE(lex);
	}
	{
		AssemblyLexer lex(make_range(
			"i123' # trailing comment" // invalid number literal
		), SourceLocation(), nullptr);
		REQUIRE(lex);
		REQUIRE(lex.current_token() == TOKEN_SOF);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE(lex.next().current_token() == TOKEN_INVALID);
		REQUIRE_FALSE(lex);
	}
}

TEST_CASE("types") {
	AssemblyLexer lex(make_range("i1 l1 ls1 ;some comment\n"), SourceLocation(), nullptr);

	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE_FALSE(lex);
}

TEST_CASE("keywords") {
	AssemblyLexer lex(make_range(
		"mod proc func garbage ;some comment\n"
	), SourceLocation(), nullptr);

	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_KW_MOD);
	REQUIRE(lex.next().current_token() == TOKEN_KW_PROC);
	REQUIRE(lex.next().current_token() == TOKEN_KW_FUNC);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE(lex.next().current_token() == TOKEN_INVALID);
	REQUIRE_FALSE(lex);
}

// TEST_CASE("basic lexical analysis") {
// 	AssemblyLexer lex(make_range(
// 		"@global_name %local_name  # some random comment\n" // names
// 	), SourceLocation(), nullptr);

// 	REQUIRE(lex);
// 	REQUIRE(lex.current_token() == TOKEN_SOF);
// 	REQUIRE(lex.next().current_token() == TOKEN_NAME_GLOBAL);
// 	REQUIRE(lex.current_string() == "@global_name");
// 	REQUIRE(lex.current_range() == SourceRange(SourceId(),0,12));
// 	REQUIRE(lex.next().current_token() == TOKEN_NAME_LOCAL);
// 	REQUIRE(lex.current_string() == "%local_name");
// 	REQUIRE(lex.next().current_token() == TOKEN_EOF);
// 	REQUIRE(lex.next().current_token() == TOKEN_EOF);
// 	REQUIRE_FALSE(lex);
// }
