/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {

class SimulationTime {
public:
	uint64_t ps;
	uint32_t delta;

	SimulationTime(): ps(0), delta(0) {}
};

} // namespace llhd
