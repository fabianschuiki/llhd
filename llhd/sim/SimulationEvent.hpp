/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

namespace llhd {

class SimulationEvent {
public:
	SimulationTime T;
	AssemblySignal *signal;
	SimulationValue value;
};

class SimulationEventQueue {
	std::map<SimulationTime, std::vector<SimulationEvent*>> events;
	std::map<AssemblySignal*, std::

public:
	typedef std::function<void(const SimulationEvent&)> SimulationEventHandler;

	void addEvent(
		SimulationTime T,
		AssemblySignal *signal,
		SimulationValue* value);

	void processUntil(
		SimulationTime T,
		SimulationEventHandler fn);
};

} // namespace llhd
