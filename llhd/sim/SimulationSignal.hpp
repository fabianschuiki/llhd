/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationValue.hpp"

namespace llhd {

class AssemblySignal;

class SimulationSignal {
public:
	AssemblySignal *as;
	SimulationValue value;
};

} // namespace llhd
