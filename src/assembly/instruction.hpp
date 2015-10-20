/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/assembly/time.hpp"
#include "llhd/assembly/type.hpp"
#include "llhd/assembly/value.hpp"
#include "llhd/utils/memory.hpp"
#include <string>
#include <vector>

namespace llhd {

/// \needsdoc
/// \ingroup assembly
class Instruction {
public:
	Instruction() = default;

	template <typename T>
	Instruction(T x) : m_self(std::make_shared<Model<T>>(std::move(x))) {}

	friend std::string to_string(const Instruction &x) { return x.m_self->to_string_(); }
	explicit operator bool() const { return bool(m_self); }

private:
	struct Concept {
		virtual ~Concept() = default;
		virtual std::string to_string_() const = 0;
	};

	template <typename T>
	struct Model : Concept {
		Model(T x) : x(std::move(x)) {}
		virtual std::string to_string_() const override { return to_string(x); }
		T x;
	};

	std::shared_ptr<const Concept> m_self;
};


inline std::string to_string(const std::string &x) {
	return x;
}

template <typename Iterator>
std::string to_string(Iterator first, Iterator last) {
	if (first == last)
		return nullptr;
	std::string r = to_string(*first);
	++first;
	while (first != last) {
		r += ", " + to_string(*first);
		++first;
	}
	return r;
}

inline std::string to_string(std::tuple<Type,Value> x) {
	return to_string(std::get<0>(x)) + " " + to_string(std::get<1>(x));
}


class LabelInstruction {
public:
	std::string name;

	friend std::string to_string(const LabelInstruction &x) {
		return x.name;
	}
};


class RetInstruction {
public:
	friend std::string to_string(const RetInstruction &/*x*/) {
		return "ret";
	}
};


class AddInstruction {
public:
	std::string name;
	Value arga;
	Value argb;

	friend std::string to_string(const AddInstruction &x) {
		std::string r = "add " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class AndInstruction {
public:
	std::string name;
	Value arga;
	Value argb;

	friend std::string to_string(const AndInstruction &x) {
		std::string r = "and " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class OrInstruction {
public:
	std::string name;
	Value arga;
	Value argb;

	friend std::string to_string(const OrInstruction &x) {
		std::string r = "or " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class SubInstruction {
public:
	std::string name;
	Value arga;
	Value argb;

	friend std::string to_string(const SubInstruction &x) {
		std::string r = "sub " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};

class XorInstruction {
public:
	std::string name;
	Value arga;
	Value argb;

	friend std::string to_string(const XorInstruction &x) {
		std::string r = "xor " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class DriveInstruction {
public:
	std::string target;
	bool clear = false;
	Value value;
	bool has_time = false;
	Time time;

	friend std::string to_string(const DriveInstruction &x) {
		std::string r = "drv " + x.target;
		if (x.clear)
			r += " clear";
		r += " ";
		r += to_string(x.value);
		if (x.has_time)
			r += " " + to_string(x.time);
		return r;
	}
};


class NotInstruction {
public:
	std::string name;
	Value value;

	friend std::string to_string(const NotInstruction &x) {
		std::string r = "not " + to_string(x.value);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class TimedWaitInstruction {
public:
	bool absolute;
	Time time;

	friend std::string to_string(const TimedWaitInstruction &x) {
		std::string r = "wait ";
		if (x.absolute)
			r += "abs ";
		r += to_string(x.time);
		return r;
	}
};


class ConditionalWaitInstruction {
public:
	Value cond;
	std::string dest;

	friend std::string to_string(const ConditionalWaitInstruction &x) {
		return "wait cond " + to_string(x.cond) + " " + x.dest;
	}
};


class UnconditionalWaitInstruction {
public:
	friend std::string to_string(const UnconditionalWaitInstruction &/*x*/) {
		return "wait";
	}
};


class StoreInstruction {
public:
	Value addr;
	Value value;

	friend std::string to_string(const StoreInstruction &x) {
		return "st " + to_string(x.addr) + " " + to_string(x.value);
	}
};


class LoadInstruction {
public:
	std::string name;
	Type type;
	Value addr;

	friend std::string to_string(const LoadInstruction &x) {
		std::string r = "ld " + to_string(x.type) + " " + to_string(x.addr);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class UnconditionalBranchInstruction {
public:
	Value dest;

	friend std::string to_string(const UnconditionalBranchInstruction &x) {
		return "br " + to_string(x.dest);
	}
};

class ConditionalBranchInstruction {
public:
	Value cond;
	Value dest_true;
	Value dest_false;

	friend std::string to_string(const ConditionalBranchInstruction &x) {
		return "br " + to_string(x.cond) + ", " + to_string(x.dest_true) + ", " + to_string(x.dest_false);
	}
};


class SignalInstruction {
public:
	std::string name;
	Type type;
	Value initial;

	friend std::string to_string(const SignalInstruction &x) {
		std::string r = "sig " + to_string(x.type);
		if (bool(x.initial))
			r += ", " + to_string(x.initial);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class AllocInstruction {
public:
	std::string name;
	Type type;
	Value initial;

	friend std::string to_string(const AllocInstruction &x) {
		std::string r = "alloc " + to_string(x.type);
		if (bool(x.initial))
			r += ", " + to_string(x.initial);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


enum CompareType {
	CMP_TYPE_EQ,
	CMP_TYPE_NE,
	CMP_TYPE_SGT,
	CMP_TYPE_SLT,
	CMP_TYPE_SGE,
	CMP_TYPE_SLE,
	CMP_TYPE_UGT,
	CMP_TYPE_ULT,
	CMP_TYPE_UGE,
	CMP_TYPE_ULE,
};

inline std::string to_string(CompareType type) {
	switch (type) {
		case CMP_TYPE_EQ:  return "eq";
		case CMP_TYPE_NE:  return "ne";
		case CMP_TYPE_SGT: return "sgt";
		case CMP_TYPE_SLT: return "slt";
		case CMP_TYPE_SGE: return "sge";
		case CMP_TYPE_SLE: return "sle";
		case CMP_TYPE_UGT: return "ugt";
		case CMP_TYPE_ULT: return "ult";
		case CMP_TYPE_UGE: return "uge";
		case CMP_TYPE_ULE: return "ule";
		default: return nullptr;
	}
}

class CompareInstruction {
public:
	std::string name;
	CompareType type;
	Value arga;
	Value argb;

	friend std::string to_string(const CompareInstruction &x) {
		std::string r = "cmp " + to_string(x.type) + " " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


enum InstructionSign {
	INS_SIGNED,
	INS_UNSIGNED,
};

inline std::string to_string(InstructionSign sign) {
	switch (sign) {
		case INS_SIGNED:   return "signed";
		case INS_UNSIGNED: return "unsigned";
		default: return nullptr;
	}
}


class MulInstruction {
public:
	std::string name;
	InstructionSign sign;
	Value arga;
	Value argb;

	friend std::string to_string(const MulInstruction &x) {
		std::string r = "mul " + to_string(x.sign) + " " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class ModInstruction {
public:
	std::string name;
	InstructionSign sign;
	Value arga;
	Value argb;

	friend std::string to_string(const ModInstruction &x) {
		std::string r = "div " + to_string(x.sign) + " mod " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class RemInstruction {
public:
	std::string name;
	InstructionSign sign;
	Value arga;
	Value argb;

	friend std::string to_string(const RemInstruction &x) {
		std::string r = "div " + to_string(x.sign) + " rem " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class DivInstruction {
public:
	std::string name;
	InstructionSign sign;
	Value arga;
	Value argb;

	friend std::string to_string(const DivInstruction &x) {
		std::string r = "div " + to_string(x.sign) + " " + to_string(x.arga) + " " + to_string(x.argb);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class LmapInstruction {
public:
	std::string name;
	Type type;
	Value value;

	friend std::string to_string(const LmapInstruction &x) {
		std::string r = "lmap " + to_string(x.type) + " " + to_string(x.value);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class TruncInstruction {
public:
	std::string name;
	Type type;
	Value value;

	friend std::string to_string(const TruncInstruction &x) {
		std::string r = "trunc " + to_string(x.type) + " " + to_string(x.value);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class SignExtInstruction {
public:
	std::string name;
	Type type;
	Value value;

	friend std::string to_string(const SignExtInstruction &x) {
		std::string r = "ext signed " + to_string(x.type) + " " + to_string(x.value);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class PaddingExtInstruction {
public:
	std::string name;
	Type type;
	Value value;
	Value padding;

	friend std::string to_string(const PaddingExtInstruction &x) {
		std::string r = "ext " + to_string(x.type) + " " + to_string(x.value) + " " + to_string(x.padding);
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class CatInstruction {
public:
	std::string name;
	std::vector<std::tuple<Type,Value>> args;

	friend std::string to_string(const CatInstruction &x) {
		std::string r = "cat " + to_string(std::begin(x.args), std::end(x.args));
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class SelInstruction {
public:
	std::string name;
	Type arg_type;
	Value arg;
	std::vector<std::tuple<unsigned,unsigned>> ranges;

	friend std::string to_string(const SelInstruction &x) {
		std::string r = "sel " + to_string(x.arg_type) + " " + to_string(x.arg);
		for (auto rng : x.ranges) {
			r += ", " + std::to_string(std::get<0>(rng));
			if (std::get<0>(rng) != std::get<1>(rng))
				r += "-" + std::to_string(std::get<1>(rng));
		}
		return x.name.empty() ? r : x.name + " = " + r;
	}
};


class InstInstruction {
public:
	std::string name;
	std::vector<std::string> inputs;
	std::vector<std::string> outputs;

	friend std::string to_string(const InstInstruction &x) {
		return "inst " + x.name + " (" + to_string(std::begin(x.inputs), std::end(x.inputs)) + ") (" + to_string(std::begin(x.outputs), std::end(x.outputs)) + ")";
	}
};


class CallInstruction {
public:
	std::string name;
	std::vector<std::string> outputs;
	std::vector<Value> inputs;

	friend std::string to_string(const CallInstruction &x) {
		std::string r = "call " + x.name + " (" + to_string(std::begin(x.inputs), std::end(x.inputs)) + ")";
		return x.outputs.empty() ? r : to_string(std::begin(x.outputs), std::end(x.outputs)) + " = " + r;
	}
};

} // namespace llhd
