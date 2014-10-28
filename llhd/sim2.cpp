/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyWriter.hpp"
#include "llhd/sim/Simulation.hpp"
#include <fstream>
#include <iostream>
using namespace llhd;

int main(int argc, char** argv) {

	auto mod = std::make_shared<AssemblyModule>("@main");

	auto sig_clk = mod->newSignal(AssemblySignal::kSignal, "%clk",
		std::make_shared<AssemblyTypeLogic>());
	// sig_clk->type.reset(new AssemblyTypeLogic);

	auto sig_rst = mod->newSignal(AssemblySignal::kSignal, "%rst",
		std::make_shared<AssemblyTypeLogic>());

	auto sigb = mod->newSignal(AssemblySignal::kSignal, "%clk2",
		std::make_shared<AssemblyTypeLogic>());

	auto sigc = mod->newSignal(AssemblySignal::kSignal, "%inv",
		std::make_shared<AssemblyTypeLogic>());

	auto sigd = mod->newSignal(AssemblySignal::kSignal, "%xord",
		std::make_shared<AssemblyTypeLogic>());

	auto sige = mod->newSignal(AssemblySignal::kSignal, "%cnt_dpb",
		std::make_shared<AssemblyTypeLogic>());

	auto sigf = mod->newSignal(AssemblySignal::kSignal, "%const0",
		std::make_shared<AssemblyTypeLogic>());

	auto sigg = mod->newSignal(AssemblySignal::kSignal, "%const1",
		std::make_shared<AssemblyTypeLogic>());

	auto sig_cnt_dn = mod->newSignal(AssemblySignal::kSignal, "%cnt_dn",
		std::make_shared<AssemblyTypeLogic>());

	auto sig_cnt_dp = mod->newSignal(AssemblySignal::kRegister, "%cnt_dp",
		std::make_shared<AssemblyTypeLogic>());

	auto sig_rise = mod->newSignal(AssemblySignal::kSignal, "%clk_rise",
		std::make_shared<AssemblyTypeLogic>());

	auto sig_rise_rst = mod->newSignal(AssemblySignal::kSignal, "%clk_rise_rst",
		std::make_shared<AssemblyTypeLogic>());

	mod->newInstruction<AssemblyUnaryIns>(AssemblyIns::kMove, sigb, 0, sig_clk);
	mod->newInstruction<AssemblyUnaryIns>(AssemblyIns::kMove, sigc, 3, sigb);
	mod->newInstruction<AssemblyBinaryIns>(AssemblyIns::kBoolXOR, sigd, sig_clk, sigc);
	mod->newInstruction<AssemblyBimuxIns>(AssemblyIns::kBimux, sig_rise_rst, sig_rst, sigg, sig_rise);
	mod->newInstruction<AssemblyUnaryIns>(AssemblyIns::kBoolNOT, sige, 0, sig_cnt_dp);
	mod->newInstruction<AssemblyUnaryIns>(AssemblyIns::kRisingEdge, sig_rise, 0, sig_clk);
	mod->newInstruction<AssemblyBimuxIns>(AssemblyIns::kBimux, sig_cnt_dn, sig_rst, sigf, sige);
	mod->newInstruction<AssemblyBinaryIns>(AssemblyIns::kStore, sig_cnt_dp, sig_rise_rst, sig_cnt_dn);

	Assembly as;
	as.modules[mod->getName()] = mod;

	std::ofstream fout("sim2.llhd");
	AssemblyWriter(fout).write(as);
	fout.close();

	Simulation sim(*mod);
	sim.addEvent(0, sigf, SimulationValue(1, kLogic0));
	sim.addEvent(0, sigg, SimulationValue(1, kLogic1));
	sim.addEvent(0, sig_rst, SimulationValue(1, kLogic1));
	sim.addEvent(3, sig_rst, SimulationValue(1, kLogic0));
	sim.addEvent(13, sig_rst, SimulationValue(1, kLogic1));
	for (unsigned i = 1; i < 20; i++) {
		sim.addEvent(i*10+0, sig_clk, SimulationValue(1, kLogic1));
		sim.addEvent(i*10+5, sig_clk, SimulationValue(1, kLogic0));
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
		fvcd << "$var wire " << value.width << " " << name << " " << sig->getName()
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
