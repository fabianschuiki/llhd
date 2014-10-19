/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyWriter.hpp"
#include "llhd/sim/Simulation.hpp"
#include <fstream>
#include <iostream>
using namespace llhd;

int main(int argc, char** argv) {


	std::shared_ptr<AssemblySignal> siga(new AssemblySignal);
	siga->dir = AssemblySignal::kSignal;
	siga->name = "%clk";
	siga->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sigb(new AssemblySignal);
	sigb->dir = AssemblySignal::kSignal;
	sigb->name = "%clk2";
	sigb->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sigc(new AssemblySignal);
	sigc->dir = AssemblySignal::kSignal;
	sigc->name = "%inv";
	sigc->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblyExprIdentity> expra(new AssemblyExprIdentity);
	expra->op = siga.get();
	sigb->assignment = expra;

	std::shared_ptr<AssemblyExprDelayed> exprb(new AssemblyExprDelayed);
	exprb->op = sigb.get();
	exprb->d = 2000;
	sigc->assignment = exprb;

	std::shared_ptr<AssemblyModule> mod(new AssemblyModule);
	mod->name = "@main";
	mod->signals[siga->name] = siga;
	mod->signals[sigb->name] = sigb;
	mod->signals[sigc->name] = sigc;

	Assembly as;
	as.modules[mod->name] = mod;

	std::ofstream fout("sim2.llhd");
	AssemblyWriter(fout).write(as);

	Simulation sim(*mod);
	sim.observe(siga.get());

	for (unsigned i = 1; i < 20; i++) {
		sim.addEvent((i*10+0) * 1000, siga.get(), SimulationValue(1, kLogic1));
		sim.addEvent((i*10+5) * 1000, siga.get(), SimulationValue(1, kLogic0));
	}

	unsigned namebase = 0;
	std::map<const AssemblySignal*, std::string> names;

	std::ofstream fvcd("sim2.vcd");
	fvcd << "$version llhd-sim2 0.1.0 $end\n";
	fvcd << "$timescale 1ns $end\n";
	fvcd << "$scope module logic $end\n";
	sim.dump([&](const AssemblySignal* sig, const SimulationValue& value){
		std::string name;
		unsigned max;
		for (max = 94; namebase >= max; max *= 94);
		for (unsigned dv = max / 94; dv > 0; dv /= 94) {
			unsigned v = (namebase / dv) % 94;
			name += 33+v;
		}
		++namebase;
		fvcd << "$var wire " << value.width << " " << name << " " << sig->name
			<< " $end\n";
		names[sig] = name;
	});
	fvcd << "$upscope $end\n";
	fvcd << "$enddefinitions $end\n\n";

	auto valueDump = [&](
		const AssemblySignal* sig,
		const SimulationValue& value){

		fvcd << 'b';
		for (unsigned i = 0; i < value.width; i++) {
			switch (value.bits[i]) {
				case kLogicU:  fvcd << 'u'; break;
				case kLogicX:  fvcd << 'x'; break;
				case kLogic0:  fvcd << '0'; break;
				case kLogic1:  fvcd << '1'; break;
				case kLogicZ:  fvcd << 'z'; break;
				case kLogicW:  fvcd << 'w'; break;
				case kLogicL:  fvcd << 'l'; break;
				case kLogicH:  fvcd << 'h'; break;
				case kLogicDC: fvcd << '-'; break;
			}
		}
		fvcd << ' ' << names[sig] << '\n';
	};

	fvcd << "$dumpvars\n";
	sim.dump(valueDump);
	fvcd << "$end\n\n";

	uint64_t lastT = -1;
	while (!sim.isAtEnd()) {
		sim.step([&](const AssemblySignal* sig, const SimulationValue& value) {
			auto T = sim.getTime();
			if (T.ps != lastT) {
				fvcd << "#" << T.ps/1000 << '\n';
				lastT = T.ps;
			}
			valueDump(sig,value);
		});
	}
	fvcd << "#" << sim.getTime().ps/1000 << '\n';

	return 0;
}
