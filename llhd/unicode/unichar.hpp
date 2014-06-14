/* Copyright (c) 2014 Fabian Schuiki */
/// \file
/// \author Fabian Schuiki
#pragma once
#include "llhd/compiler.hpp"

namespace llhd {
namespace unicode {

/// A single 8 bit character in a UTF-8-encoded string.
typedef uint8_t  utf8char;
/// A single 16 bit character in a UTF-16-encoded string.
typedef uint16_t utf16char;
/// A single 32 bit character in a UTF-32-encoded string.
typedef uint32_t utf32char;
/// A unicode code point. Equivalent to the UTF-32 representation of the code.
typedef uint32_t unichar;

/// Code point indicating an illegal character.
const unichar illegal    = -1; // = 0xFFFFFFFFu
/// Code point indicating an incomplete character.
const unichar incomplete = -2; // = 0xFFFFFFFEu


/// Checks whether the code point \a c is a non-character. The last two code
/// points of each plane are non-characters; i.e. \c 0x00FFFE and \c 0x00FFFF,
/// \c 0x01FFFE and \c 0x01FFFF, until \c 0x10FFFE and \c 0x10FFFF.
/// Additionally, the BMP (Basic Multilingual Plane) contains 32 contiguous
/// non-characters in the range \c 0xFDD0-0xFDEF. See [Universal Character Set
/// characters on Wikipedia][1] for more information.
///
/// [1]: http://en.wikipedia.org/wiki/Universal_Character_Set_characters
inline bool isNonCharacter(unichar c) {
	if ((c & 0xFFFE) == 0xFFFE) // 0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF ... 0x10FFFF
		return true;
	if (0xFDD0 <= c && c <= 0xFDEF) // continuous range of 32 non-characters in the BMP
		return true;
	return false;
}

/// Checks whether the code point \a c is a surrogate code point. Unicode
/// defines these to map all BMP codes into one word in UTF-16. See [Universal
/// Character Set characters on Wikipedia][1] for more information.
///
/// [1]: http://en.wikipedia.org/wiki/Universal_Character_Set_characters
inline bool isSurrogate(unichar c) {
	return (0xD800 <= c && c <= 0xDFFF);
}

/// Checks whether the code point \a c is valid. A code point is invalid if it
/// lies outside the unicode definition range (0x0..0x10FFFF), is a non-
/// character or a surrogate.
///
/// \return Returns true if the code point \a c is a valid character.
inline bool isValid(unichar c) {
	if (c > 0x10FFFF)
		return false;
	if (isNonCharacter(c))
		return false;
	if (isSurrogate(c))
		return false;
	return true;
}


const utf8char* fullCaseFold(const utf8char* c);
const utf8char* simpleCaseFold(const utf8char* c);

} // namespace unicode
} // namespace llhd


// Export the fundamental types into the llhd namespace for convenience.
// Otherwise the entire source would be littered with using statements for
// utf8char and friends.
namespace llhd {

using llhd::unicode::utf8char;
using llhd::unicode::utf16char;
using llhd::unicode::utf32char;
using llhd::unicode::unichar;

} // namespace llhd
