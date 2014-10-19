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
	std::set<const AssemblySignal*> changed;
	eventQueue.nextEvents([&](const SimulationEvent& ev){
		if (ev.signal->value == ev.value)
			return;
		ev.signal->value = ev.value;
		fn(ev.signal->as, ev.value);
		changed.insert(ev.signal->as);
	});
	eventQueue.pop();

	SimulationTime Tn = T;
	Tn.delta++;

	// Re-evaluate everything that depends on the changed signals.
	for (auto& is : wrappers) {
		auto s = is.second.get();
		if (s->as->assignment) {
			auto ab = s->as->assignment.get();
			if (auto a = dynamic_cast<const AssemblyExprIdentity*>(ab)) {
				if (changed.count(a->op)) {
					eventQueue.addEvent(SimulationEvent(
						Tn, s, wrappers[a->op]->value));
				}
			}
			else if (auto a = dynamic_cast<const AssemblyExprDelayed*>(ab)) {
				if (changed.count(a->op)) {
					SimulationTime Td = Tn;
					if (a->d > 0) {
						Td.ps += a->d;
						Td.delta = 0;
					}
					eventQueue.addEvent(SimulationEvent(
						Td, s, wrappers[a->op]->value));
				}
			}
			else if (auto a = dynamic_cast<const AssemblyExprBoolean*>(ab)) {
				if (changed.count(a->op0) || changed.count(a->op1)) {
					auto v0 = wrappers[a->op0]->value;
					auto v1 = wrappers[a->op1]->value;
					if (v0.width != v1.width)
						throw std::runtime_error(
							"boolean operator widths don't match");
					SimulationValue r(v0.width, kLogicU);
					SimulationLogicValue *b0 = v0.bits, *be = v0.bits+v0.width,
						*b1 = v1.bits, *br = r.bits;
					for (; b0 != be; b0++, b1++, br++) {
						if ((*b0 == kLogic0 || *b0 == kLogic1) &&
							(*b1 == kLogic0 || *b1 == kLogic1)) {
							switch (a->type) {
								case AssemblyExprBoolean::kAND:
									*br = kLogic0;
									if (*b0 == kLogic1 && *b1 == kLogic1)
										*br = kLogic1;
									break;
								case AssemblyExprBoolean::kNAND:
									*br = kLogic1;
									if (*b0 == kLogic1 && *b1 == kLogic1)
										*br = kLogic0;
									break;
								case AssemblyExprBoolean::kOR:
									*br = kLogic0;
									if (*b0 == kLogic1 || *b1 == kLogic1)
										*br = kLogic1;
									break;
								case AssemblyExprBoolean::kNOR:
									*br = kLogic1;
									if (*b0 == kLogic1 || *b1 == kLogic1)
										*br = kLogic0;
									break;
								case AssemblyExprBoolean::kXOR:
									*br = kLogic0;
									if (*b0 != *b1)
										*br = kLogic1;
									break;
							}
						}
					}
					eventQueue.addEvent(SimulationEvent(Tn, s, r));
				}
			}
			else {
				throw std::runtime_error("unknown expression");
			}
		}
	}
}
