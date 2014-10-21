/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationTime.hpp"

namespace llhd {

class SimulationEventQueue;

/// Base class for all simulation objects that need to be updated in response to
/// some input changing value. This applies to simple instructions which
/// calculate a signal's value from a few input values.
class SimulationDependency {
	bool outdated;

public:
	SimulationDependency(): outdated(false) {}

	/// Marks the dependency as being outdated. Returns true if it was not
	/// marked outdated before, false otherwise. The return value is useful if
	/// the caller desires to build a vector of outdated dependencies.
	bool markOutdated() {
		if (outdated)
			return false;
		outdated = true;
		return true;
	}

	/// Marks the dependecy as up-to-date.
	void clearOutdated() { outdated = false; }
	/// Returns whether the dependency is outdated.
	bool isOutdated() const { return outdated; }

	/// Implemented by subclasses to perform the necessary updates in reaction
	/// to a change in its dependencies. Called from the simulation's step()
	/// function. Subclasses are expected to spawn new events in the queue that
	/// reflect the updated values.
	virtual void update(SimulationTime T, SimulationEventQueue& queue) = 0;
};

} // namespace llhd
