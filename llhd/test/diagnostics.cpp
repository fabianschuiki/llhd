/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Diagnostic.hpp"
#include "llhd/DiagnosticContext.hpp"
#include "llhd/DiagnosticFormatterConsole.hpp"
#include "llhd/DiagnosticMessage.hpp"
#include <iostream>
using namespace llhd;

int main(int argc, char** argv) {

	DiagnosticContext ctx;
	Diagnostic* diag = ctx.alloc.one<Diagnostic>();

	DiagnosticMessage* msg = ctx.alloc.one<DiagnosticMessage>(
		kError,
		"component declaration 'arbiter' (1) disagrees with the corresponding entity (2)");
	msg->setArgument(0, "Liberty City");
	diag->addMessage(msg);

	msg = ctx.alloc.one<DiagnosticMessage>(
		kNote,
		"both declare the same port signals, however the order differs:\n"
		"(1) declares\n"
        "  - output_do\n"
        "  - error_so\n"
        "(2) declares\n"
        "  - error_so\n"
        "  - output_do");
	diag->addMessage(msg);

	msg = ctx.alloc.one<DiagnosticMessage>(
		kFixit,
		"assuming the entity declaration (2) is authorative:");
	diag->addMessage(msg);

	// Format the diagnostic to the console.
	DiagnosticFormatterConsole fmt(std::cout);
	fmt << diag;

	return 0;
}
