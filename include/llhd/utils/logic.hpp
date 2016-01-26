/* Copyright (c) 2016 Fabian Schuiki */
#pragma once
#include "llhd/common.hpp"

namespace llhd {

/// Logic value of arbitrary bit width.
class Logic {
public:
	enum Bit {
		U,X,O,I,Z,W,L,H,DC
	};

	// TODO:
	// - and
	// - or
	// - xor
	// - not
	// - soft equality (1)
	// - soft inequality (1)
	//
	// DONE:
	// - construct by copy
	// - construct by move
	// - construct from string
	// - string conversion
	// - copy assignment
	// - move assignment
	// - equality
	// - inequality
	//
	// (1) considering that L == O, H == I, X != X, DC == *, etc.

	explicit Logic(unsigned width, Bit initial = U);
	Logic(const Logic & other);
	Logic(Logic && other);
	explicit Logic(const std::string & str);
	~Logic();

	Logic & operator=(const Logic & other);
	Logic & operator=(Logic && other);

	bool operator==(const Logic & other) const;
	bool operator!=(const Logic & other) const;

	std::string toString() const;
	unsigned getWidth() const { return width; }

private:
	unsigned width;
	uint64_t * data;

	unsigned numWords() const { return (width+15)/16; }
};

// TODO:
// - add
// - sub
// - signed mul
// - unsigned mul
// - signed div+mod+rem
// - unsigned div+mod+rem
// - signed LT,GT,LE,GE
// - unsigned LT,GT,LE,GE

} // namespace llhd
