/* Copyright (c) 2015 Fabian Schuiki */
#define CATCH_CONFIG_MAIN
#include "catch.hpp"
#include "llhd/assembly/lexer.hpp"

static const char *input =
	"@global_name %local_name # some random comment\n"
	"i1 l1 ls1\n";

TEST_CASE("basic lexical analysis") {
	using namespace llhd;
	AssemblyLexer lex(make_range(input), SourceLocation(), nullptr);

	REQUIRE(lex);
	REQUIRE(lex.current_token() == TOKEN_SOF);
	REQUIRE(lex.next().current_token() == TOKEN_NAME_GLOBAL);
	REQUIRE(lex.next().current_token() == TOKEN_NAME_LOCAL);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_TYPE);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE(lex.next().current_token() == TOKEN_EOF);
	REQUIRE_FALSE(lex);
}
