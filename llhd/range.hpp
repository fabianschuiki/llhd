/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

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

	Iterator begin() const { return first; }
	Iterator end() const { return last; }
};

/// Returns a Range object for the range [\a first, \a last) spanned by the
/// iterators. This is a mere convenience function.
template<class Iterator>
Range<Iterator> range(Iterator first, Iterator last) {
	return Range<Iterator>(first, last);
}

} // namespace llhd
