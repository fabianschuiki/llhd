/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/utils/logic.hpp"

namespace llhd {

static Logic::Bit charToBit(char c) {
	switch (c) {
		case 'u':
		case 'U': return Logic::U;
		case 'x':
		case 'X': return Logic::X;
		case '0': return Logic::O;
		case '1': return Logic::I;
		case 'z':
		case 'Z': return Logic::Z;
		case 'w':
		case 'W': return Logic::W;
		case 'l':
		case 'L': return Logic::L;
		case 'h':
		case 'H': return Logic::H;
		case '-': return Logic::DC;
		default:
			llhd_abort_msg("input character not a valid logic bit");
	}
}

static char bitToChar(Logic::Bit b) {
	switch (b) {
		case Logic::U: return 'U';
		case Logic::X: return 'X';
		case Logic::O: return '0';
		case Logic::I: return '1';
		case Logic::Z: return 'Z';
		case Logic::W: return 'W';
		case Logic::L: return 'L';
		case Logic::H: return 'H';
		case Logic::DC: return '-';
		default:
			llhd_abort_msg("invalid input bit");
	}
}

static uint64_t lowerNBits(unsigned N) {
	llhd_assert(N <= 64);
	return (uint64_t)(-1) >> (64-N);
}


Logic::Logic(unsigned width, Bit initial):
	width(width) {
	data = new uint64_t[numWords()];
	unsigned word = 0;
	for (unsigned i = 0; i < 64; i += 4)
		word |= initial << i;
	for (unsigned i = 0; i < numWords(); ++i)
		data[i] = word;
}

Logic::Logic(const Logic & other):
	width(other.width) {
	data = new uint64_t[numWords()];
	std::copy(other.data, other.data+numWords(), data);
}

Logic::Logic(Logic && other):
	width(other.width) {
	data = other.data;
	other.data = nullptr;
}

Logic::Logic(const std::string & str):
	width(str.size()) {
	data = new uint64_t[numWords()];
	std::fill(data, data+numWords(), 0);
	for (unsigned i = 0; i < width; ++i)
		data[i/16] |= charToBit(str[width-i-1]) << ((i%16)*4);
}

Logic::~Logic() {
	delete[] data;
}

Logic & Logic::operator=(const Logic & other) {
	if (width != other.width) {
		delete[] data;
		width = other.width;
		data = new uint64_t[numWords()];
	}
	std::copy(other.data, other.data+numWords(), data);
	return *this;
}

Logic & Logic::operator=(Logic && other) {
	delete[] data;
	width = other.width;
	data = other.data;
	other.data = nullptr;
	return *this;
}

bool Logic::operator==(const Logic & other) const {
	if (width != other.width)
		return false;
	unsigned topWord = numWords()-1;
	for (unsigned i = 0; i < topWord; ++i)
		if (data[i] != other.data[i])
			return false;
	unsigned bitsLeft = width % 16;
	return bitsLeft == 0 || (data[topWord] & lowerNBits(bitsLeft*4)) == (other.data[topWord] & lowerNBits(bitsLeft*4));
}

bool Logic::operator!=(const Logic & other) const {
	return !operator==(other);
}

std::string Logic::toString() const {
	std::string result(width, 0);
	for (unsigned i = 0; i < width; ++i) {
		result[width-i-1] = bitToChar(Bit((data[i/16] >> ((i%16)*4)) & 0xF));
	}
	return result;
}

} // namespace llhd
