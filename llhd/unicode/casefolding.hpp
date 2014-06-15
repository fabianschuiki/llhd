/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/unicode/unichar.hpp"
#include <iterator>

namespace llhd {
namespace unicode {

/// Looks up the casefolded equivalent for the first character in \a c.
///
/// \tparam T  Character type of the string to be casefolded. Should be
///         \ref utf8char, \ref utf16char, or \ref utf32char.
/// \tparam full  Whether to perform full (possibly changing the string length)
///         or simple casefolding (leaving string length constant).
///
/// \param  c  String to be casefolded. Should be an array of \ref utf8char,
///         \ref utf16char, or \ref utf32char.
/// \param  shift  Set to the width of the skipped character in \a c if
///         casefolding occurs. May be NULL.
/// \return The same pointer as \a c if no casefolding is necessary, or a
///         pointer to a null-terminated casefolded replacement string.
template<typename T, bool full = true>
const T* casefold(const T* c, unsigned* shift = 0);

/// Forward iterator that performs unicode casefolding. Note that the iterator
/// steps through the elements of the string, not the unicode code points
/// represented in the string. For example, if you iterator over a UTF-8 string,
/// the iterator steps through each byte of the string individually which
/// frequently places the iterator in the middle of a multi-byte character.
template<typename T, bool full = true>
class casefold_iterator : public std::iterator<std::input_iterator_tag, T> {
	const T *base, *mapped;

	void lookup() {
		unsigned shift;
		const T* p = casefold<T,full>(base, &shift);
		if (p != base) {
			mapped = p;
			base += shift;
			if (*base == 0)
				base = 0;
		}
	}

public:
	/// Yields the end iterator.
	casefold_iterator(): base(0), mapped(0) {}
	/// Yields the begin iterator for string \a b.
	casefold_iterator(const T* b): base(b), mapped(0) { lookup(); }
	/// Copies the given iterator.
	casefold_iterator(const casefold_iterator& o):
		base(o.base),
		mapped(o.mapped) {}

	/// Advances the iterator to the next element of the string.
	casefold_iterator& operator++() {
		if (mapped != 0) {
			++mapped;
			if (*mapped == 0) {
				mapped = 0;
				if (base != 0)
					lookup();
			}
		} else {
			++base;
			if (*base == 0) {
				base = 0;
			} else {
				lookup();
			}
		}
		return *this;
	}

	/// Advances the iterator to the next element of the string.
	casefold_iterator operator++(int) {
		casefold_iterator tmp(*this);
		operator++();
		return tmp;
	}

	/// Checks whether two iterators are equal.
	bool operator==(const casefold_iterator& i) const {
		return base == i.base && mapped == i.mapped;
	}
	/// Checks whether two iterators are unequal.
	bool operator!=(const casefold_iterator& i) const {
		return base != i.base || mapped != i.mapped;
	}

	/// Returns the element of the string the iterator is currently at.
	/// \warning The result of dereferencing the end iterator or any iterator
	///          that stepped past the end iterator is undefined.
	T operator*() const {
		return mapped ? *mapped : *base;
	}
};

} // namespace unicode
} // namespace llhd
