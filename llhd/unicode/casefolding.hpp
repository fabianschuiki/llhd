/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/unicode/unichar.hpp"

namespace llhd {
namespace unicode {

struct casefolding {
	static const utf8char* full(const utf8char* c, unsigned* shift = 0);
	static const utf8char* simple(const utf8char* c, unsigned* shift = 0);

	static const utf16char* full(const utf16char* c, unsigned* shift = 0);
	static const utf16char* simple(const utf16char* c, unsigned* shift = 0);

	static const utf32char* full(const utf32char* c, unsigned* shift = 0);
	static const utf32char* simple(const utf32char* c, unsigned* shift = 0);
};

} // namespace unicode
} // namespace llhd
