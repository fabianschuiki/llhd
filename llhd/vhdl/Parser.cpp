/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Token.hpp"
#include "llhd/TokenBuffer.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenType.hpp"
#include <iostream>

using namespace llhd::vhdl;

/// IEEE 1076-2000 §11.1
/// design_file : design_unit { design_unit }
void Parser::parse(const TokenBuffer& input) {
	Iterator t(input.getStart(), input.getEnd());
	while (*t && !diactx.isFatal()) {
		parseDesignUnit(t);
	}
}

/// IEEE 1076-2000 §11.1, §11.3
/// design_unit    : context_clause library_unit
/// context_clause : { context_item }
/// context_item   : library_clause
///                | use_clause
/// library_unit   : primary_unit
///                | secondary_unit
/// primary_unit   : entity_declaration
///                | configuration_declaration
///                | package_declaration
/// secondary_unit : architecture_body
///                | package_body
void Parser::parseDesignUnit(Iterator& input) {
	while (!diactx.isFatal() && (
		acceptLibraryClause(input) ||
		acceptUseClause(input)));
	if (diactx.isFatal())
		return;

	if (acceptEntityDeclaration(input) ||
	    acceptConfigurationDeclaration(input) ||
	    acceptPackageDeclaration(input) ||
	    acceptArchitectureBody(input) ||
	    acceptPackageBody(input)) {

		std::cout << "read design unit\n";
	} else {
		addDiagnostic(
			(*input)->range,
			kFatal,
			"expected entity declaration, configuration declaration, package "
			"declaration, architecture body, or package body").end();

		input++;
	}
}

/// IEEE 1076-2000 §11.2
/// library_clause    : "library" logical_name_list ";"
/// logical_name_list : logical_name { "," logical_name }
/// logical_name      : identifier
bool Parser::acceptLibraryClause(Iterator& input) {
	if ((*input)->type == kKeywordLibrary) {
		auto libraryKeyword = *input;
		input++;
		if (!*input || true) {
			addDiagnostic(
				libraryKeyword->range,
				kFatal,
				"expected name after library keyword").end();
			return false;
		}
		// if ((*input)->type == kTokenSemicolon)
		return true;
	}
	return false;
}

bool Parser::acceptUseClause(Iterator& input) {
	return false;
}

bool Parser::acceptEntityDeclaration(Iterator& input) {
	return false;
}

bool Parser::acceptConfigurationDeclaration(Iterator& input) {
	return false;
}

bool Parser::acceptPackageDeclaration(Iterator& input) {
	return false;
}

bool Parser::acceptArchitectureBody(Iterator& input) {
	return false;
}

bool Parser::acceptPackageBody(Iterator& input) {
	return false;
}

