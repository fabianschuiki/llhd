/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <iterator>

namespace llhd {

/// Iterator over a range of memory whose end reads as 0. A convenience iterator
/// that is robust against iterating past the end of the range it is assigned.
/// When dereferenced with the \ref operator*(), returns the memory location the
/// iterator is at, or 0 if it is at or past its end. Can be used to write code
/// in a more compact manner since null-termination can be assumed.
template<typename T>
class NullTerminatedIterator :
	public std::iterator<std::input_iterator_tag, T> {

	const T* pos;
	const T* end;

	inline bool isEndIterator() const { return pos == 0 && end == 0; }

public:
	NullTerminatedIterator():
		pos(0),
		end(0) {}
	NullTerminatedIterator(const T* first, const T* last):
		pos(first),
		end(last) {}
	NullTerminatedIterator(const NullTerminatedIterator& i):
		pos(i.pos),
		end(i.end) {}

	/// Checks whether the iterator is at the end of its range.
	bool isAtEnd() const {
		return pos >= end;
	}

	/// Advances the iterator to the next element.
	NullTerminatedIterator& operator++() {
		pos++;
		return *this;
	}

	/// Advances the iterator to the next element.
	NullTerminatedIterator operator++(int) {
		NullTerminatedIterator i(*this);
		pos++;
		return i;
	}

	/// Checks whether two iterators are equal.
	bool operator==(const NullTerminatedIterator& i) const {
		if (i.isEndIterator())
			return isAtEnd();
		return pos == i.pos && end == i.end;
	}
	/// Checks whether two iterators are unequal.
	bool operator!=(const NullTerminatedIterator& i) const {
		if (i.isEndIterator())
			return !isAtEnd();
		return pos != i.pos || end != i.end;
	}

	/// Returns the element the iterator is currently at. If the iterator is at
	/// the end of its range (i.e. \ref isAtEnd() returns true), 0 is returned.
	T operator*() const {
		return isAtEnd() ? T(0) : *pos;
	}

	/// Returns the memory location the iterator is currently at. If the
	/// iterator is at the end of its range (i.e. \ref isAtEnd() returns true),
	/// 0 is returned.
	operator const T*() const {
		return isAtEnd() ? 0 : pos;
	}
};

} // namespace llhd
