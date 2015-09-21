/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include <iterator>

/// \file
/// Extensions to the standard library's <iterator> header.

namespace llhd {


/// \needsdoc
template<class Iterator, class Mapping>
class MappingIterator : public Iterator {
public:
	typedef MappingIterator self;
	typedef Iterator iterator_type;
	typedef Mapping mapping_type;
	typedef typename Mapping::result_type result_type;

	explicit MappingIterator(Mapping m): mapping(m) {}
	explicit MappingIterator(Iterator it): Iterator(it) {}
	MappingIterator(Iterator it, Mapping m): Iterator(it), mapping(m) {}

	result_type operator*() const { return mapping(Iterator::operator*()); }
	result_type operator->() const { return mapping(Iterator::operator->()); }

private:
	mapping_type mapping;
};


/// \needsdoc
template<class T>
struct dereference {
	typedef T argument_type;
	typedef decltype(*T(nullptr)) result_type;

	result_type operator() (T const& x) const { return *x; }
};


/// \needsdoc
template<class Iterator>
using DereferencingIterator = MappingIterator<Iterator, dereference<
	typename std::remove_reference<decltype(*Iterator())>::type>
>;


} // namespace llhd
