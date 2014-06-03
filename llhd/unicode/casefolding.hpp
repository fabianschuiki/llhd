/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/unicode/unichar.hpp"

namespace llhd {
namespace unicode {

const utf8char* fullCaseFold(const utf8char* c);
const utf8char* simpleCaseFold(const utf8char* c);

} // namespace unicode
} // namespace llhd
