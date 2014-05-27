/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

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
	SourceLocation operator+ (int offset) const {
		assert(offset > 0 || id > (unsigned)(-offset));
		return SourceLocation(id + offset); }

	/// Offsets this location by \a offset.
	SourceLocation& operator+= (int offset) {
		assert(offset > 0 || id > (unsigned)(-offset));
		id += offset;
		return *this;
	}
};


/// A decoded SourceLocation, presentable to humans. The SourceManager resolves
/// SourceLocation objects to PresumedLocation objects, filling in the filename,
/// offset, line, and column. See SourceManager::getPresumedLocation() for more
/// details.
struct PresumedLocation {
	const char* filename;
	unsigned offset;
	unsigned line;
	unsigned column;

	PresumedLocation():
		filename(NULL),
		offset(0),
		line(0),
		column(0) {}

	/// Returns true if this is a valid PresumedLocation.
	bool isValid() const { return filename != NULL; }
};


} // namespace llhd
