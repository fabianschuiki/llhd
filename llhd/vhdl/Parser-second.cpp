/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenGroup.hpp"
#include "llhd/vhdl/TokenType.hpp"

using namespace llhd::vhdl;

bool Parser::parseSecondStage(
	Token**& start,
	Token** end,
	TokenGroup& into) {

	while (start != end) {
		auto before = start;

		switch ((*start)->type) {
		}
		start++;

		// Sentinel that prevents infinite loops.
		assert(start > before && "parse loop did not progress");
	}
	return false;
}

// IEEE 1076-2000 §11.1
// design_file    : design_unit {design_unit}
// design_unit    : context_clause library_unit
// library_unit   : primary_unit
//                | secondary_unit
// primary_unit   : entity_declaration
//                | configuration_declaration
//                | package_declaration
// secondary_unit : architecture_body
//                | package_body
//
// IEEE 1076-2000 §1.1
// entity_declaration :
//     "entity" identifier "is"
//         entity_header
//         entity_declarative_part
//     ["begin" entity_statement_part]
//     "end" ["entity"] [simple_name] ";"
//
// IEEE 1076-2000 §1.3
// configuration_declaration :
//     "configuration" identifier "of" name "is"
//         configuration_declarative_part
//         block_configuration
//     "end" ["configuration"] [simple_name] ";"
//
// IEEE 1076-2000 §2.5
// package_declaration :
//     "package" identifier "is"
//         package_declarative_part
//     "end" ["package"] [simple_name] ";"
//
// IEEE 1076-2000 §1.2
// architecture_body :
//     "architecture" identifier "of" name "is"
//         architecture_declarative_part
//     "begin"
//         architecture_statement_part
//     "end" ["architecture"] [simple_name] ";"
//
// IEEE 1076-2000 §2.6
// package_body :
//     "package" "body" simple_name "is"
//         package_body_declarative_part
//     "end" ["package" "body"] [simple_name] ";"
