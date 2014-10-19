/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationSignal.hpp"
#include "llhd/sim/SimulationTime.hpp"
#include "llhd/sim/SimulationValue.hpp"

namespace llhd {

class SimulationEvent {
public:
	SimulationTime T;
	SimulationSignal *signal;
	SimulationValue value;

	SimulationEvent() {}

	SimulationEvent(
		SimulationTime T,
		SimulationSignal* signal,
		SimulationValue&& value):
		T(T),
		signal(signal),
		value(std::move(value)) {}

	SimulationEvent(
		SimulationTime T,
		SimulationSignal* signal,
		const SimulationValue& value):
		T(T),
		signal(signal),
		value(value) {}
};

class SimulationEventQueue {
	std::map<SimulationTime, std::vector<SimulationEvent>> events;

public:
	typedef std::function<void(const SimulationEvent&)> EventFunc;

	void addEvent(SimulationEvent&& ev) {
		events[ev.T].push_back(std::move(ev));
	}

	void addEvent(const SimulationEvent& ev) {
		events[ev.T].push_back(ev);
	}

	bool isAtEnd() const { return events.empty(); }

	SimulationTime nextTime() const {
		if (events.empty())
			return SimulationTime();
		return events.begin()->first;
	}

	void nextEvents(EventFunc fn) const {
		if (!events.empty()) {
			for (const auto& ev : events.begin()->second)
				fn(ev);
		}
	}

	void pop() {
		if (!events.empty()) {
			events.erase(events.begin());
		}
	}
};

} // namespace llhd
