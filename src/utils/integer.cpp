/* Copyright (c) 2016 Fabian Schuiki */
#include "llhd/utils/integer.hpp"

namespace llhd {


/// General addition for 64bit integer arrays. Calculates \a dst = \a x + \a y.
/// Returns the carry bit of the addition, i.e. true if the addition overflowed.
bool add(uint64_t * dst, const uint64_t * x, const uint64_t * y, unsigned len) {
	bool carry = false;
	for (unsigned i = 0; i < len; ++i) {
		auto limit = std::min(x[i], y[i]);
		dst[i] = x[i] + y[i] + carry;
		carry = (dst[i] < limit) || (carry && dst[i] == limit);
	}
	return carry;
}


/// General subtraction for 64bit integer arrays. Calculates \a dst = \a x -
/// \a y. Returns the borrow bit of the subtraction, i.e. true if the
/// subtraction underflowed.
bool sub(uint64_t * dst, const uint64_t * x, const uint64_t * y, unsigned len) {
	bool borrow = false;
	for (unsigned i = 0; i < len; ++i) {
		auto xb = x[i] - borrow;
		borrow = (y[i] > xb) || (borrow && x[i] == 0);
		dst[i] = xb - y[i];
	}
	return borrow;
}


/// General multiplication for 64bit integer arrays. Calculates \a dst = \a x *
/// \a y.
void mul(uint64_t * dst, unsigned dstlen, const uint64_t * x, unsigned xlen, const uint64_t * y, unsigned ylen) {
	llhd_assert(dst != x);
	llhd_assert(dst != y);
	std::fill(dst, dst+dstlen, 0);
	const uint64_t bit32 = uint64_t(1) << 32;
	for (unsigned i = 0; i < ylen && i < dstlen; ++i) {
		uint64_t ly = uint32_t(y[i]);
		uint64_t hy = uint32_t(y[i] >> 32);
		uint64_t carry = 0;

		for (unsigned j = 0; j < xlen && i+j < dstlen; ++j) {
			uint64_t lx = uint32_t(x[j]);
			uint64_t hx = uint32_t(x[i] >> 32);

			// hasCarry == 0: no carry
			// hasCarry == 1: carry
			// hasCarry == 2: no carry, and calculation result == 0
			unsigned hasCarry = 0;

			// lower x * lower y
			uint64_t result = carry + lx * ly;
			hasCarry = (result < carry);

			// higher x * lower y
			carry = (hasCarry ? bit32 : 0) + hx * ly + (result >> 32);
			hasCarry = (carry == 0 ? (hasCarry > 0 ? 1 : 2) : 0);

			// lower x * higher y
			carry += uint32_t(lx * hy);
			result = (carry << 32) | uint32_t(result);
			dst[i+j] += result;

			// higher x * higher y => carry
			carry = ((carry == 0 && hasCarry != 2) || hasCarry == 1 ? bit32 : 0)
				+ (carry >> 32) + (dst[i+j] < result ? 1 : 0)
				+ ((lx * hy) >> 32) + hx * hy;
		}

		if (i+xlen < dstlen)
			dst[i+xlen] = carry;
	}
}


unsigned countLeadingZeros(uint64_t * x, unsigned len) {
	for (unsigned i = 0; i < len; ++i) {
		if (x[len-i-1] != 0)
			return i*64 + countLeadingZeros(x[len-i-1]);
	}
	return len*64;
}

unsigned countLeadingZeros(uint64_t x) {
	unsigned result = 0;
	if (x >> 32 != 0) { result += 32; x >>= 32; }
	if (x >> 16 != 0) { result += 16; x >>= 16; }
	if (x >>  8 != 0) { result +=  8; x >>=  8; }
	if (x >>  4 != 0) { result +=  4; x >>=  4; }
	if (x >>  2 != 0) { result +=  2; x >>=  2; }
	if (x >>  1 != 0) { result +=  1; x >>=  1; }
	return 64-result;
}

unsigned countLeadingOnes(uint64_t * x, unsigned len) {
	for (unsigned i = 0; i < len; ++i) {
		if (x[len-i-1] != ~uint64_t(0))
			return i*64 + countLeadingOnes(x[len-i-1]);
	}
	return len*64;
}

unsigned countLeadingOnes(uint64_t x) {
	return countLeadingZeros(~x);
}


Integer & Integer::operator=(const Integer & that) {
	resize(that.bitWidth);
	std::copy(that.words, that.words+that.getWordWidth(), words);
	return *this;
}

Integer & Integer::operator=(Integer && that) {
	if (!isSingleWord())
		delete[] words;
	bitWidth = that.bitWidth;
	words = that.words;
	that.words = nullptr;
	return *this;
}


Integer & Integer::add(const Integer & that) {
	llhd_assert(bitWidth == that.bitWidth);
	if (isSingleWord()) {
		value += that.value;
	} else {
		llhd::add(words, words, that.words, getWordWidth());
		clearUnusedBits();
	}
	return *this;
}

Integer & Integer::sub(const Integer & that) {
	llhd_assert(bitWidth == that.bitWidth);
	if (isSingleWord()) {
		value -= that.value;
	} else {
		llhd::sub(words, words, that.words, getWordWidth());
		clearUnusedBits();
	}
	return *this;
}

Integer & Integer::umul(const Integer & that) {
	if (isSingleWord()) {
		value *= that.value;
	} else {
		auto tmp = new uint64_t[getWordWidth()];
		llhd::mul(tmp, getWordWidth(), words, getWordWidth(), that.words, that.getWordWidth());
		delete[] words;
		words = tmp;
		clearUnusedBits();
	}
	return *this;
}

Integer & Integer::smul(const Integer & that) {
	llhd_unimplemented();
	return *this;
}

Integer & Integer::lsl(unsigned shift) {
	unsigned width = getWordWidth();
	if (shift == 0) {
		// do nothing
	} else if (isSingleWord()) {
		value <<= shift;
	} else if (shift >= bitWidth) {
		std::fill(words, words+width, 0);
	} else {
		int major = shift / 64;
		int minor = shift % 64;
		for (int i = width-1; i >= 0; --i) {
			uint64_t wh = (i >= major   ? words[i-major]   : 0);
			uint64_t wl = (i >= major+1 ? words[i-major-1] : 0);
			words[i] = wh << minor | wl >> (64-minor);
		}
	}
	clearUnusedBits();
	return *this;
}

Integer & Integer::lsr(unsigned shift) {
	unsigned width = getWordWidth();
	if (shift == 0) {
		// do nothing
	} else if (isSingleWord()) {
		value >>= shift;
	} else if (shift >= bitWidth) {
		std::fill(words, words+width, 0);
	} else {
		int major = shift / 64;
		int minor = shift % 64;
		for (unsigned i = 0; i < width; ++i) {
			uint64_t wh = (i < width-major-1 ? words[i+major+1] : 0);
			uint64_t wl = (i < width-major   ? words[i+major]   : 0);
			words[i] = wh << (64-minor) | wl >> minor;
		}
	}
	return *this;
}

Integer & Integer::asr(unsigned shift) {
	unsigned width = getWordWidth();
	if (shift == 0) {
		// do nothing
	} else if (isSingleWord()) {
		unsigned unused = getUnusedBits();
		value = int64_t(value << unused) >> (shift + unused);
	} else {
		bool isNegative = get(bitWidth-1);
		uint64_t mask = isNegative ? ~uint64_t(0) : 0;
		if (shift >= bitWidth) {
			std::fill(words, words+width, mask);
		} else {
			unsigned major = shift / 64;
			unsigned minor = shift % 64;
			for (unsigned i = 0; i < width; ++i) {
				uint64_t wh = (i < width-major-1 ? words[i+major+1] : mask);
				uint64_t wl = (i < width-major   ? words[i+major]   : mask);
				words[i] = wh << (64-minor) | wl >> minor;
			}
		}
	}
	clearUnusedBits();
	return *this;
}

Integer & Integer::rol(unsigned shift) {
	shift %= bitWidth;
	unsigned width = getWordWidth();
	if (shift == 0) {
		// do nothing
	} else if (isSingleWord()) {
		value = value << shift | value >> (bitWidth-shift);
	} else {
		int major = shift / 64;
		int minor = shift % 64;
		llhd_unimplemented();
		for (int i = width-1; i >= 0; --i) {
			// Tricky since the topmost word need not be fully used...
			// uint64_t wh = words[(i-major+width)   % width];
			// uint64_t wl = words[(i-major+width-1) % width];
			// words[i] = wh << minor | wl >> (64-minor);
		}
	}
	clearUnusedBits();
	return *this;
}

Integer & Integer::ror(unsigned shift) {
	shift %= bitWidth;
	llhd_unimplemented();
	return *this;
}


Integer add(const Integer & a, const Integer & b) {
	llhd_assert(a.bitWidth == b.bitWidth);
	Integer r(a.bitWidth);
	if (a.isSingleWord())
		r.value = a.value + b.value;
	else
		llhd::add(r.words, a.words, b.words, r.getWordWidth());
	r.clearUnusedBits();
	return r;
}

Integer sub(const Integer & a, const Integer & b) {
	llhd_assert(a.bitWidth == b.bitWidth);
	Integer r(a.bitWidth);
	if (a.isSingleWord())
		r.value = a.value - b.value;
	else
		llhd::sub(r.words, a.words, b.words, r.getWordWidth());
	r.clearUnusedBits();
	return r;
}

Integer umul(const Integer & a, const Integer & b) {
	llhd_assert(a.bitWidth == b.bitWidth);
	Integer r(a.bitWidth);
	if (a.isSingleWord())
		r.value = a.value * b.value;
	else
		llhd::mul(r.words, r.getWordWidth(), a.words, a.getWordWidth(), b.words, b.getWordWidth());
	r.clearUnusedBits();
	return r;
}

Integer smul(const Integer & a, const Integer & b) {
	llhd_unimplemented();
}

Integer lsl(const Integer & a, unsigned shift) {
	Integer r(a);
	r.lsl(shift);
	return r;
}

Integer lsr(const Integer & a, unsigned shift) {
	Integer r(a);
	r.lsr(shift);
	return r;
}

Integer asr(const Integer & a, unsigned shift) {
	Integer r(a);
	r.asr(shift);
	return r;
}


Integer Integer::operator+(const Integer & that) const {
	return llhd::add(*this, that);
}

Integer Integer::operator-(const Integer & that) const {
	return llhd::sub(*this, that);
}


} // namespace llhd
