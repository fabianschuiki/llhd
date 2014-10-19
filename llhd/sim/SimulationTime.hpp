/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {

class SimulationTime {
public:
	uint64_t ps;
	uint32_t delta;

	SimulationTime(): ps(0), delta(0) {}
	SimulationTime(uint64_t ps): ps(ps), delta(0) {}

	bool operator<(SimulationTime t) const {
		if (ps < t.ps) return true;
		if (ps > t.ps) return false;
		return delta < t.delta;
	}
	bool operator<=(SimulationTime t) const {
		if (ps < t.ps) return true;
		if (ps > t.ps) return false;
		return delta <= t.delta;
	}
	bool operator>(SimulationTime t) const { return !(*this <= t); }
	bool operator>=(SimulationTime t) const { return !(*this < t); }

	bool operator==(SimulationTime t) const {
		if (ps != t.ps) return false;
		return delta == t.delta;
	}
	bool operator!=(SimulationTime t) const { return !(*this == t); }
};

} // namespace llhd
