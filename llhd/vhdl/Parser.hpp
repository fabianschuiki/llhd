/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <istream>

namespace llhd {
namespace vhdl {

class Parser
{
public:
	Parser();
	~Parser();

	void parse(std::istream& input);
};

} // namespace vhdl
} // namespace llhd
