/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Diagnostic.hpp"
#include "llhd/DiagnosticFormatterConsole.hpp"
#include "llhd/DiagnosticMessage.hpp"
using namespace llhd;


DiagnosticFormatterConsole& DiagnosticFormatterConsole::operator<<(
	const Diagnostic* diag) {

	for (unsigned i = 0; i < diag->getNumMessages(); i++) {
		const DiagnosticMessage* msg = diag->getMessage(i);

		switch (msg->getType()) {
			case kFatal: output << "\033[31;1mfatal error:\033[0m"; break;
			case kError: output << "\033[31merror:\033[0m"; break;
			case kWarning: output << "\033[33;1mwarning:\033[0m"; break;
			case kNote: output << "\033[1mnote:\033[0m"; break;
			case kFixit: output << "\033[36;1mfixit:\033[0m"; break;
			default: output << "unspecified:"; break;
		}
		output << " ";
		if (i == 0) output << "\033[1m";
		output << msg->getMessage();
		if (i == 0) output << "\033[0m";
		output << '\n';

		// output << "- message " << msg->getMessage() << '\n';
		output << '\n';
	}

	return *this;
}
