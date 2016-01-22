/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/module.hpp"
#include "llhd/ir/process.hpp"
#include "llhd/ir/argument.hpp"

namespace llhd {

Process::Process(const std::string & name, Module * parent):
	OwnedBy(parent),
	name(name) {
	parent->getProcessList().push_back(this);
}

Process::~Process() {
	for (auto A : inputs)
		delete A;
	for (auto A : outputs)
		delete A;
}

} // namespace llhd
