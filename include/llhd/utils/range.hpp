/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <iterator>

/// \file
/// Defines classes and functions that allows for easy iteration over ranges as
/// defined by two iterators or other means.

namespace llhd {

/// A range as defined by two iterators [first, last). May be used to group a
/// pair of iterators that is easy to pass around and may be used in functions
/// that expect a container, e.g. for (auto v : Range<...>()).
template<class Iterator>
class Range {
	Iterator first;
	Iterator last;

public:
	Range(Iterator f, Iterator l): first(f), last(l) {}
	template <class Other> Range(const Range<Other> &r): first(r.begin()), last(r.end()) {}

	Iterator begin() const { return first; }
	Iterator end() const { return last; }
};

/// Returns a Range object for the range [\a first, \a last) spanned by the
/// iterators. This is a mere convenience function.
template<class Iterator>
Range<Iterator> make_range(Iterator first, Iterator last) {
	return Range<Iterator>(first, last);
}

/// Returns a Range object that covers the given null-terminated string.
template<typename String>
Range<String*> make_range(String *str) {
	String *end = str;
	while (*end)
		++end;
	return Range<String*>(str, end);
}

/// Returns a Range object spanning a mutable container.
template<typename Container>
Range<typename Container::iterator> make_range(Container &container) {
	return Range<typename Container::iterator>(std::begin(container), std::end(container));
}

/// Returns a Range object spanning an immutable container.
template<typename Container>
Range<typename Container::const_iterator> make_range(const Container &container) {
	return Range<typename Container::const_iterator>(std::begin(container), std::end(container));
}

} // namespace llhd
