/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationTime.hpp"
#include "llhd/sim/SimulationEvent.hpp"
#include "llhd/sim/SimulationSignal.hpp"
#include <map>
#include <set>
#include <vector>

namespace llhd {

class AssemblyModule;
class AssemblySignal;
class AssemblyType;

class SimulationDependency;

/// A simple implementation of an LLHD assembly simulator. Wraps a single
/// AssemblyModule into a simulation structure that may then be examined
/// step-by-step. A proof of concept.
class Simulation {
public:
	typedef std::function<void(
		SimulationTime T,
		const AssemblySignal*,
		const SimulationValue&)> ObserverFunc;

private:
	const AssemblyModule& as;
	SimulationTime T;
	SimulationEventQueue eventQueue;
	std::map<const AssemblySignal*, std::unique_ptr<SimulationSignal>> wrappers;
	std::set<std::unique_ptr<SimulationDependency>> dependencies;

	void wrap(const AssemblySignal& signal);
	SimulationValue wrap(const AssemblyType *type);
	void wrap(const AssemblyIns& ins);

public:
	Simulation(const AssemblyModule& as);
	void eachSignal(ObserverFunc fn);
	void addEvent(
		SimulationTime T,
		const AssemblySignal* signal,
		const SimulationValue& value);

	bool isAtEnd() const { return eventQueue.isAtEnd(); }
	SimulationTime getTime() const { return T; }
	void step(ObserverFunc fn);
};

} // namespace llhd
