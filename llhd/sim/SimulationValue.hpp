/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

namespace llhd {

/// Logic values supported by the simulator. Correspond to VHDL's logic model.
enum SimulationLogicValue {
	kLogicU,  // uninitialized
	kLogicX,  // strong drive, unknown logic value
	kLogic0,  // strong drive, logic zero
	kLogic1,  // strong drive, logic one
	kLogicZ,  // high impedance
	kLogicW,  // weak drive, unknown logic value
	kLogicL,  // weak drive, logic zero
	kLogicH,  // weak drive, logic one
	kLogicDC, // don't care
};

/// A simulated signal value. At the moment only supports simple logic words.
class SimulationValue {
public:
	unsigned width;
	SimulationLogicValue* bits;

	SimulationValue(): width(0), bits(nullptr) {}

	SimulationValue(unsigned w, SimulationLogicValue v) {
		width = w;
		bits = new SimulationLogicValue[w];
		std::fill(bits, bits+w, v);
	}

	SimulationValue(const SimulationValue& v) { *this = v; }
	SimulationValue(SimulationValue&& v) { *this = std::move(v); }

	~SimulationValue() {
		delete[] bits;
		bits = nullptr;
	}

	SimulationValue& operator= (const SimulationValue& v) {
		width = v.width;
		bits = new SimulationLogicValue[width];
		std::copy(v.bits, v.bits+v.width, bits);
		return *this;
	}

	SimulationValue& operator= (SimulationValue&& v) {
		width = v.width;
		bits = v.bits;
		v.bits = nullptr;
		return *this;
	}

	SimulationLogicValue& operator[] (unsigned idx) {
		return bits[idx];
	}

	bool operator==(const SimulationValue& v) const {
		if (width != v.width) return false;
		return std::equal(bits, bits+width, v.bits);
	}
	bool operator!=(const SimulationValue& v) const { return !(*this == v); }
};

} // namespace llhd
