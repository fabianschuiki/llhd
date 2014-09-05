/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/TokenScanner.hpp"
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenGroup.hpp"
#include "llhd/vhdl/TokenType.hpp"
using namespace llhd::vhdl;

#define not_implemented \
	static char name[256] = {0}; \
	if (!*name) \
		snprintf(name, 256, "%s not implemented", __FUNCTION__); \
	addDiagnostic(input.getRange(), kNote, name); \
	return false;
