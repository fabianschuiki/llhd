/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <ostream>

namespace llhd {

class Diagnostic;
class SourceManager;

class DiagnosticFormatterConsole {
	std::ostream& output;
	SourceManager& manager;

public:
	DiagnosticFormatterConsole(std::ostream& o, SourceManager& m):
		output(o),
		manager(m) {}
	DiagnosticFormatterConsole& operator<<(const Diagnostic* diag);
};

} // namespace llhd
