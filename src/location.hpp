/* Copyright (c) 2014-2015 Fabian Schuiki */
#pragma once
#include "llhd/types.hpp"
#include "llhd/utils/assert.hpp"
#include <ostream>

/// \file
/// This file establishes facilities that deal with uniquely identifying source
/// files and tracking locations across multiple source files. A key design goal
/// is to make these structures as efficient as possible, as they are used all
/// throughout the code base.

namespace llhd {

class SourceId;
class SourceLocation;
class SourceRange;
class PerceivedLocation;
class PerceivedRange;

/// A unique identifier for a source file. Each processed source file shall be
/// assigned a SourceId to keep track of locations within the file.
class SourceId {
	/// 0 indicates "invalid", everything else refers to a source file.
	uint32_t id;

public:
	/// Creates an invalid source id.
	SourceId(): id(0) {}
	/// Creates a source id with the given \a id.
	explicit SourceId(uint32_t id): id(id) {}

	/// Returns true if this is a valid source id.
	bool is_valid() const { return id != 0; }
	/// Returns true if this is a valid source id.
	explicit operator bool() const { return is_valid(); }

	bool operator==(SourceId l) const { return id == l.id; }
	bool operator!=(SourceId l) const { return id != l.id; }
	bool operator<=(SourceId l) const { return id <= l.id; }
	bool operator>=(SourceId l) const { return id >= l.id; }
	bool operator< (SourceId l) const { return id <  l.id; }
	bool operator> (SourceId l) const { return id >  l.id; }

	/// Returns an opaque ID describing this file ID.
	uint32_t get_id() const { return id; }
};


/// A location within a source file. May be used to track locations across
/// multiple files.
class SourceLocation {
	friend class SourceRange;

	/// Source file this location refers to.
	SourceId sid;
	/// Offset from the start of the source file.
	uint32_t pos;

public:
	/// Creates an invalid source location.
	SourceLocation(): pos(0) {}
	/// Creats a location pointing to the beginning of the source file \a sid.
	explicit SourceLocation(SourceId sid): sid(sid), pos(0) {}
	/// Creates a location pointing to the offset \a pos in the source file \a
	/// sid.
	SourceLocation(SourceId sid, uint32_t pos): sid(sid), pos(pos) {}

	/// Returns true if this is a valid source location.
	bool is_valid() const { return sid.is_valid(); }
	/// Returns true if this is a valid source location.
	explicit operator bool() const { return is_valid(); }

	/// Returns a new location shifted i positions away from the beginning.
	SourceLocation operator+(int i) const { return SourceLocation(sid, pos+i); }
	/// Returns a new location shifted i positions towards the beginning.
	SourceLocation operator-(int i) const { return SourceLocation(sid, pos-i); }
	/// Shifts the location i positions away from the beginning.
	SourceLocation& operator+=(int i) { pos += i; return *this; }
	/// Shifts the location i positions towards the beginning.
	SourceLocation& operator-=(int i) { pos -= i; return *this; }

	bool operator==(SourceLocation l) const {
		return sid == l.sid && pos == l.pos;
	}
	bool operator!=(SourceLocation l) const {
		return sid != l.sid || pos != l.pos;
	}
	bool operator<=(SourceLocation l) const {
		return sid < l.sid || (sid == l.sid && pos <= l.pos);
	}
	bool operator>=(SourceLocation l) const {
		return sid > l.sid || (sid == l.sid && pos >= l.pos);
	}
	bool operator<(SourceLocation l) const {
		return sid < l.sid || (sid == l.sid && pos < l.pos);
	}
	bool operator>(SourceLocation l) const {
		return sid > l.sid || (sid == l.sid && pos > l.pos);
	}

	/// Returns the source ID of the file this location refers to.
	SourceId get_source_id() const { return sid; }
	/// Returns the offset from the beginning of the source file.
	uint32_t get_offset() const { return pos; }
};


/// A range within a single source file. May not reach across file boundaries,
/// as this usually does not make sense.
class SourceRange {
	/// Source file this range refers to.
	SourceId sid;
	/// Offset from the start of the source file where the range begins.
	uint32_t pos;
	/// Length of the range.
	uint32_t len;

public:
	/// Creates an invalid source range.
	SourceRange(): pos(0), len(0) {}
	/// Creates a source range starting at offset \a pos from the beginning of
	/// source file \a sid, ending after a \a len positions.
	SourceRange(SourceId sid, uint32_t pos, uint32_t len):
		sid(sid),
		pos(pos),
		len(len) {}
	/// Creates a source range starting at \a a and ending just before \a b,
	/// i.e. [a,b). \a a and \a b must refer to the same source file, and \a a
	/// must lie before \a b.
	SourceRange(SourceLocation a, SourceLocation b):
		sid(a.sid),
		pos(a.pos),
		len(b.pos-a.pos) {
		llhd_assert(a.sid == b.sid);
		llhd_assert(a.pos <= b.pos);
	}

	/// Returns true if this is a valid source range.
	bool is_valid() const { return sid.is_valid(); }
	/// Returns true if this is a valid source range.
	explicit operator bool() const { return is_valid(); }

	bool operator==(SourceRange r) const {
		return sid == r.sid && pos == r.pos && len == r.len;
	}
	bool operator!=(SourceRange r) const {
		return sid != r.sid || pos != r.pos || len != r.len;
	}

	/// Returns the source ID of the file this location refers to.
	SourceId get_source_id() const { return sid; }
	/// Returns the offset from the begining of the source file where this range
	/// starts.
	uint32_t get_offset() const { return pos; }
	/// Returns the length of this range.
	uint32_t get_length() const { return len; }
	/// Returns the source location at the beginning of this range.
	SourceLocation get_begin() const { return SourceLocation(sid, pos); }
	/// Returns the source location just after the end of this range.
	SourceLocation get_end() const { return SourceLocation(sid, pos+len); }

	/// Returns true if the location \a l lies within this range.
	bool contains(SourceLocation l) const {
		return sid == l.sid && pos <= l.pos && pos+len > l.pos;
	}
	/// Returns true if the range \a r is fully enclosed by this range.
	bool contains(SourceRange r) const {
		return sid == r.sid && pos <= r.pos && pos+len >= r.pos+r.len;
	}
	/// Returns true if the given offset \a p lies within this range, assuming
	/// that \a p lies in the same source file as this range.
	bool contains(uint32_t p) const { return pos <= p && pos+len > p; }

	/// Returns true if the range \a r intersects with this range.
	bool intersects(SourceRange r) const {
		return sid == r.sid && pos < r.pos+r.len && pos+len > r.pos;
	}
	/// Returns true if the range \a r touches this range. This means the two
	/// ranges either intersect, or one immediately follows the other.
	bool touches(SourceRange r) const {
		return sid == r.sid && pos <= r.pos+r.len && pos+len >= r.pos;
	}
};


/// \needsdoc
class PerceivedLocation {
	friend class PerceivedRange;

	SourceId sid;
	uint32_t line   = 0;
	uint32_t column = 0;

public:
	PerceivedLocation() {}
	PerceivedLocation(SourceId sid, uint32_t line, uint32_t column):
		sid(sid),
		line(line),
		column(column) {}

	SourceId get_source_id() const { return sid; }
	uint32_t get_line() const { return line; }
	uint32_t get_column() const { return column; }

	bool is_valid() const { return sid.is_valid(); }
	explicit operator bool() const { return is_valid(); }

	bool operator==(PerceivedLocation l) const {
		return line == l.line && column == l.column;
	}
	bool operator!=(PerceivedLocation l) const {
		return line != l.line || column != l.column;
	}
	bool operator<=(PerceivedLocation l) const {
		return line < l.line || (line == l.line && column <= l.column);
	}
	bool operator>=(PerceivedLocation l) const {
		return line > l.line || (line == l.line && column >= l.column);
	}
	bool operator<(PerceivedLocation l) const {
		return line < l.line || (line == l.line && column < l.column);
	}
	bool operator>(PerceivedLocation l) const {
		return line > l.line || (line == l.line && column > l.column);
	}
};


/// \needsdoc
class PerceivedRange {
	SourceId sid;
	uint32_t first_line   = 0;
	uint32_t first_column = 0;
	uint32_t last_line    = 0;
	uint32_t last_column  = 0;

public:
	PerceivedRange() {}

	PerceivedRange(
		SourceId sid,
		uint32_t first_line,
		uint32_t first_column,
		uint32_t last_line,
		uint32_t last_column):
		sid(sid),
		first_line(first_line),
		first_column(first_column),
		last_line(last_line),
		last_column(last_column) {}

	PerceivedRange(PerceivedLocation a, PerceivedLocation b):
		sid(a.sid),
		first_line(a.line),
		first_column(a.column),
		last_line(b.line),
		last_column(b.column) {
		llhd_assert(a.sid == b.sid);
		llhd_assert(a <= b);
	}

	SourceId get_source_id() const { return sid; }
	uint32_t get_first_line() const { return first_line; }
	uint32_t get_first_column() const { return first_column; }
	uint32_t get_last_line() const { return last_line; }
	uint32_t get_last_column() const { return last_column; }

	PerceivedLocation get_begin() const {
		return PerceivedLocation(sid, first_line, first_column);
	}
	PerceivedLocation get_end() const {
		return PerceivedLocation(sid, last_line, last_column);
	}

	bool is_valid() const { return sid.is_valid(); }
	explicit operator bool() const { return is_valid(); }

	bool operator==(PerceivedRange r) const {
		return sid == r.sid &&
			first_line == r.first_line && first_column == r.first_column &&
			last_line  == r.last_line  && last_column  == r.last_column;
	}
	bool operator!=(PerceivedRange r) const {
		return sid != r.sid ||
			first_line != r.first_line || first_column != r.first_column ||
			last_line  != r.last_line  || last_column  != r.last_column;
	}
};


/// \needsdoc
template<typename... Args>
SourceRange union_range(SourceRange a, SourceRange b, Args... rest) {
	auto c = union_range(b, rest...);
	auto first = std::min(a.get_begin(), c.get_begin());
	auto last  = std::max(a.get_end(), c.get_end());
	return SourceRange(first,last);
}

/// \needsdoc
inline SourceRange union_range(SourceRange a) {
	return a;
}


inline std::ostream& operator<< (std::ostream& o, SourceId id) {
	o << id.get_id();
	return o;
}

inline std::ostream& operator<< (std::ostream& o, SourceLocation l) {
	o << l.get_source_id() << ':' << l.get_offset();
	return o;
}

inline std::ostream& operator<< (std::ostream& o, SourceRange r) {
	auto off = r.get_offset();
	auto len = r.get_length();
	o << r.get_source_id() << ':' << off << '-' << (off+len);
	return o;
}


inline std::ostream& operator<< (std::ostream& o, PerceivedLocation l) {
	o << l.get_line() << ':' << l.get_column();
	return o;
}

inline std::ostream& operator<< (std::ostream& o, PerceivedRange r) {
	o << r.get_source_id()  << ':' <<
	     r.get_first_line() << '.' << r.get_first_column() << '-' <<
	     r.get_last_line()  << '.' << r.get_last_column();
	return o;
}


} // namespace llhd
