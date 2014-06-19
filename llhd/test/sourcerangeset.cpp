/* Copyright (c) 2014 Fabian Schuiki */
/* Unit tests for llhd::SourceRangeSet. */

#define BOOST_TEST_MODULE sourcerangeset
#include "llhd/SourceRangeSet.hpp"
#include <boost/test/unit_test.hpp>
using namespace llhd;

BOOST_AUTO_TEST_CASE(fusion) {
	SourceLocation base;
	SourceRange r1(base, base+10);
	SourceRange r2(base+15, base+25);
	SourceRange r3(base+25, base+30);
	SourceRange r4(base+7, base+22);

	SourceRangeSet s;
	BOOST_CHECK_EQUAL(s.getSize(), 0);
	s.insert(r1);
	BOOST_CHECK_EQUAL(s.getSize(), 1);
	s.insert(r2);
	BOOST_CHECK_EQUAL(s.getSize(), 2);
	s.insert(r3);
	BOOST_CHECK_EQUAL(s.getSize(), 2);
	s.insert(r4);
	BOOST_CHECK_EQUAL(s.getSize(), 1);
}
