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

	std::shared_ptr<AssemblySignal> sigrst(new AssemblySignal);
	sigrst->dir = AssemblySignal::kSignal;
	sigrst->name = "%rst";
	sigrst->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sigb(new AssemblySignal);
	sigb->dir = AssemblySignal::kSignal;
	sigb->name = "%clk2";
	sigb->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sigc(new AssemblySignal);
	sigc->dir = AssemblySignal::kSignal;
	sigc->name = "%inv";
	sigc->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sigd(new AssemblySignal);
	sigd->dir = AssemblySignal::kSignal;
	sigd->name = "%xord";
	sigd->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> sige(new AssemblySignal);
	sige->dir = AssemblySignal::kSignal;
	sige->name = "%cnt_dn";
	sige->type.reset(new AssemblyTypeLogic);

	std::shared_ptr<AssemblySignal> rega(new AssemblySignal);
	rega->dir = AssemblySignal::kRegister;
	rega->name = "%cnt_dp";
	rega->type.reset(new AssemblyTypeLogic);

	auto expra = std::make_shared<AssemblyUnaryIns>(AssemblyIns::kMove, 0, siga.get(), sigb.get());
	// expra->op = siga.get();
	sigb->assignment = expra;

	auto exprb = std::make_shared<AssemblyUnaryIns>(AssemblyIns::kMove, 3, sigb.get(), sigc.get());
	// exprb->op = sigb.get();
	// exprb->d = 3;
	sigc->assignment = exprb;

	auto exprc = std::make_shared<AssemblyBinaryIns>(AssemblyIns::kBoolXOR, siga.get(), sigc.get(), sigd.get());
	// exprc->type = AssemblyBinaryIns::kXOR;
	// exprc->op0 = siga.get();
	// exprc->op1 = sigc.get();
	sigd->assignment = exprc;

	auto exprd = std::make_shared<AssemblyBinaryIns>(AssemblyIns::kBoolNOR, rega.get(), rega.get(), sige.get());
	// exprd->type = AssemblyBinaryIns::kNOR;
	// exprd->op0 = rega.get();
	// exprd->op1 = rega.get();
	sige->assignment = exprd;

	std::shared_ptr<AssemblyModule> mod(new AssemblyModule);
	mod->name = "@main";
	mod->signals[siga->name] = siga;
	mod->signals[sigrst->name] = sigrst;
	mod->signals[sigb->name] = sigb;
	mod->signals[sigc->name] = sigc;
	mod->signals[sigd->name] = sigd;
	mod->signals[sige->name] = sige;
	mod->signals[rega->name] = rega;

	Assembly as;
	as.modules[mod->name] = mod;

	std::ofstream fout("sim2.llhd");
	AssemblyWriter(fout).write(as);

	Simulation sim(*mod);
	sim.addEvent(0, sigrst.get(), SimulationValue(1, kLogic1));
	sim.addEvent(3, sigrst.get(), SimulationValue(1, kLogic0));
	sim.addEvent(13, sigrst.get(), SimulationValue(1, kLogic1));
	for (unsigned i = 1; i < 20; i++) {
		sim.addEvent(i*10+0, siga.get(), SimulationValue(1, kLogic1));
		sim.addEvent(i*10+5, siga.get(), SimulationValue(1, kLogic0));
	}

	unsigned namebase = 0;
	std::map<const AssemblySignal*, std::string> names;

	std::ofstream fvcd("sim2.vcd");
	fvcd << "$version llhd-sim2 0.1.0 $end\n";
	fvcd << "$timescale 1ns $end\n";
	fvcd << "$scope module logic $end\n";
	sim.eachSignal([&](
		SimulationTime T,
		const AssemblySignal* sig,
		const SimulationValue& value){

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
		SimulationTime T,
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
	sim.eachSignal(valueDump);
	fvcd << "$end\n\n";

	uint64_t lastT = -1;
	while (!sim.isAtEnd()) {
		sim.step([&](
			SimulationTime T,
			const AssemblySignal* sig,
			const SimulationValue& value) {

			if (T.value != lastT) {
				fvcd << "#" << T.value << '\n';
				lastT = T.value;
			}
			valueDump(T,sig,value);
		});
	}
	fvcd << "#" << sim.getTime().value << '\n';

	return 0;
}
