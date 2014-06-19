/* Copyright (c) 2014 Fabian Schuiki */
/* Unit tests for llhd::SourceManager. */

#define BOOST_TEST_MODULE sourcemanager
#include "llhd/SourceManager.hpp"
#include "llhd/SourceBuffer.hpp"
#include <boost/test/unit_test.hpp>
using namespace llhd;

BOOST_AUTO_TEST_CASE(locations_to_fileids) {
	// Register two file buffers.
	SourceManager mgr;
	FileId f1 = mgr.addBuffer(SourceBuffer((const utf8char*)"hel\nlo"), "f1");
	FileId f2 = mgr.addBuffer(SourceBuffer((const utf8char*)"wor\nld"), "f2");
	BOOST_CHECK_EQUAL(f1.getId(), 1);
	BOOST_CHECK_EQUAL(f2.getId(), 2);

	// Obtain source locations for the two.
	SourceLocation ls1 = mgr.getStartLocation(f1), le1 = mgr.getEndLocation(f1);
	SourceLocation ls2 = mgr.getStartLocation(f2), le2 = mgr.getEndLocation(f2);
	BOOST_CHECK_EQUAL(ls1.getId(), 0);
	BOOST_CHECK_EQUAL(le1.getId(), 6);
	BOOST_CHECK_EQUAL(ls2.getId(), 7);
	BOOST_CHECK_EQUAL(le2.getId(), 13);

	// Convert to presumed locations.
	PresumedLocation ps1 = mgr.getPresumedLocation(ls1);
	PresumedLocation pe1 = mgr.getPresumedLocation(le1);
	PresumedLocation ps2 = mgr.getPresumedLocation(ls2);
	PresumedLocation pe2 = mgr.getPresumedLocation(le2);

	BOOST_CHECK_EQUAL(ps1.fid, f1);
	BOOST_CHECK_EQUAL(pe1.fid, f1);
	BOOST_CHECK_EQUAL(ps2.fid, f2);
	BOOST_CHECK_EQUAL(pe2.fid, f2);

	BOOST_CHECK_EQUAL(ps1.line, 1); BOOST_CHECK_EQUAL(ps1.column, 1);
	BOOST_CHECK_EQUAL(pe1.line, 2); BOOST_CHECK_EQUAL(pe1.column, 3);
	BOOST_CHECK_EQUAL(ps2.line, 1); BOOST_CHECK_EQUAL(ps2.column, 1);
	BOOST_CHECK_EQUAL(pe2.line, 2); BOOST_CHECK_EQUAL(pe2.column, 3);
}
