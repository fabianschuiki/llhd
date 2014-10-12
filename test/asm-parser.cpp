/* Copyright (c) 2014 Fabian Schuiki */
#define BOOST_TEST_MODULE asm_parser_module
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyLexer.hpp"
#include "llhd/AssemblyParser.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticFormatterConsole.hpp"
#include "llhd/SourceLocation.hpp"
#include "llhd/SourceManager.hpp"
#include <boost/test/unit_test.hpp>
using namespace llhd;

const char* src = "\
define @marx_tb {\n\
	in l1 %clk_ci\n\
	in l1 %rst_rbi\n\
";

BOOST_AUTO_TEST_CASE(asm_parser_case) {

	SourceManager mgr;
	DiagnosticContext diag;
	FileId f = mgr.addBuffer(SourceBuffer((const utf8char*)src), "temporary");
	SourceLocation loc = mgr.getStartLocation(f);

	Assembly as;
	AssemblyLexer lexer(loc, Buffer<const char>(src), &diag);
	AssemblyParser parser(as, lexer, &diag);
	BOOST_CHECK(parser);

	DiagnosticFormatterConsole fmt(std::cerr, mgr);
	fmt << diag;
	BOOST_CHECK(!diag.isErrorSeverity());
}
