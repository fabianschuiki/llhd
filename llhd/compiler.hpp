/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

// provide assertions
#include <cassert>

// Expands to an expression that is true if the given 'derived' class overrides
// the function 'fn', which returns 'rettype' and takes arguments 'type'. The
// expression is constant, allowing modern compilers to statically assert its
// value. Handy if used inside static_assert.
#define LLHD_DERIVED_CLASS_OVERRIDES(base, derived, fn, rettype, type)\
	static_cast<rettype (base::*)type>(&base::fn) !=\
	static_cast<rettype (derived::*)type>(&derived::fn)

namespace llhd {

/// Checks whether \a value is a power of 2. Zero is not considered a power of
/// 2, even if it is in the mathematical sense. The implementation exploits the
/// fact that in binary, powers of two have one bit set to 1, and all others
/// zero. Therefore it suffices to check whether all but the most significant
/// bit of the number are zero.
template <typename T> inline bool isPowerOf2(T value) {
	// Example for power-of-two:
	//   value = 8 = '1000'
	//   value-1 = 7 = '0111'
	//   value & (value-1) = '0000'
	// Example for non-power-of-two:
	//   value = 10 = '1010'
	//   value-1 = 9 = '1001'
	//   value & (value-1) = '1000'
	return value && !(value & (value - T(1)));
}

/// Aligns \a ptr to \a alignment bytes. The alignment always rounds up, i.e.
/// aligned ptr >= original ptr. \a alignment must be a power of 2.
inline char* alignPtr(char* ptr, size_t alignment) {
	assert(isPowerOf2(alignment) && "Alignment is not a power of 2!");
	return (char*)((size_t(ptr) + alignment - 1) & ~(alignment - 1));
}

template <typename T> class alignOf {

	/// A helper that places T at an annoying location in a struct. This will
	/// cause the compiler to insert padding bytes between \c prefix and \c t.
	/// The amount of padding may then be estimated as the difference between
	/// the size of \c prefixed and the the size of the original \c T.
	struct prefixed {
		char prefix;
		T t;
	};

public:
	enum { alignment = static_cast<unsigned>(sizeof(prefixed) - sizeof(T)) };

	enum { geq2  = alignment >= 2  ? 1 : 0 };
	enum { geq4  = alignment >= 4  ? 1 : 0 };
	enum { geq8  = alignment >= 8  ? 1 : 0 };
	enum { geq16 = alignment >= 16 ? 1 : 0 };

	enum { leq2  = alignment <= 2  ? 1 : 0 };
	enum { leq4  = alignment <= 4  ? 1 : 0 };
	enum { leq8  = alignment <= 8  ? 1 : 0 };
	enum { leq16 = alignment <= 16 ? 1 : 0 };
};

} // namespace llhd
