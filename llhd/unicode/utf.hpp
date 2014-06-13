/* Copyright (c) 2014 Fabian Schuiki */
/// \file
/// Provides mechanisms to encode and decode UTF-8, UTF-16 and UTF-32 strings.
///
/// \note The overall structure is heavily inspired by [boost::locale's
/// utf_traits][1]. Many thanks to the boost developers for coming up with this
/// clean approach.
///
/// \author Fabian Schuiki
///
/// [1]: http://www.boost.org/doc/libs/release/libs/locale/doc/html/utf_8hpp_source.html
#pragma once
#include "llhd/unicode.hpp"

namespace llhd {
namespace unicode {

/// UTF-8 encoding and decoding facilities.
struct utf8 {
	/// Maximum number of UTF-8 characters generated for a single code point.
	static const unsigned maxWidth = 4;

	/// Computes the number of UTF-8 characters needed to encode the code
	/// point \a c.
	static unsigned getWidth(unichar c) {
		if (c <= 0x7F)
			return 1;
		if (c <= 0x7FF)
			return 2;
		if (c <= 0xFFFF)
			return 3;
		return 4;
	}

	/// Checks whether \a u is the leading character in a multibyte group. If
	/// this function returns true, one or more subsequent characters are
	/// trailing characters that encode one code point. Also returns true for
	/// singlebyte characters.
	static bool isLead(utf8char u) {
		return (u & 0xC0) != 0x80;
	}

	/// Checks whether \a u is a trailing character in a multibyte group.
	static bool isTrail(utf8char u) {
		return (u & 0xC0) == 0x80;
	}

	/// Computes the number of trailing characters to be expected after \a u.
	static int getTrailLength(utf8char u) {
		if (u < 0x80)
			return 0;
		if (u < 194)
			return -1;
		if (u < 224)
			return 1;
		if (u < 240)
			return 2;
		if (u < 244)
			return 3;
		return -1;
	}

	/// Decodes a code point from the iterator \a p, thoroughly checking it for
	/// errors. This function is slower than decode(), but catches possible
	/// encoding errors. If you are sure that the data to be decoded is well-
	/// formed UTF-8, use decode() instead.
	///
	/// \param p Iterator from which the bytes to be decoded are fetched. After
	///          the function returns, points to the byte after the decoded
	///          character.
	/// \param e Iterator pointing to the end of the input. Used for error
	///          checking only; the function returns incomplete if p == e
	///          pre maturely.
	/// \return  Returns the decoded code point as unichar, or ::incomplete if
	///          the end of the input was reached prematurely (p == e), or
	///          ::illegal if an encoding error was detected.
	template <typename Iterator>
	static unichar decodePedantic(Iterator& p, Iterator e) {
		if (p == e)
			return incomplete;

		// Calculate the number of characters to expect after this lead. If
		// there are none, we may return the character immediately, thus
		// optimizing for ASCII text. Trail lengths < 0 indicate an illegal
		// encoding.
		utf8char lead = *p++;
		int trail = getTrailLength(lead);
		if (trail == 0)
			return lead;
		if (trail < 0)
			return illegal;

		// In UTF-8, the leading byte encodes the number of bytes as the number
		// of leading ones in its binary representation. E.g. the lead
		// 0b1110xxxx is the first of a 3-byte group, and therefore has 2
		// trailing bytes; the lead 0b110xxxxx is the first of a 2-byte group,
		// and therefore has 1 trailing byte. Note how the number of data bits
		// 'x' is (6 - trail length). To create a mask for the data bits, we
		// simply shift a 1 in front of all data bits, then subtract 1 to turn
		// all bits to the right into a 1. E.g. the lead 0b110xxxxx: We shift a
		// 1 in front of all bits, so 0b00100000, then subtract one to obtain
		// 0b00011111.
		utf8char mask = (1 << (6-trail)) - 1;
		unichar c = lead & mask;

		// Decoding trailing bytes is a fairly regular process, implemented
		// here by falling through the cases of a switch statement.
		assert(trail >= 0 && trail < 4);
		utf8char tmp;
		switch (trail) {
		case 3:
			if (p == e)
				return incomplete;
			tmp = *p++;
			if (!isTrail(tmp))
				return illegal;
			c = (c << 6) | (tmp & 0x3F);
		case 2:
			if (p == e)
				return incomplete;
			tmp = *p++;
			if (!isTrail(tmp))
				return illegal;
			c = (c << 6) | (tmp & 0x3F);
		case 1:
			if (p == e)
				return incomplete;
			tmp = *p++;
			if (!isTrail(tmp))
				return illegal;
			c = (c << 6) | (tmp & 0x3F);
		}

		// Make sure the code point is valid.
		if (!isValid(c))
			return illegal;

		// Make sure the code point was encoded in the most compact manner.
		if (getWidth(c) != trail + 1)
			return illegal;

		return c;
	}

	/// Decodes a code point from the iterator \a p. This function assumes that
	/// the input is encoded correctly and is therefore faster than
	/// decodePedantic(). The downside is that this function is less robust to
	/// encoding errors. While decodePedantic() returns individual ::illegal or
	/// ::incomplete code points but keeps the rest of the input intact,
	/// decode() is likely to only decode garbage after an encoding error was
	/// encountered.
	///
	/// \warning This function will crash horribly if the Iterator \a p cannot
	///          cope with out-of-bounds accesses, since the function is likely
	///          to read beyond the end of the input in case of an encoding
	///          error.
	///
	/// \param p Iterator from which the bytes to be decoded are fetched. After
	///          the function returns, points to the byte after the decoded
	///          character.
	/// \return  Returns the decoded code point as unichar. Depending on the
	///          validity of the input encoding, this may very well be an invalid
	///          code point.
	template <typename Iterator>
	static unichar decode(Iterator& p) {
		// Inspect the lead, returning it immediately if it indicates a single
		// byte character.
		utf8char lead = *p++;
		if (lead < 192)
			return lead;

		// Quickly compute the trail size.
		unsigned trail = 0;
		if (lead < 224)
			trail = 1;
		else if (lead < 240)
			trail = 2;
		else
			trail = 3;

		// Decode the code point. See decodePedantic() for more information.
		utf8char mask = (1 << (6-trail)) - 1;
		unichar c = lead & mask;

		switch (trail) {
		case 3: c = (c << 6) | (utf8char(*p++) & 0x3F);
		case 2: c = (c << 6) | (utf8char(*p++) & 0x3F);
		case 1: c = (c << 6) | (utf8char(*p++) & 0x3F);
		}

		return c;
	}

	/// Encodes a code point to the iterator \a out. Make sure the iterator
	/// points to a destination that has enough room to accept at least
	/// maxWidth characters, or that it is an insertion iterator that will
	/// dynamically expand the destination to accomodate new bytes.
	///
	/// \param c   Code point to be encoded.
	/// \param out Iterator where encoded bytes are written to.
	template <typename Iterator>
	static void encode(unichar c, Iterator& out) {
		assert(isValid(c));
		if (c <= 0x7F) {
			*out++ = utf8char(c);
		} else if (c <= 0x7FF) {
			*out++ = utf8char(((c >>  6) & 0x3F) | 0xC0);
			*out++ = utf8char(((c >>  0) & 0x3F) | 0x80);
		} else if (c <= 0xFFFF) {
			*out++ = utf8char(((c >> 12) & 0x3F) | 0xE0);
			*out++ = utf8char(((c >>  6) & 0x3F) | 0x80);
			*out++ = utf8char(((c >>  0) & 0x3F) | 0x80);
		} else {
			*out++ = utf8char(((c >> 18) & 0x3F) | 0xF0);
			*out++ = utf8char(((c >> 12) & 0x3F) | 0x80);
			*out++ = utf8char(((c >>  6) & 0x3F) | 0x80);
			*out++ = utf8char(((c >>  0) & 0x3F) | 0x80);
		}
	}
};

/// UTF-16 encoding and decoding facilities.
struct utf16 {
	/// Maximum number of UTF-16 characters generated for a single code point.
	static const unsigned maxWidth = 2;

	/// Computes the number of UTF-16 characters needed to encode the code
	/// point \a c.
	static unsigned getWidth(unichar c) {
		return (c <= 0xFFFF) ? 1 : 2;
	}

	/// Checks whether the character \a u belongs to the group of first
	/// surrogates.
	static bool isFirstSurrogate(utf16char u) {
		return 0xD800 <= u && u <= 0xDBFF;
	}

	/// Checks whether the character \a u belongs to the group of second
	/// surrogates.
	static bool isSecondSurrogate(utf16char u) {
		return 0xDC00 <= u && u <= 0xDFFF;
	}

	/// Computes the code point encoded by the two surrogates \a u1 and \a u2.
	static unichar combineSurrogates(utf16char u1, utf16char u2) {
		unichar c1 = u1 & 0x3FF;
		unichar c2 = u2 & 0x3FF;
		return ((c1 << 10) | c2) + 0x10000;
	}

	/// Checks whether \a u is the leading character in a multiword group. If
	/// this function returns true, the subsequent character is a trailing
	/// character and encodes one code point together with this character.
	/// Also returns true for singleword characters.
	static bool isLead(utf16char u) {
		return !isSecondSurrogate(u);
	}

	/// Checks whether \a u is a trailing character in a multiword group.
	static bool isTrail(utf16char u) {
		return isSecondSurrogate(u);
	}

	/// Computes the number of trailing characters to be expected after \a u.
	static int getTrailLength(utf16char u) {
		if (isFirstSurrogate(u))
			return 1;
		if (isSecondSurrogate(u))
			return -1;
		return 0;
	}

	/// Decodes a code point from the iterator \a p, thoroughly checking it for
	/// errors. This function is slower than decode(), but catches possible
	/// encoding errors. If you are sure that the data to be decoded is well-
	/// formed UTF-16, use decode() instead.
	///
	/// \param p Iterator from which the words to be decoded are fetched. After
	///          the function returns, points to the word after the decoded
	///          character.
	/// \param e Iterator pointing to the end of the input. Used for error
	///          checking only; the function returns ::incomplete if p == e
	///          pre maturely.
	/// \return  Returns the decoded code point as unichar, or ::incomplete if
	///          the end of the input was reached prematurely (p == e), or
	///          ::illegal if an encoding error was detected.
	template <typename Iterator>
	static unichar decodePedantic(Iterator& p, Iterator e) {
		if (p == e)
			return incomplete;

		// Fetch the first character. If it is not a surrogate, return it
		// immediately as this indicates a singleword character. Otherwise
		// ensure it is in the range of a first surrogate and that we have not
		// read past the end of the input.
		utf16char u1 = *p++;
		if (!isSurrogate(u1))
			return u1;
		if (u1 > 0xDBFF)
			return illegal;
		if (p == e)
			return incomplete;

		// Fetch the second character. This now needs to be a second surrogate.
		utf16char u2 = *p++;
		if (!isSecondSurrogate(u2))
			return illegal;

		// Combine the two surrogates into a code point.
		return combineSurrogates(u1, u2);
	}

	/// Decodes a code point from the iterator \a p. This function assumes that
	/// the input is encoded correctly and is therefore faster than
	/// decodePedantic(). The downside is that this function is less robust to
	/// encoding errors. While decodePedantic() returns individual ::illegal or
	/// ::incomplete code points but keeps the rest of the input intact,
	/// decode() is likely to only decode garbage after an encoding error was
	/// encountered.
	///
	/// \warning This function will crash horribly if the Iterator \a p cannot
	///          cope with out-of-bounds accesses, since the function is likely
	///          to read beyond the end of the input in case of an encoding
	///          error.
	///
	/// \param p Iterator from which the words to be decoded are fetched. After
	///          the function returns, points to the word after the decoded
	///          character.
	/// \return  Returns the decoded code point as unichar. Depending on the
	///          validity of the input encoding, this may very well be an
	///          invalid code point.
	template <typename Iterator>
	static unichar decode(Iterator& p) {
		utf16char u1 = *p++;
		if (!isSurrogate(u1))
			return u1;
		utf16char u2 = *p++;
		return combineSurrogates(u1, u2);
	}

	/// Encodes a code point to the iterator \a out. Make sure the iterator
	/// points to a destination that has enough room to accept at least
	/// maxWidth words, or that it is an insertion iterator that will
	/// dynamically expand the destination to accomodate new words.
	///
	/// \param c   Code point to be encoded.
	/// \param out Iterator where encoded words are written to.
	template <typename Iterator>
	static void encode(unichar c, Iterator& out) {
		assert(isValid(c));
		if (c <= 0xFFFF) {
			*out++ = utf16char(c);
		} else {
			c -= 0x10000;
			*out++ = utf16char(0xD800 | (c >> 10));
			*out++ = utf16char(0xDC00 | (c & 0x3FF));
		}
	}
};

} // namespace unicode
} // namespace llhd
