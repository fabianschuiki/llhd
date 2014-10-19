/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/sim/Simulation.hpp"
#include <iostream>
#include <stdexcept>
using namespace llhd;

Simulation::Simulation(const AssemblyModule& as):
	as(as) {

	// Initialize the signal wrappers.
	for (auto& is : as.signals) {
		const auto s = is.second.get();

		std::cout << "wrapping " << s->name << '\n';
		wrappers[s].reset(new SimulationSignal(s,
			getInitialValue(s->type.get())
		));
	}
}

SimulationValue Simulation::getInitialValue(const AssemblyType* type) {
	if (dynamic_cast<const AssemblyTypeLogic*>(type)) {
		return SimulationValue(1, kLogicU);
	}
	if (auto subtype = dynamic_cast<const AssemblyTypeWord*>(type)) {
		if (dynamic_cast<const AssemblyTypeLogic*>(subtype->type.get())) {
			return SimulationValue(subtype->width, kLogicU);
		}
	}
	throw std::runtime_error("unknown type");
}

bool Simulation::observe(const AssemblySignal* signal) {
	auto it = wrappers.find(signal);
	if (it == wrappers.end())
		return false;

	observedSignals.insert(it->second.get());
	return true;
}

void Simulation::dump(ObserverFunc fn) {
	for (auto& is : wrappers) {
		const auto& s = *is.second;
		fn(s.as, s.value);
	}
}

void Simulation::addEvent(
	SimulationTime T,
	const AssemblySignal* signal,
	const SimulationValue& value) {

	auto it = wrappers.find(signal);
	if (it == wrappers.end())
		return;

	eventQueue.addEvent(SimulationEvent(T, it->second.get(), value));
}

void Simulation::step(ObserverFunc fn) {
	if (eventQueue.isAtEnd())
		return;
	T = eventQueue.nextTime();
	eventQueue.nextEvents([&](const SimulationEvent& ev){
		ev.signal->value = ev.value;
		fn(ev.signal->as, ev.value);
	});
	eventQueue.pop();
}
