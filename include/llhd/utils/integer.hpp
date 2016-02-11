/* Copyright (c) 2016 Fabian Schuiki */
/// @file
/// This file contains utilities to work with integers,

#pragma once
#include "llhd/common.hpp"

namespace llhd {


bool add(uint64_t * dst, const uint64_t * x, const uint64_t * y, unsigned len);
bool sub(uint64_t * dst, const uint64_t * x, const uint64_t * y, unsigned len);
void mul(uint64_t * dst, unsigned dstlen, const uint64_t * x, unsigned xlen, const uint64_t * y, unsigned ylen);

unsigned countLeadingZeros(uint64_t * x, unsigned len);
unsigned countLeadingZeros(uint64_t x);
unsigned countLeadingOnes(uint64_t * x, unsigned len);
unsigned countLeadingOnes(uint64_t x);

template<typename T> T upper1(unsigned bits) { return ~(~T(0) >> bits); }
template<typename T> T upper0(unsigned bits) { return ~T(0) >> bits; }
template<typename T> T lower1(unsigned bits) { return ~(~T(0) << bits); }
template<typename T> T lower0(unsigned bits) { return ~T(0) << bits; }


class Integer {
	/// Width of the number in bits.
	unsigned bitWidth;

	/// Value of the number, either as the value itself (of bits <= 64), or a
	/// pointer to memory containing the words.
	union {
		uint64_t value;
		uint64_t * words;
	};

	/// Constructs a zeroed Integer of the given @a bitWidth. Allocates memory
	/// in case of a multi-word integer.
	explicit Integer(unsigned bitWidth): bitWidth(bitWidth), words(nullptr) {
		unsigned nw = getWordWidth();
		if (nw > 1) {
			words = new uint64_t[nw];
			std::fill(words, words+nw, 0);
		}
	}

	bool isSingleWord() const { return bitWidth <= 64; }
	static unsigned whichWord(unsigned index) { return index / 64; }
	static unsigned whichBit(unsigned index) { return index % 64; }

	/// Get the value of a single bit.
	bool get(unsigned index) const {
		llhd_assert(index < bitWidth);
		if (isSingleWord())
			return (value >> index) & 1;
		else
			return (words[whichWord(index)] >> whichBit(index)) & 1;
	}

	/// Set the value of a single bit;
	void set(unsigned index, bool v) {
		llhd_assert(index < bitWidth);
		uint64_t mask = ~(1 << whichBit(index));
		uint64_t bit = (v << whichBit(index));
		uint64_t & word = (isSingleWord() ? value : words[whichWord(index)]);
		word &= mask;
		word |= bit;
	}

	/// Clears the unused bits in the uppermost word.
	void clearUnusedBits() {
		unsigned bits = getUnusedBits();
		if (bits > 0) {
			if (isSingleWord())
				value &= upper0<uint64_t>(bits);
			else
				words[getWordWidth()-1] &= upper0<uint64_t>(bits);
		}
	}

	/// Changes the bit width to @a width. Reallocates memory as appropriate.
	void resize(unsigned width) {
		if (getWordWidth() == getWordWidth(width))
			return;
		if (!isSingleWord())
			delete[] words;
		bitWidth = width;
		if (!isSingleWord())
			words = new uint64_t[getWordWidth()];
	}

public:
	/// @name Constructors
	/// @{

	/// Construct an Integer initialized to @a value. If @a isSigned is true,
	/// the value is sign-extended to @a bitWidth, otherwise it is
	/// zero-extended.
	Integer(unsigned bitWidth, uint64_t value, bool isSigned = false): Integer(bitWidth) {
		if (isSingleWord())
			this->value = value;
		else {
			words[0] = value;
			if (isSigned && (value & upper1<uint64_t>(1)))
				std::fill(words+1, words+getWordWidth(), ~uint64_t(0));
		}
		clearUnusedBits();
	}

	/// Copy constructor.
	Integer(const Integer & that): Integer(that.bitWidth) {
		if (isSingleWord())
			value = that.value;
		else
			std::copy(that.words, that.words+getWordWidth(), words);
	}

	/// Move constructor. Using @a that afterwards yields undefined behaviour.
	Integer(Integer && that): bitWidth(that.bitWidth) {
		words = that.words;
		that.words = nullptr;
	}

	/// Raw constructor. The new instance of Integer takes ownership of the
	/// memory pointed to by @a words, which needs to be at least getWordWidth()
	/// words in length.
	Integer(unsigned bitWidth, uint64_t * words): bitWidth(bitWidth), words(words) {
		clearUnusedBits();
	}

	/// Construct an Integer initialized to @a words. If @a isSigned is true,
	/// the words are sign-extended to @a bitWidth, otherwise they are
	/// zero-extended.
	Integer(unsigned bitWidth, const uint64_t * words, unsigned numWords, bool isSigned = false): Integer(bitWidth) {
		if (isSingleWord())
			value = words[0];
		else {
			std::copy(words, words+numWords, this->words);
			if (isSigned && (words[numWords-1] & upper1<uint64_t>(1)))
				std::fill(this->words+numWords, this->words+getWordWidth(), ~uint64_t(0));
		}
		clearUnusedBits();
	}

	~Integer() {
		if (!isSingleWord())
			delete[] words;
	}
	/// @}

	unsigned getBitWidth() const { return bitWidth; }
	static unsigned getWordWidth(unsigned bits) { return (bits+63)/64; }
	unsigned getWordWidth() const { return getWordWidth(bitWidth); }

	unsigned getUnusedBits() const { return getWordWidth() * 64 - bitWidth; }
	unsigned getActiveBits() const { return bitWidth - countLeadingZeros(); }
	unsigned getActiveWords() const {
		unsigned bits = getActiveBits();
		return bits > 0 ? whichWord(bits-1)+1 : 0;
	}

	unsigned countLeadingZeros() const {
		unsigned unused = getUnusedBits();
		if (isSingleWord())
			return llhd::countLeadingZeros(value) - unused;
		else
			return llhd::countLeadingZeros(words, getWordWidth()) - unused;
	}

	unsigned countLeadingOnes() const {
		unsigned unused = getUnusedBits();
		if (isSingleWord())
			return llhd::countLeadingOnes(value) - unused;
		else
			return llhd::countLeadingOnes(words, getWordWidth()) - unused;
	}

	Integer & operator=(const Integer & that);
	Integer & operator=(Integer && that);

	Integer operator+(const Integer & that) const;
	Integer operator-(const Integer & that) const;

	Integer & operator+=(const Integer & that) { return add(that); }
	Integer & operator-=(const Integer & that) { return sub(that); }

	Integer & add(const Integer & that);
	Integer & sub(const Integer & that);
	Integer & umul(const Integer & that);
	Integer & smul(const Integer & that);
	Integer & lsl(unsigned shift);
	Integer & lsr(unsigned shift);
	Integer & asr(unsigned shift);

	friend Integer add(const Integer & a, const Integer & b);
	friend Integer sub(const Integer & a, const Integer & b);
	friend Integer umul(const Integer & a, const Integer & b);
	friend Integer smul(const Integer & a, const Integer & b);
	friend Integer lsl(const Integer & a, unsigned shift);
	friend Integer lsr(const Integer & a, unsigned shift);
	friend Integer asr(const Integer & a, unsigned shift);

	uint64_t getZExtValue() const {
		if (isSingleWord()) {
			return value;
		} else {
			llhd_assert_msg(getActiveBits() <= 64, "value does not fit into uint64_t");
			return words[0];
		}
	}

	int64_t getSExtValue() const {
		unsigned min = getMinSignedBits();
		unsigned unused = 64 - (min % 64);
		if (isSingleWord()) {
			return int64_t(value << unused) >> unused;
		} else {
			llhd_assert_msg(min <= 64, "value does not fit into int64_t");
			return int64_t(words[0] << unused) >> unused;
		}
	}

	unsigned getMinSignedBits() const {
		if (get(bitWidth-1))
			return bitWidth - countLeadingOnes() + 1;
		else
			return getActiveBits() + 1;
	}

	// TODO:
	// smul
	// rol,ror
	// fromString
	// toString
	// to uint64_t, to int64_t
};


} // namespace llhd
