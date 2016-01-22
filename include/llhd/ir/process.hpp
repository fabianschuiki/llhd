/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

class Argument;
class BasicBlock;
class Module;

class Process : public OwnedBy<Module> {
public:
	typedef std::vector<Argument*> ArgumentList;
	typedef std::vector<BasicBlock*> BasicBlockList;

	Process(const std::string & name, Module * parent = nullptr);
	~Process();

	const std::string & getName() const { return name; }

	ArgumentList & getInputList() { return inputs; }
	const ArgumentList & getInputList() const { return inputs; }

	ArgumentList & getOutputList() { return outputs; }
	const ArgumentList & getOutputList() const { return outputs; }

	BasicBlockList & getBasicBlockList() { return basicBlocks; }
	const BasicBlockList & getBasicBlockList() const { return basicBlocks; }

private:
	std::string name;
	ArgumentList inputs;
	ArgumentList outputs;
	BasicBlockList basicBlocks;
};

} // namespace llhd
