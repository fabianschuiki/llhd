/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenGroup.hpp"
#include "llhd/vhdl/TokenType.hpp"

using namespace llhd::vhdl;

bool Parser::parseSecondStage(
	Token**& start,
	Token** end,
	TokenGroup& into) {

	while (start != end) {
		auto before = start;

		start++;

		// Sentinel that prevents infinite loops.
		assert(start > before && "parse loop did not progress");
	}
	return false;
}
