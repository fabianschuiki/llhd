/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/assembly/instruction.hpp"
#include "llhd/assembly/type.hpp"
#include <string>
#include <vector>

namespace llhd {

/// \needsdoc
/// \ingroup assembly
class FunctionArgument {
public:
	Type type;
	std::string name;

	friend std::string to_string(const FunctionArgument &x) {
		return to_string(x.type) + " " + x.name;
	}
};


inline std::string to_string(const std::vector<FunctionArgument> &x, const std::string &join = ", ") {
	auto i = x.begin();
	if (i == x.end())
		return std::string();
	std::string result = to_string(*i);
	++i;
	for (; i != x.end(); ++i)
		result += join + to_string(*i);
	return result;
}


/// \needsdoc
/// \ingroup assembly
class Function {
public:
	std::string name;
	std::vector<FunctionArgument> inputs;
	std::vector<FunctionArgument> outputs;
	std::vector<Instruction> instructions;

	friend std::string to_string(const Function &x) {
		std::string r = "func " + x.name + " (" + to_string(x.inputs) + ") (" + to_string(x.outputs) + ") {\n";
		for (auto &ins : x.instructions)
			r += "    " + to_string(ins) + "\n";
		r += "}";
		return r;
	}
};

} // namespace llhd
