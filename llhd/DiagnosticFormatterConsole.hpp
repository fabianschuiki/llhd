/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <ostream>

namespace llhd {

class Diagnostic;

class DiagnosticFormatterConsole {
	std::ostream& output;

public:
	DiagnosticFormatterConsole(std::ostream& o): output(o) {}
	DiagnosticFormatterConsole& operator<<(const Diagnostic* diag);
};

} // namespace llhd
