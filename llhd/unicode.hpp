/* Copyright (c) 2014 Fabian Schuiki */
/// \file
/// Unicode support library.
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
static const unichar illegal    = -1; // = 0xFFFFFFFFu
/// Code point indicating an incomplete character.
static const unichar incomplete = -2; // = 0xFFFFFFFEu


/// Checks whether the unicode code point \a c is valid. A code point may be
/// invalid if it lies outside the unicode definition range (0x0..0x10FFFF),
/// if it is 
inline bool isValid(unichar c) {
	// Check whether the code point is inside the definition range.
	if (c > 0x10FFFF)
		return false;
	// Check whether the code point belongs to the "non-characters".
	if ((c & 0xFFFE) == 0xFFFE) // 0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF ... 0x10FFFF
		return false;
	if (0xFDD0 <= c)
	if (0xD800 <= c && c <= 0xDFFF) // surrogates
		return false;
}

/// Checks whether the code point \a c is a non-character. The last two code
/// points of each plane are non-characters; i.e. 0x00FFFE and 0x00FFFF,
/// 0x01FFFE and 0x01FFFF, until 0x10FFFE and 0x10FFFF. Additionally, the BMP
/// (Basic Multilingual Plane) contains 32 contiguous non-characters in the
/// range 0xFDD0-0xFDEF. See [Universal Character Set characters on Wikipedia][1]
/// for more information.
///
/// [1]: http://en.wikipedia.org/wiki/Universal_Character_Set_characters
inline bool isNonCharacter(unichar c) {
	if ((c & 0xFFFE) == 0xFFFE) // 0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF ... 0x10FFFF
		return false;
	if (0xFDD0 <= c && c <= 0xFDEF) // continuous range of 32 non-characters in the BMP
		return false;
	return true;
}

/// Lol!
struct Whadup {};


const utf8char* fullCaseFold(const utf8char* c);
const utf8char* simpleCaseFold(const utf8char* c);

} // namespace unicode
} // namespace llhd
