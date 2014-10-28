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


/// Instruction that applies a unary boolean function to a signal if the signal
/// is 0, 1, L, or H; and produces an undefined signal otherwise.
class SimulationBooleanUnaryIns : public SimulationDependency {
public:
	typedef std::function<char(char)> FuncType;

	SimulationBooleanUnaryIns(
		SimulationSignal* out,
		const SimulationSignal* arg,
		FuncType fn):
		out(out),
		arg(arg),
		fn(fn) {}

	static char fNOT(char a) { return !a; }

	void update(SimulationTime T, SimulationEventQueue& queue) {
		const auto& v = arg->getValue();
		SimulationValue r(v.width, kLogicU);

		auto *b = v.bits, *br = r.bits, *be = v.bits+v.width;
		for (; b != be; b++, br++) {
			char i;

			if (*b == kLogic0 || *b == kLogicL)
				i = 0;
			else if (*b == kLogic1 || *b == kLogicH)
				i = 1;
			else
				continue;

			char r = fn(i);
			*br = (r ? kLogic1 : kLogic0);
		}

		queue.addEvent(T.advDelta(), out, r);
	}

private:
	SimulationSignal *out;
	const SimulationSignal *arg;
	FuncType fn;
};


/// Instruction that applies a binary boolean function to two signals if both
/// are 0, 1, L, or H; and produces an undefined signal otherwise.
class SimulationBooleanBinaryIns : public SimulationDependency {
public:
	typedef std::function<char(char,char)> FuncType;

	SimulationBooleanBinaryIns(
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
	static char fEQV(char a, char b) { return a == b; }

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


class SimulationEdgeIns : public SimulationDependency {
public:
	SimulationEdgeIns(
		SimulationSignal* out,
		const SimulationSignal* arg,
		bool rise,
		bool fall):
		out(out),
		arg(arg),
		rise(rise),
		fall(fall),
		last(arg->getValue()) {}

	void update(SimulationTime T, SimulationEventQueue& queue) {
		const auto& v = arg->getValue();
		assert(v.width == last.width);
		SimulationValue r(v.width, kLogicU);

		bool any = false;
		auto *bp = v.bits, *bl = last.bits, *br = r.bits, *be = v.bits+v.width;
		for (; bp != be; bp++, bl++, br++) {
			if (*bp != *bl) {
				if (*bp == kLogic0 || *bp == kLogicL) {
					*br = fall ? kLogic1 : kLogic0;
					any = true;
				}
				if (*bp == kLogic1 || *bp == kLogicH) {
					*br = rise ? kLogic1 : kLogic0;
					any = true;
				}
			}
		}

		last = v;
		if (any) {
			auto T0 = T.advDelta();
			auto T1 = T0.advDelta();
			queue.addEvent(T0, out, r);
			queue.addEvent(T1, out, SimulationValue(r.width, kLogic0));
		}
	}

private:
	SimulationSignal* out;
	const SimulationSignal* arg;
	bool rise, fall;
	SimulationValue last;
};


class SimulationStoreIns : public SimulationDependency {
public:
	SimulationStoreIns(
		SimulationSignal* out,
		const SimulationSignal* trigger,
		const SimulationSignal* data):
		out(out),
		trigger(trigger),
		data(data) {}

	void update(SimulationTime T, SimulationEventQueue& queue) {
		const auto& tv = trigger->getValue();
		assert(tv.width == 1);
		if (*tv.bits == kLogic1 || *tv.bits == kLogicH) {
			queue.addEvent(T.advDelta(), out, data->getValue());
		}
	}

private:
	SimulationSignal* out;
	const SimulationSignal* trigger;
	const SimulationSignal* data;
};


class SimulationBimuxIns : public SimulationDependency {
public:
	SimulationBimuxIns(
		SimulationSignal* out,
		const SimulationSignal* select,
		const SimulationSignal* case0,
		const SimulationSignal* case1):
		out(out),
		select(select),
		case0(case0),
		case1(case1) {}

	void update(SimulationTime T, SimulationEventQueue& queue) {
		const auto& v0 = case0->getValue();
		const auto& v1 = case1->getValue();
		const auto& vs = select->getValue();
		assert(vs.width == 1);
		assert(v0.width == v1.width);

		if (*vs.bits == kLogic0 || *vs.bits == kLogicL) {
			queue.addEvent(T.advDelta(), out, v0);
		} else if (*vs.bits == kLogic1 || *vs.bits == kLogicH) {
			queue.addEvent(T.advDelta(), out, v1);
		} else {
			queue.addEvent(
				T.advDelta(),
				out,
				SimulationValue(v0.width, kLogicU));
		}
	}

private:
	SimulationSignal* out;
	const SimulationSignal* select;
	const SimulationSignal* case0;
	const SimulationSignal* case1;
};

} // namespace llhd
