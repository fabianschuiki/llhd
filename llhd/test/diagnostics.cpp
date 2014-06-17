/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Diagnostic.hpp"
#include "llhd/DiagnosticContext.hpp"
#include "llhd/DiagnosticFormatterConsole.hpp"
#include "llhd/DiagnosticMessage.hpp"
#include "llhd/SourceManager.hpp"
#include <iostream>
using namespace llhd;

static const char* system_vhd =
"/* system.vhd */\n"
"component arbiter is\n"
"	port(\n"
"		output_do : out std_logic;\n"
"		error_so  : out std_logic);\n"
"end component arbiter;\n";

static const char* arbiter_vhd =
"/* arbiter.vhd */\n"
"entity arbiter is\n"
"	port(\n"
"		error_so  : out std_logic;\n"
"		output_do : out std_logic);\n"
"end entity arbiter;\n";

int main(int argc, char** argv) {

	// Create a source manager to provide some in-memory source code.
	SourceManager manager;
	FileId system_vhd_id = manager.addBuffer(
		SourceBuffer((const utf8char*)system_vhd),
		"system.vhd");
	FileId arbiter_vhd_id = manager.addBuffer(
		SourceBuffer((const utf8char*)arbiter_vhd),
		"arbiter.vhd");

	SourceLocation system_start = manager.getStartLocation(system_vhd_id);
	SourceLocation arbiter_start = manager.getStartLocation(arbiter_vhd_id);

	system_start += 30;
	arbiter_start += 30;

	PresumedLocation system_pl = manager.getPresumedLocation(system_start);
	PresumedLocation arbiter_pl = manager.getPresumedLocation(arbiter_start);

	std::cout << system_pl.filename << ":" << system_pl.line << "." << system_pl.column << '\n';
	std::cout << arbiter_pl.filename << ":" << arbiter_pl.line << "." << arbiter_pl.column << '\n';

	// Create a diagnostic context to be filled with messages.
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
	DiagnosticFormatterConsole fmt(std::cout, manager);
	fmt << diag;

	return 0;
}
