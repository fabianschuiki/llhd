/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/compiler.hpp"

namespace llhd {
namespace unicode {

typedef uint8_t  utf8char;
typedef uint16_t utf16char;
typedef uint32_t utf32char;

const utf8char* fullCaseFold(const utf8char* c);
const utf8char* simpleCaseFold(const utf8char* c);

} // namespace unicode
} // namespace llhd
