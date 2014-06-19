/* Copyright (c) 2014 Fabian Schuiki */
#include "SourceLocation.hpp"

namespace llhd {

std::ostream& operator<<(std::ostream& o, FileId fid) {
	o << '#' << fid.getId();
	return o;
}

std::ostream& operator<<(std::ostream& o, SourceLocation loc) {
	o << '$' << loc.getId();
	return o;
}

std::ostream& operator<<(std::ostream& o, SourceRange rng) {
	o << '$' << rng.s.getId() << '-' << rng.e.getId();
	return o;
}


std::ostream& operator<<(std::ostream& o, PresumedLocation loc) {
	o << loc.line << '.' << loc.column;
	return o;
}

std::ostream& operator<<(std::ostream& o, PresumedRange rng) {
	o << rng.s.line << '.' << rng.s.column;
	if (rng.s.line != rng.e.line) {
		o << '-' << rng.e.line << '.' << rng.e.column;
	} else if (rng.s.column != rng.e.column) {
		o << '-' << rng.e.column;
	}
	return o;
}

} // namespace llhd
