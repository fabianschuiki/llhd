/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/sim/Simulation.hpp"
#include <iostream>
using namespace llhd;

Simulation::Simulation(const AssemblyModule& as):
	as(as) {

	// Initialize the signal wrappers.
	for (auto& sig : as.signals) {
		std::cout << "wrapping " << sig.second->name << '\n';
	}
}
