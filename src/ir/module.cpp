/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/entity.hpp"
#include "llhd/ir/module.hpp"
#include "llhd/ir/process.hpp"

namespace llhd {

Module::~Module() {
	for (auto E : entities)  delete E;
	for (auto P : processes) delete P;
	// for (auto F : functions) delete F;
}

} // namespace llhd
