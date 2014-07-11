/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <ostream>

namespace llhd {

class Diagnostic;
class DiagnosticContext;
class SourceManager;

class DiagnosticFormatterConsole {
	std::ostream& output;
	SourceManager& manager;
	bool breakLinesToTerminalSize;

public:
	DiagnosticFormatterConsole(std::ostream& o, SourceManager& m):
		output(o),
		manager(m) {

		breakLinesToTerminalSize = true;
	}

	DiagnosticFormatterConsole& operator<<(const DiagnosticContext& diag);
	DiagnosticFormatterConsole& operator<<(const Diagnostic& diag);
};

} // namespace llhd
