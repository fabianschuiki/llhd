/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/ir/entity.hpp"
#include "llhd/ir/module.hpp"

namespace llhd {

Entity::Entity(Module * parent, const std::string & name)
:	M(parent), name(name) {
	parent->getEntityList().push_back(this);
}

} // namespace llhd
