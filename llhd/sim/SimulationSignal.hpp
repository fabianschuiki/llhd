/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationValue.hpp"

namespace llhd {

class AssemblySignal;

class SimulationSignal {
public:
	const AssemblySignal * const as;
	SimulationValue value;

	SimulationSignal(const AssemblySignal *as, SimulationValue v):
		as(as),
		value(v) {}
};

} // namespace llhd
