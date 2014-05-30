/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"

namespace llhd {
namespace unicode {

namespace utf32 {
	struct full {
		static const uint32_t nodes[];
		static const uint32_t leaves[];
	};
	struct simple {
		static const uint32_t nodes[];
		static const uint32_t leaves[];
	};
} // namespace utf32

namespace utf16 {
	struct full {
		static const uint32_t nodes[];
		static const uint16_t leaves[];
	};
	struct simple {
		static const uint32_t nodes[];
		static const uint16_t leaves[];
	};
} // namespace utf16

namespace utf8 {
	struct full {
		static const uint32_t nodes[];
		static const uint8_t leaves[];
	};
	struct simple {
		static const uint32_t nodes[];
		static const uint8_t leaves[];
	};
} // namespace utf8

} // namespace unicode
} // namespace llhd
