/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

namespace llhd {
namespace vhdl {
namespace ast {

struct Node {
	
};

template<typename T> struct slot

struct DesignUnit : public Node {

};

struct DesignFile : public Node {
	Slots<DesignUnit> designUnits;
};

} // namespace ast
} // namespace vhdl
} // namespace llhd
