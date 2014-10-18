/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationTime.hpp"
// #include "llhd/sim/SimulationEvent.hpp"
#include "llhd/sim/SimulationSignal.hpp"

namespace llhd {

class AssemblyModule;

class Simulation {
	const AssemblyModule& as;
	SimulationTime T;
	// SimulationEventQueue eventQueue;

public:
	Simulation(const AssemblyModule& as);
};

} // namespace llhd
