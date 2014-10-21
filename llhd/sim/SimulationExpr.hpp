/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/sim/SimulationDependency.hpp"
#include <cassert>

namespace llhd {

/// Expression that simply passes values from one signal to another, delayed by
/// one delta step.
class SimulationIdentityExpr : public SimulationDependency {
	SimulationSignal *out;
	const SimulationSignal *arg;

public:
	SimulationIdentityExpr(
		SimulationSignal *out,
		const SimulationSignal *arg):
		out(out),
		arg(arg) {}

	void update(SimulationTime T, SimulationEventQueue& queue) {
		queue.addEvent(T.advDelta(), out, arg->getValue());
	}
};


/// Expression that passes values from one signal to another with a controllable
/// delay.
class SimulationDelayExpr : public SimulationDependency {
	SimulationSignal *out;
	const SimulationSignal *arg;
	uint64_t delay;

public:
	SimulationDelayExpr(
		SimulationSignal *out,
		const SimulationSignal *arg,
		uint64_t delay):
		out(out),
		arg(arg),
		delay(delay) {}

	void update(SimulationTime T, SimulationEventQueue& queue) {
		queue.addEvent(
			delay > 0 ? T.advTime(delay) : T.advDelta(),
			out, arg->getValue());
	}
};


/// Expression that applies a binary boolean function to two signals in a
/// bitwise manner.
class SimulationBooleanExpr : public SimulationDependency {
public:
	typedef std::function<char(char,char)>
		FuncType;

	SimulationBooleanExpr(
		SimulationSignal *out,
		const SimulationSignal *arg0,
		const SimulationSignal *arg1,
		FuncType fn):
		out(out),
		arg0(arg0),
		arg1(arg1),
		fn(fn) {}

	static char fAND(char a, char b) { return a && b; }
	static char fNAND(char a, char b) { return !(a && b); }
	static char fOR(char a, char b) { return a || b; }
	static char fNOR(char a, char b) { return !(a || b); }
	static char fXOR(char a, char b) { return a != b; }

	void update(SimulationTime T, SimulationEventQueue& queue) {
		const auto& v0 = arg0->getValue();
		const auto& v1 = arg1->getValue();
		assert(v0.width == v1.width);
		SimulationValue r(v0.width, kLogicU);

		auto *b0 = v0.bits, *b1 = v1.bits, *br = r.bits, *be = v0.bits+v0.width;
		for (; b0 != be; b0++, b1++, br++) {
			char i0, i1;

			if (*b0 == kLogic0 || *b0 == kLogicL)
				i0 = 0;
			else if (*b0 == kLogic1 || *b0 == kLogicH)
				i0 = 1;
			else
				continue;

			if (*b1 == kLogic0 || *b1 == kLogicL)
				i1 = 0;
			else if (*b1 == kLogic1 || *b1 == kLogicH)
				i1 = 1;
			else
				continue;

			char r = fn(i0,i1);
			*br = (r ? kLogic1 : kLogic0);
		}

		queue.addEvent(T.advDelta(), out, r);
	}

private:
	SimulationSignal *out;
	const SimulationSignal *arg0;
	const SimulationSignal *arg1;
	FuncType fn;
};

} // namespace llhd
