/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Token.hpp"
#include "llhd/TokenBuffer.hpp"
#include "llhd/diagnostic/Diagnostic.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticMessage.hpp"
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

	// Diagnostic* diag = diactx.alloc.one<Diagnostic>();
	// DiagnosticMessage* msg = diactx.alloc.one<DiagnosticMessage>(
	// 	kFatal,
	// 	"trying to parse $0, but nobody told me how");
	// msg->addArgument(input.getStart()[0]->range);
	// msg->setMainRange(input.getStart()[0]->range);
	// diag->addMessage(msg);
	// diactx.addDiagnostic(diag);
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
		auto dia = diactx.alloc.one<Diagnostic>();
		auto msg = diactx.alloc.one<DiagnosticMessage>(kFatal,
			"expected entity declaration, configuration declaration, package "
			"declaration, architecture body, or package body");
		msg->setMainRange((*input)->range);
		dia->addMessage(msg);
		diactx.addDiagnostic(dia);
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
			auto dia = diactx.alloc.one<Diagnostic>();
			auto msg = diactx.alloc.one<DiagnosticMessage>(kFatal,
				"expected name after library keyword $0");
			msg->addArgument(libraryKeyword->range);
			msg->setMainRange(libraryKeyword->range);
			dia->addMessage(msg);
			diactx.addDiagnostic(dia);
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

