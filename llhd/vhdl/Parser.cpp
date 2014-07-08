/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Token.hpp"
#include "llhd/TokenBuffer.hpp"
#include "llhd/diagnostic/Diagnostic.hpp"
#include "llhd/diagnostic/DiagnosticContext.hpp"
#include "llhd/diagnostic/DiagnosticMessage.hpp"
#include "llhd/vhdl/Parser.hpp"
#include <iostream>

using namespace llhd::vhdl;

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

/// design_unit    : context_clause library_unit
/// context_clause : { context_item }
/// library_unit   : primary_unit
///                | secondary_unit
/// primary_unit   : entity_declaration
///                | configuration_declaration
///                | package_declaration
/// secondary_unit : architecture_body
///                | package_body
void Parser::parseDesignUnit(Iterator& input) {
	while (!diactx.isFatal() && acceptContextItem(input));

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

bool Parser::acceptContextItem(Iterator& input) {
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

