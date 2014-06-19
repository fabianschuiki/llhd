/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include <ostream>
#include <string>

/// @file
/// Declares classes and tools that may be used to describe the location of
/// something in a source file.

namespace llhd {

class SourceManager;


/// An opaque identifier that refers to a source file.
class FileId {
	friend class SourceManager;

	/// 0 is "invalid", everything else refers to a source file.
	unsigned id;

	/// Creates a FileId with the given \a id.
	explicit FileId(unsigned id): id(id) {}

public:
	FileId(): id(0) {}

	bool isValid() const { return id != 0; }
	bool operator== (const FileId& rhs) const { return id == rhs.id; }
	bool operator!= (const FileId& rhs) const { return id != rhs.id; }
	bool operator<  (const FileId& rhs) const { return id <  rhs.id; }
	bool operator>  (const FileId& rhs) const { return id >  rhs.id; }
	bool operator<= (const FileId& rhs) const { return id <= rhs.id; }
	bool operator>= (const FileId& rhs) const { return id >= rhs.id; }

	/// Returns an opaque ID describing this file ID.
	unsigned getId() const { return id; }
};


/// An opaque location that points to a source file and location therein. The
/// \c id corresponds to an offset into the corresponding SourceManager's
/// continuous source space.
class SourceLocation {
	friend class SourceManager;

	/// 0 is "invalid", everything else refers to a precise location in the
	/// corresponding SourceManager's continuous source space.
	unsigned id;

	/// Creates a source location with the given \a id.
	explicit SourceLocation(unsigned id): id(id) {}

public:
	/// Creates an invalid source location.
	SourceLocation(): id(0) {}

	/// Returns true if this is a valid SourceLocation.
	bool isValid() const { return id != 0; }

	/// Returns another location which is offset by \a offset.
	SourceLocation operator+ (int offset) const { return SourceLocation(id + offset); }

	/// Offsets this location by \a offset.
	SourceLocation& operator+= (int offset) { id += offset; return *this; }

	/// Returns an opaque ID describing this location.
	unsigned getId() const { return id; }
};


/// An opaque range that points at a portion of a source file. It consists of
/// two SourceLocation objects, pointing to the first and just beyond the last
/// byte in the range.
struct SourceRange {
	/// Location at the beginning this range.
	SourceLocation s;
	/// Location just after the last character in this range.
	SourceLocation e;

	/// Creates an invalid source range, consisting of two invalid locations.
	SourceRange() {}
	/// Creates a range from location \a s to location \a e.
	SourceRange(SourceLocation s, SourceLocation e): s(s), e(e) {}
	/// Creates a range from location \a s over the next \a l characters.
	SourceRange(SourceLocation s, unsigned l): s(s), e(s+l) {}

	/// Returns true if this is a valid SourceRange.
	bool isValid() const { return s.isValid() && e.isValid(); }
};


/// A decoded SourceLocation, presentable to humans. The SourceManager resolves
/// SourceLocation objects to PresumedLocation objects, filling in the filename,
/// offset, line, and column. See SourceManager::getPresumedLocation() for more
/// details.
struct PresumedLocation {
	FileId fid;
	unsigned offset;
	unsigned line;
	unsigned column;

	PresumedLocation():
		offset(0),
		line(0),
		column(0) {}

	/// Returns true if this is a valid PresumedLocation.
	bool isValid() const { return fid.isValid(); }
};

/// A decoded SourceRange, presentable to humans. Actually consists of two
/// PresumedLocation objects. The SourceManager resolves SourceRange objects to
/// PresumedRange objects. See SourceManager::getPresumedRange() for more
/// details.
struct PresumedRange {
	/// Location at the beinnging of this range.
	PresumedLocation s;
	/// Location just after the last character in this range.
	PresumedLocation e;

	/// Creates an invalid presumed range, consisting of two invalid locations.
	PresumedRange() {}
	/// Creates a presumed range from location \a s to \a e.
	PresumedRange(PresumedLocation s, PresumedLocation e): s(s), e(e) {}

	/// Returns true if this is a vlid PresumedLocation.
	bool isValid() const { return s.isValid() && e.isValid(); }
};

// Formatting functions.
std::ostream& operator<<(std::ostream& o, FileId fid);
std::ostream& operator<<(std::ostream& o, SourceLocation loc);
std::ostream& operator<<(std::ostream& o, SourceRange rng);

std::ostream& operator<<(std::ostream& o, PresumedLocation loc);
std::ostream& operator<<(std::ostream& o, PresumedRange rng);


} // namespace llhd
