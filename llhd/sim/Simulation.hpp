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

class Simulation {
public:
	typedef std::function<
		void(const AssemblySignal*, const SimulationValue& v)> ObserverFunc;

private:
	const AssemblyModule& as;
	SimulationTime T;
	SimulationEventQueue eventQueue;
	std::map<const AssemblySignal*, std::unique_ptr<SimulationSignal>> wrappers;
	std::set<SimulationSignal*> observedSignals;

	static SimulationValue getInitialValue(const AssemblyType* type);

public:
	Simulation(const AssemblyModule& as);
	bool observe(const AssemblySignal* signal);
	void dump(ObserverFunc fn);
	void addEvent(
		SimulationTime T,
		const AssemblySignal* signal,
		const SimulationValue& value);

	bool isAtEnd() const { return eventQueue.isAtEnd(); }
	SimulationTime getTime() const { return T; }
	void step(ObserverFunc fn);
};

} // namespace llhd
