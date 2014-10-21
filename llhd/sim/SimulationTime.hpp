/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {

/// Class that represents one point in time during simulation. It consists of a
/// time value that captures the passing of time, and an imaginary delta value
/// that captures the iterations required per simulation step for the signals to
/// settle to their intended value.
class SimulationTime {
public:
	/// The real time value. The unit is specified by the simulation. Usually
	/// ns or ps.
	uint64_t value;
	/// The imaginary delta time value. Unitless. Represent infinitesimally
	/// small steps in time.
 	uint32_t delta;

 	/// Creates time zero.
	SimulationTime(): value(0), delta(0) {}
	/// Creates a time with the given real value, and the delta value set to 0.
	SimulationTime(uint64_t value): value(value), delta(0) {}
	/// Creates a time with the given real and delta values.
	SimulationTime(uint64_t value, uint64_t d): value(value), delta(d) {}

	bool operator<(SimulationTime t) const {
		if (value < t.value) return true;
		if (value > t.value) return false;
		return delta < t.delta;
	}
	bool operator<=(SimulationTime t) const {
		if (value < t.value) return true;
		if (value > t.value) return false;
		return delta <= t.delta;
	}
	bool operator>(SimulationTime t) const { return !(*this <= t); }
	bool operator>=(SimulationTime t) const { return !(*this < t); }

	bool operator==(SimulationTime t) const {
		if (value != t.value) return false;
		return delta == t.delta;
	}
	bool operator!=(SimulationTime t) const { return !(*this == t); }

	/// Returns a SimulationTime with the delta incremented by the given amount.
	SimulationTime advDelta(unsigned d = 1) const {
		return SimulationTime(value, delta+d);
	}

	/// Returns a SimulationTime advanced into the future by the given amount of
	/// time. Resets the delta time back to zero.
	SimulationTime advTime(unsigned t) const {
		return SimulationTime(value+t);
	}
};

} // namespace llhd
