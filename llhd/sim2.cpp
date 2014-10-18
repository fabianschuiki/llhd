/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyWriter.hpp"
#include "llhd/sim/Simulation.hpp"
#include <fstream>
using namespace llhd;

int main(int argc, char** argv) {

	Assembly as;
	std::shared_ptr<AssemblyModule> mod(new AssemblyModule);
	std::shared_ptr<AssemblySignal> siga(new AssemblySignal);

	mod->name = "@main";
	siga->dir = AssemblySignal::kSignal;
	siga->name = "%clk";
	siga->type.reset(new AssemblyTypeLogic);
	as.modules[mod->name] = mod;
	mod->signals[siga->name] = siga;

	std::ofstream fout("sim2.llhd");
	AssemblyWriter(fout).write(as);

	Simulation sim(*mod);

	return 0;
}
