/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace llhd {

typedef uint64_t AssemblyDuration;
class AssemblySignal;

/// Base class for all assembly instructions. Stores an instruction's opcode and
/// potential result signal. The opcode may be used to identify what specific
/// type of instruction this is and to cast the instruction to one of its
/// subclasses.
class AssemblyIns {
public:
	/// Assembly instruction opcodes.
	enum Opcode {

		// Unary Instructions
		kUnaryOps = 0x1000,
		kMove,
		kBoolNOT,

		// Binary Instructions
		kBinaryOps = 0x2000,
		kBoolAND,
		kBoolOR,
		kBoolNAND,
		kBoolNOR,
		kBoolXOR,
		kBoolEQV,

		kOpMask = 0xf000
	};

	explicit AssemblyIns(
		Opcode opcode,
		const AssemblySignal* result):
		opcode(opcode),
		result(result) {}
	virtual ~AssemblyIns() {}

	/// Returns the instruction's opcode.
	Opcode getOpcode() const { return opcode; }

	/// Returns the signal which carries the instruction's result, or \c nullptr
	/// if there is none.
	const AssemblySignal* getResult() const { return result; }

private:
	Opcode opcode;
	const AssemblySignal* result;
};


/// Base class for all unary assembly instructions. These are instructions that
/// take one signal as argument and produces a corresponding result.
class AssemblyUnaryIns : public AssemblyIns {
public:

	explicit AssemblyUnaryIns(
		Opcode opcode,
		AssemblyDuration delay,
		const AssemblySignal* arg,
		const AssemblySignal* result):
		AssemblyIns(opcode, result),
		delay(delay),
		arg(arg) {}

	AssemblyDuration getDelay() const { return delay; }
	const AssemblySignal* getArg() const { return arg; }

private:
	AssemblyDuration delay;
	const AssemblySignal* arg;
};


/// Base class for all binary assembly instructions. These are instructions that
/// take two signals as arguments and produce a corresponding result.
class AssemblyBinaryIns : public AssemblyIns {
public:

	explicit AssemblyBinaryIns(
		Opcode opcode,
		const AssemblySignal* arg0,
		const AssemblySignal* arg1,
		const AssemblySignal* result = nullptr):
		AssemblyIns(opcode, result),
		arg0(arg0),
		arg1(arg1) {}

	const AssemblySignal* getArg0() const { return arg0; }
	const AssemblySignal* getArg1() const { return arg1; }

private:
	const AssemblySignal* arg0;
	const AssemblySignal* arg1;
};


class AssemblyType {
public:
	virtual ~AssemblyType() {}
};

class AssemblyTypeLogic : public AssemblyType {};

class AssemblyTypeWord : public AssemblyType {
public:
	unsigned width;
	std::shared_ptr<AssemblyType> type;
};

class AssemblySignal {
public:
	enum Direction {
		kSignal,
		kRegister,
		kPortIn,
		kPortOut
	};

	Direction dir;
	std::string name;
	std::shared_ptr<AssemblyType> type;
	std::shared_ptr<AssemblyIns> assignment;
};

class AssemblyModule {
public:
	std::string name;
	std::map<std::string, std::shared_ptr<const AssemblySignal>> signals;
};

class Assembly {
public:
	std::map<std::string, std::shared_ptr<const AssemblyModule>> modules;
};

} // namespace llhd
