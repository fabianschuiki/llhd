/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationDependency.hpp"
#include "llhd/sim/SimulationValue.hpp"
#include <set>

namespace llhd {

class AssemblySignal;

class SimulationSignal {
	std::set<SimulationDependency*> dependencies;

public:
	const AssemblySignal * const as;
	SimulationValue value;

	SimulationSignal(const AssemblySignal *as, SimulationValue v):
		as(as),
		value(v) {}

	bool addDependency(SimulationDependency *d) {
		return dependencies.insert(d).second;
	}
};

} // namespace llhd
