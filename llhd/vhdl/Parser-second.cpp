/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/vhdl/Parser-private.hpp"

/// IEEE 1076-2000 §11.1
/// design_file : design_unit {design_unit}
bool Parser::parseSecondStage(
	Token**& start,
	Token** end,
	TokenGroup& into) {

	TokenScanner scn(start, end);

	auto before = scn.getCurrent();
	while (requireDesignUnit(scn)) {
		assert(scn.getCurrent() > before && "parse loop did not progress");
		before = scn.getCurrent();
	}

	assert(scn.isAtEnd() && "did not consume all input tokens");

	// while (start != end) {
	// 	auto before = start;

	// 	switch ((*start)->type) {
	// 	}
	// 	start++;

	// 	// Sentinel that prevents infinite loops.
	// 	assert(start > before && "parse loop did not progress");
	// }
	// return false;

	return true;
}

/// IEEE 1076-2000 §11.1
/// design_unit : context_clause library_unit
bool Parser::requireDesignUnit(TokenScanner& input) {
	if (!requireContextClause(input)) return false;
	if (!requireLibraryUnit(input)) return false;
	return true;
}

/// IEEE 1076-2000 §11.1
/// library_unit : primary_unit | secondary_unit
bool Parser::requireLibraryUnit(TokenScanner& input) {
	return false;
}

/// IEEE 1076-2000 §11.2
/// library_clause : "library" logical_name_list ";"
bool Parser::acceptLibraryClause(TokenScanner& input) {
	auto scn = input.branch();
	if (!scn.accept(kKeywordLibrary))
		return false;
	while (!scn.find(kTokenSemicolon));
	auto namescn = scn.slice(1,1);
	// requireLogicalNameList(namescn);
	std::cout << "parsed library clause " << namescn.getRange() << "\n";

	scn.commit();
	return true;
}

/// IEEE 1076-2000 §11.3
/// context_clause : {context_item}
bool Parser::requireContextClause(TokenScanner& input) {
	while (acceptContextItem(input));
	return true;
}

/// IEEE 1076-2000 §11.3
/// context_item : library_clause | use_clause
bool Parser::acceptContextItem(TokenScanner& input) {
	return acceptLibraryClause(input) || acceptUseClause(input);
}

/// IEEE 1076-2000 §10.4
/// use_clause : "use" selected_name {"," selected_name} ";"
bool Parser::acceptUseClause(TokenScanner& input) {
	auto scn = input.branch();
	if (!scn.accept(kKeywordUse))
		return false;

	// Find the terminating semicolon.
	while (!scn.find(kTokenSemicolon));
	auto namescn = scn.slice(1,1);

	// Parse the names.
	parseSelectedName(namescn, true);
	while (namescn.accept(kTokenComma))
		parseSelectedName(namescn, true);

	scn.commit();
	return true;
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
