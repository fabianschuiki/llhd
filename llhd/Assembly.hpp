/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace llhd {

typedef uint64_t AssemblyDuration;
class AssemblySignal;
class AssemblyUnaryIns;
class AssemblyBinaryIns;

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
		kEdge,
		kRisingEdge,
		kFallingEdge,
		kBoolNOT,

		// Binary Instructions
		kBinaryOps = 0x2000,
		kBoolAND,
		kBoolOR,
		kBoolNAND,
		kBoolNOR,
		kBoolXOR,
		kBoolEQV,
		kStore,

		// Multiplexer Instructions
		kMuxOps = 0x3000,
		kBimux,
		kMux,

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
		const AssemblySignal* result,
		AssemblyDuration delay,
		const AssemblySignal* arg):
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
		const AssemblySignal* result,
		const AssemblySignal* arg0,
		const AssemblySignal* arg1):
		AssemblyIns(opcode, result),
		arg0(arg0),
		arg1(arg1) {}

	const AssemblySignal* getArg0() const { return arg0; }
	const AssemblySignal* getArg1() const { return arg1; }

private:
	const AssemblySignal* arg0;
	const AssemblySignal* arg1;
};


class AssemblyBimuxIns : public AssemblyIns {
public:
	explicit AssemblyBimuxIns(
		Opcode opcode,
		const AssemblySignal* result,
		const AssemblySignal* select,
		const AssemblySignal* case0,
		const AssemblySignal* case1):
		AssemblyIns(opcode, result),
		select(select),
		case0(case0),
		case1(case1) {}

	const AssemblySignal* getSelect() const { return select; }
	const AssemblySignal* getCase0() const { return case0; }
	const AssemblySignal* getCase1() const { return case1; }

private:
	const AssemblySignal* select;
	const AssemblySignal* case0;
	const AssemblySignal* case1;
};


class AssemblyMuxIns : public AssemblyIns {
public:
	explicit AssemblyMuxIns(
		Opcode opcode,
		const AssemblySignal* result,
		const AssemblySignal* select):
		AssemblyIns(opcode, result),
		select(select) {}

	const AssemblySignal* getSelect() const { return select; }

private:
	const AssemblySignal* select;
	std::vector<std::pair<void*, const AssemblySignal*>> cases;
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

	AssemblySignal() {}
	explicit AssemblySignal(
		Direction dir,
		const std::string& name,
		const std::shared_ptr<AssemblyType> type):
		dir(dir),
		name(name),
		type(type) {}

	Direction getDirection() const { return dir; }
	void setDirection(Direction dir) { this->dir = dir; }

	const std::string& getName() const { return name; }
	void setName(const std::string& name) { this->name = name; }

	const std::shared_ptr<AssemblyType>& getType() const { return type; }
	void setType(const std::shared_ptr<AssemblyType>& type) {
		this->type = type;
	}

private:
	Direction dir;
	std::string name;
	std::shared_ptr<AssemblyType> type;
	// std::shared_ptr<AssemblyIns> assignment;
};

class AssemblyModule {
public:
	// std::string name;
	// std::map<std::string, std::shared_ptr<const AssemblySignal>> signals;

	AssemblyModule() {}
	explicit AssemblyModule(const std::string& name): name(name) {}

	void setName(const std::string& name) { this->name = name; }
	const std::string& getName() const { return name; }


	void addSignal(const std::shared_ptr<const AssemblySignal>& sig) {
		signals.push_back(sig);
	}

	void addSignal(std::shared_ptr<const AssemblySignal>&& sig) {
		signals.push_back(std::move(sig));
	}

	template<typename... Args>
	const AssemblySignal* newSignal(Args&&... args) {
		auto i = std::make_shared<AssemblySignal>(args...);
		auto p = i.get();
		signals.emplace_back(std::move(i));
		return p;
	}

	void eachSignal(std::function<void(const AssemblySignal&)> fn) const {
		for (auto& sig : signals)
			fn(*sig);
	}


	void addInstruction(const std::shared_ptr<const AssemblyIns>& ins) {
		instructions.push_back(ins);
	}

	void addInstruction(std::shared_ptr<const AssemblyIns>&& ins) {
		instructions.push_back(std::move(ins));
	}

	template<typename T, typename... Args>
	const T* newInstruction(Args&&... args) {
		auto i = std::make_shared<T>(args...);
		auto p = i.get();
		instructions.emplace_back(std::move(i));
		return p;
	}

	void eachInstruction(std::function<void(const AssemblyIns&)> fn) const {
		for (auto& ins : instructions)
			fn(*ins);
	}

private:
	std::string name;
	std::vector<std::shared_ptr<const AssemblySignal>> signals;
	std::vector<std::shared_ptr<const AssemblyIns>> instructions;
};

class Assembly {
public:
	std::map<std::string, std::shared_ptr<const AssemblyModule>> modules;
};

} // namespace llhd
