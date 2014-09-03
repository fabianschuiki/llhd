/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/Buffer.hpp"

namespace llhd {
namespace vhdl {

class TokenGroup {
public:
	unsigned type;
	union {
		Buffer<TokenGroup*> groups;
		Buffer<Token*> tokens;
	};
};

enum TokenGroupType {
	kTokenGroupRaw = 0x0,
	kTokenGroupParen,
	kTokenGroupBrack,
	kTokenGroupBrace,

	kTokenGroupEntity,
	kTokenGroupArchitecture
};

} // namespace vhdl
} // namespace llhd
