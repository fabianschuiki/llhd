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
	if (!*input || diactx.isFatal())
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
	if (*input && (*input)->type == kKeywordLibrary) {
		auto libraryKeyword = *input;
		input++;

		if (!*input) {
			addDiagnostic(
				libraryKeyword->range,
				kFatal,
				"library keyword must be followed by a comma-separated list "
				"of names")
			.end();
			return false;
		}

		if (((*input)->type & kTokenMask) != kTokenIdentifier) {
			addDiagnostic(
				(*input)->range,
				kFatal,
				"library keyword $0 must be followed by a name")
			.arg(libraryKeyword->range)
			.highlight(libraryKeyword->range)
			.end();
			return false;
		}
		std::cout << "library name " << (*input)->range << '\n';
		auto lastIdentifier = *input;
		input++;

		while (*input && (*input)->type == kTokenComma) {
			auto prev = *input++;
			if (!*input || ((*input)->type & kTokenMask) != kTokenIdentifier) {
				addDiagnostic(
					prev->range,
					kError,
					"gratuitous comma after library name")
				.highlight(lastIdentifier->range)
				.message(
					kFixit,
					"add another library name after the comma or remove it")
				.end();
				break;
			}

			std::cout << "library name " << (*input)->range << '\n';
			lastIdentifier = *input;
			input++;
		}

		if (!*input || (*input)->type != kTokenSemicolon) {
			addDiagnostic(
				lastIdentifier->range,
				kWarning,
				"semicolon missing after library name '$0'")
			.arg(lastIdentifier->range.s.getId())
			.message(kFixit, "insert a semicolon")
			.end();
			// \todo: Improve the fixit hint here. Don't add any message to it
			// as the required actions are clear from the diagnostic.
			return true;
		}
		input++;

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

