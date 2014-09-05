/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Token.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenGroup.hpp"
#include "llhd/vhdl/TokenType.hpp"
#include <iostream>

using namespace llhd::vhdl;

bool Parser::accept(Iterator& input, unsigned type, Token*& token) {
	if (*input && (*input)->type == type) {
		token = *input;
		input++;
		return true;
	}
	return false;
}

bool Parser::accept(Iterator& input, unsigned type) {
	Token* ignored;
	return accept(input, type, ignored);
}

bool Parser::acceptIdentifier(Iterator& input, Token*& token) {
	if (*input && ((*input)->type & kTokenMask) == kTokenIdentifier) {
		token = *input;
		input++;
		return true;
	}
	return false;
}

/// Parses the \a input tokens into a VHDL abstract syntax tree.
void Parser::parse(const TokenBuffer& input) {

	// Run the first parse stage.
	Token** start = input.getStart();
	Token** end = input.getEnd();
	if (start == end)
		return;
	TokenGroup first(kTokenInvalid);
	first.range.s = (*start)->range.s;
	parseFirstStage(start, end, first);

	// Run the second parse stage.
	TokenBuffer tb = first.getBuffer();
	start = tb.getStart();
	end = tb.getEnd();
	if (start == end)
		return;
	TokenGroup second(kTokenInvalid);
	second.range.s = (*start)->range.s;
	parseSecondStage(start, end, second);
}

/// IEEE 1076-2000 §11.1
/// design_file : design_unit { design_unit }
/// \todo implement this!

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
	// while (!diag.isFatalSeverity() && (
	// 	acceptLibraryClause(input) ||
	// 	acceptUseClause(input)));
	// if (!*input || diag.isFatalSeverity())
	// 	return;

	// if (acceptEntityDeclaration(input) ||
	//     acceptConfigurationDeclaration(input) ||
	//     acceptPackageDeclaration(input) ||
	//     acceptArchitectureBody(input) ||
	//     acceptPackageBody(input)) {

	// 	std::cout << "read design unit\n";
	// } else {
	// 	addDiagnostic(
	// 		(*input)->range,
	// 		kFatal,
	// 		"expected entity declaration, configuration declaration, package "
	// 		"declaration, architecture body, or package body").end();

	// 	input++;
	// }
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

/// IEEE 1076-2000 §10.4
/// use_clause : "use" selected_name { "," selected_name } ";"
// bool Parser::acceptUseClause(Iterator& input) {
// 	Token* keyword = nullptr;
// 	if (accept(input, kKeywordUse, keyword)) {
// 		auto lastToken = keyword;
// 		if (!*input)
// 			goto premature;
// 		lastToken = *input;

// 		if (!acceptSelectedName(input)) {
// 			addDiagnostic(
// 				(*input)->range, kFatal,
// 				"'use' keyword must be followed by one or more selected names")
// 			.highlight(keyword->range);
// 			return false;
// 		}

// 		while (*input && accept(input, kTokenComma)) {
// 			if (!*input)
// 				goto premature;
// 			lastToken = *input;

// 			if (!acceptSelectedName(input)) {
// 				addDiagnostic(
// 					(*input)->range, kFatal,
// 					"expected a name inside 'use' clause")
// 				.highlight(keyword->range);
// 				return false;
// 			}
// 			lastToken = *input;
// 		}

// 		if (!accept(input, kTokenSemicolon)) {
// 			addDiagnostic(
// 				*input ? (*input)->range : lastToken->range,
// 				kWarning,
// 				"semicolon missing at the end of 'use' clause")
// 			.highlight(keyword->range)
// 			.message(kFixit, "insert a semicolon");
// 			/// \todo: Improve fixit hint.
// 			return true;
// 		}

// 		return true;

// 	premature:
// 		addDiagnostic(lastToken->range, kFatal,
// 			"incomplete 'use' clause")
// 		.message(kNote, "must be of the form 'use' <name>[, <name>];");
// 		return false;
// 	}
// 	return false;
// }

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

/// IEEE 1076-2000 §6.2, §6.3
/// selected_name : prefix "." suffix
/// suffix        : simple_name
///               | character_literal
///               | operator_symbol
///               | all
/// simple_name   : identifier
bool Parser::acceptSelectedName(Iterator& input) {
	Token* ignored;
	return acceptIdentifier(input, ignored);
	return false;
}
