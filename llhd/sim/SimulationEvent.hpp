/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationSignal.hpp"
#include "llhd/sim/SimulationTime.hpp"
#include "llhd/sim/SimulationValue.hpp"

namespace llhd {

/// A single event in the simulation's event queue. Describes the transition of
/// a signal from one value to another at a specific point in time. The queue is
/// organized into batches of events. Each batch corresponds to one specific
/// point in time and contains an ordered list of events for that time. If a
/// batch's events are processed in the order that they appear in the list, the
/// queue forms a fully deterministic execution engine.
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

	void clear() {
		signal = nullptr;
	}

	operator bool() const { return signal != nullptr; }
};

/// Queue of pending events to be simulated. Each event describes the changing
/// of a signal. As a reaction to the change, more events are spawned to
/// describe the changes to dependent signals.
class SimulationEventQueue {
	std::map<SimulationTime, std::vector<SimulationEvent>> events;

public:
	typedef std::function<void(const SimulationEvent&)> EventFunc;

	/// Adds an event for the given time \a T to the queue. Refer to the
	/// constructors of SimulationEvent for possible arguments \a args.
	template<typename... Args>
	void addEvent(SimulationTime T, Args&&... args) {
		events[T].emplace_back(T, args...);
	}

	/// Returns true if the event queue is empty and thus no more events need
	/// to be processed, effectively identifying the end of the simulation.
	bool isAtEnd() const { return events.empty(); }

	/// Returns the simulation time of the next batch of events pending. May be
	/// used to predict to what point in time the simulation jumps when
	/// processing the next events.
	SimulationTime nextTime() const {
		if (events.empty())
			return SimulationTime();
		return events.begin()->first;
	}

	/// Calls \a fn on each event in the next batch.
	void nextEvents(EventFunc fn) const {
		if (!events.empty()) {
			for (const auto& ev : events.begin()->second) {
				if (ev) fn(ev);
			}
		}
	}

	/// Pops the next batch of events off the queue, effectively advancing the
	/// queue in time.
	void pop() {
		if (!events.empty()) {
			events.erase(events.begin());
		}
	}
};

} // namespace llhd
