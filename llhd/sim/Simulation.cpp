/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/sim/Simulation.hpp"
#include "llhd/sim/SimulationExpr.hpp"
#include <cassert>
#include <iostream>
#include <stdexcept>
using namespace llhd;

Simulation::Simulation(const AssemblyModule& as):
	as(as) {

	// Initialize the signal wrappers.
	// for (auto& is : as.signals) {
	// 	wrap(is.second.get());
	// }
	as.eachSignal([&](const AssemblySignal& sig){
		wrap(sig);
	});

	// Initialize the instructions.
	as.eachInstruction([&](const AssemblyIns& ins){
		wrap(ins);
	});
}

/// Wraps the given signal in a structure suitable for simulation. Called
/// internally when the module to be simulated is prepared.
void Simulation::wrap(const AssemblySignal& sig) {

	// Wrap the signal in a simulation-specific structure.
	std::unique_ptr<SimulationSignal> w(new SimulationSignal(
		&sig,
		wrap(sig.getType().get())));

	// Add the signal to the list of wrappers.
	wrappers[&sig] = std::move(w);
}

/// Wraps the given type in a structure suitable for simulation.
SimulationValue Simulation::wrap(const AssemblyType *type) {
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

/// Wraps the given expression in a structure that implements the corresponding
/// operation and keeps track of the input and output signals.
void Simulation::wrap(const AssemblyIns& ins) {

	SimulationDependency *w = nullptr;

	switch (ins.getOpcode() & AssemblyIns::kOpMask) {

		// unary operations
		case AssemblyIns::kUnaryOps: {
			auto uins = *(const AssemblyUnaryIns*)&ins;
			auto result = wrappers[uins.getResult()].get();
			auto arg0 = wrappers[uins.getArg()].get();
			assert(arg0);

			switch (ins.getOpcode()) {
				case AssemblyIns::kMove: {
					if (uins.getDelay() == 0) {
						auto it = dependencies.emplace(
							new SimulationIdentityExpr(result, arg0));
						w = it.first->get();
						arg0->addDependency(w);
					} else {
						auto it = dependencies.emplace(new SimulationDelayExpr(
							result, arg0, uins.getDelay()));
						w = it.first->get();
						arg0->addDependency(w);
					}
				} break;

				case AssemblyIns::kBoolNOT: {
					auto it = dependencies.emplace(
						new SimulationBooleanUnaryIns(result, arg0,
							SimulationBooleanUnaryIns::fNOT));
					w = it.first->get();
					arg0->addDependency(w);
				} break;

				case AssemblyIns::kEdge:
				case AssemblyIns::kRisingEdge:
				case AssemblyIns::kFallingEdge: {
					auto it = dependencies.emplace(new SimulationEdgeIns(
						result, arg0,
						ins.getOpcode() != AssemblyIns::kFallingEdge,
						ins.getOpcode() != AssemblyIns::kRisingEdge));
					w = it.first->get();
					arg0->addDependency(w);
				} break;

				default:
					throw std::runtime_error("unknown unary opcode");
			}
		} break;

		// binary operations
		case AssemblyIns::kBinaryOps: {
			auto bins = *(const AssemblyBinaryIns*)&ins;
			auto result = wrappers[bins.getResult()].get();
			auto arg0 = wrappers[bins.getArg0()].get();
			auto arg1 = wrappers[bins.getArg1()].get();
			assert(arg0 && arg1);

			switch (bins.getOpcode()) {
				case AssemblyIns::kBoolAND:
				case AssemblyIns::kBoolNAND:
				case AssemblyIns::kBoolOR:
				case AssemblyIns::kBoolNOR:
				case AssemblyIns::kBoolXOR:
				case AssemblyIns::kBoolEQV: {
					SimulationBooleanBinaryIns::FuncType fn;
					switch (bins.getOpcode()) {
						case AssemblyIns::kBoolAND:
							fn = SimulationBooleanBinaryIns::fAND; break;
						case AssemblyIns::kBoolNAND:
							fn = SimulationBooleanBinaryIns::fNAND; break;
						case AssemblyIns::kBoolOR:
							fn = SimulationBooleanBinaryIns::fOR; break;
						case AssemblyIns::kBoolNOR:
							fn = SimulationBooleanBinaryIns::fNOR; break;
						case AssemblyIns::kBoolXOR:
							fn = SimulationBooleanBinaryIns::fXOR; break;
						default:
							throw std::runtime_error("unknown boolean opcode");
					}

					auto it = dependencies.emplace(
						new SimulationBooleanBinaryIns(result, arg0, arg1, fn));
					w = it.first->get();
					arg0->addDependency(w);
					arg1->addDependency(w);
				} break;

				case AssemblyIns::kStore: {
					auto it = dependencies.emplace(new SimulationStoreIns(
						result, arg0, arg1));
					w = it.first->get();
					arg0->addDependency(w);
					arg1->addDependency(w);
				} break;

				default:
					throw std::runtime_error("unknown binary opcode");
			}
		} break;

		// mux operations
		case AssemblyIns::kMuxOps: {
			switch (ins.getOpcode()) {
				case AssemblyIns::kBimux: {
					auto mins = *(const AssemblyBimuxIns*)&ins;
					auto result = wrappers[mins.getResult()].get();
					auto select = wrappers[mins.getSelect()].get();
					auto case0 = wrappers[mins.getCase0()].get();
					auto case1 = wrappers[mins.getCase1()].get();
					assert(select && case0 && case1);

					auto it = dependencies.emplace(new SimulationBimuxIns(
						result, select, case0, case1));
					w = it.first->get();
					select->addDependency(w);
					case0->addDependency(w);
					case1->addDependency(w);
				} break;
				default:
					throw std::runtime_error("unknown mux opcode");
			}
		} break;

		default:
			throw std::runtime_error("unknown opcode");
	}
}

/// Calls the function \a fn for all signals that are being simulated, passing
/// it each signal and that signal's current value. Useful to get a dump of the
/// current simulation state.
void Simulation::eachSignal(ObserverFunc fn) {
	for (auto& is : wrappers) {
		fn(T, is.second->getAssemblySignal(), is.second->getValue());
	}
}

/// Adds an event to the event queue. Useful for debugging and applying external
/// stimuli generated by the user.
void Simulation::addEvent(
	SimulationTime T,
	const AssemblySignal* signal,
	const SimulationValue& value) {

	auto it = wrappers.find(signal);
	if (it == wrappers.end())
		return;

	eventQueue.addEvent(T, it->second.get(), value);
}

/// Advances the simulation one step and calls \a fn for all signals that have
/// changed their value.
void Simulation::step(ObserverFunc fn) {

	// Do nothing if the event queue is empty.
	if (eventQueue.isAtEnd())
		return;

	// Fetch the next time step from the queue and iterate over all events that
	// are listed for that specific point in time.
	T = eventQueue.nextTime();
	std::vector<SimulationDependency*> outdated;
	outdated.reserve(64);
	eventQueue.nextEvents([&](const SimulationEvent& ev){
		auto& sig = *ev.signal;
		if (sig.getValue() == ev.value)
			return;
		sig.setValue(ev.value);
		fn(T, sig.getAssemblySignal(), ev.value);

		// Mark all dependencies of this signal as outdated.
		sig.eachDependency([&](SimulationDependency *dep){
			if (dep->markOutdated())
				outdated.push_back(dep);
		});
	});
	eventQueue.pop();

	// Update all dependencies that were marked outdated.
	for (auto dep : outdated) {
		dep->update(T, eventQueue);
		dep->clearOutdated();
	}
}
