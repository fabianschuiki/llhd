/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationDependency.hpp"
#include "llhd/sim/SimulationValue.hpp"
#include <set>

namespace llhd {

class AssemblySignal;

/// A single signal being simulated. Wraps an AssemblySignal and keeps track of
/// the signal's dependencies such that updates may be triggered efficiently
/// during simulation steps.
class SimulationSignal {
	/// Set of objects that depend on the value of this signal and need to be
	/// updated when it changes.
	std::set<SimulationDependency*> dependencies;
	/// The underlying AssemblySignal.
	const AssemblySignal * const as;
	/// The signal's current value.
	SimulationValue value;

public:
	/// Creates a new signal that wraps around the given AssemblySignal \a as
	/// and has initial value \a v.
	SimulationSignal(const AssemblySignal *as, SimulationValue v):
		as(as),
		value(v) {}

	/// Returns the signal's current value.
	const SimulationValue& getValue() const { return value; }
	/// Sets the signal's current value to \a v.
	void setValue(const SimulationValue& v) { value = v; }
	/// Returns the underlying AssemblySignal.
	const AssemblySignal* getAssemblySignal() const { return as; }

	/// Adds the dependency \a d to the signal. Returns true if \a d was not yet
	/// listed as a dependency.
	bool addDependency(SimulationDependency *d) {
		return dependencies.insert(d).second;
	}

	/// Removes the dependency \a d from the signal. Returns true if \a d was
	/// found and removed, false otherwise.
	bool removeDependency(SimulationDependency *d) {
		return dependencies.erase(d);
	}

	void eachDependency(std::function<void(SimulationDependency*)> fn) {
		for (auto d : dependencies)
			fn(d);
	}
};

} // namespace llhd
