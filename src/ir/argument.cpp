/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/argument.hpp"

namespace llhd {

Argument::Argument(const std::string & name, Type * type, Process * parent):
	Value(type),
	OwnedBy(parent),
	name(name),
	type(type) {
}

} // namespace llhd
