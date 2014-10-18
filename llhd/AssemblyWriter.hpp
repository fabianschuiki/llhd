/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/Assembly.hpp"
#include <ostream>

namespace llhd {

/// Maps an in-memory Assembly to its textual equivalent.
class AssemblyWriter {
	std::ostream& out;

	void write(const AssemblySignal& in);
	void write(const AssemblyType& in);

public:
	AssemblyWriter(std::ostream& out): out(out) {}

	AssemblyWriter& write(const Assembly& in);
	AssemblyWriter& write(const AssemblyModule& in);
};

} // namespace llhd
