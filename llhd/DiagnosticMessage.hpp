/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"
#include <cassert>

namespace llhd {

enum DiagnosticType {
	kFatal,
	kError,
	kWarning,
	kNote,
	kFixit
};

struct DiagnosticMessageArgument {
	enum {
		kUndefined,
		kSignedInt,
		kUnsignedInt,
		kString,
		kSourceRange
	} type;

	union {
		signed i;
		unsigned u;
		const char* s;
		SourceRange r;
	};

	DiagnosticMessageArgument& operator=(signed v) {
		type = kSignedInt;
		i = v;
		return *this;
	}
	DiagnosticMessageArgument& operator=(unsigned v) {
		type = kUnsignedInt;
		u = v;
		return *this;
	}
	DiagnosticMessageArgument& operator=(const char* v) {
		type = kString;
		s = v;
		return *this;
	}
	DiagnosticMessageArgument& operator=(SourceRange v) {
		type = kSourceRange;
		r = v;
		return *this;
	}

	// otherwise clang whines about 'r' having a non-trivial constructor
	DiagnosticMessageArgument(): type(kUndefined) {}
};

class DiagnosticMessage {
	DiagnosticType type;
	const char* message;

	const static unsigned maxArgs = 16;
	DiagnosticMessageArgument args[maxArgs];

	const static unsigned maxRanges = 16;
	unsigned numHighlighted;
	unsigned numRelevant;
	SourceRange mainRange;
	SourceRange highlightedRanges[maxRanges];
	SourceRange relevantRanges[maxRanges];

public:
	DiagnosticMessage(DiagnosticType t, const char* msg):
		type(t),
		message(msg),
		numHighlighted(0),
		numRelevant(0) {}

	DiagnosticType getType() const {
		return type;
	}

	const char* getMessage() const {
		return message;
	}

	const DiagnosticMessageArgument& getArgument(unsigned idx) const {
		assert(idx < maxArgs);
		return args[idx];
	}

	template<typename T>
	void setArgument(unsigned idx, T v) {
		assert(idx < maxArgs);
		args[idx] = v;
	}

	void setMainRange(SourceRange r) { mainRange = r; }
	SourceRange getMainRange() const { return mainRange; }

	void addHighlightedRange(SourceRange r) {
		assert(numHighlighted < maxRanges);
		highlightedRanges[numHighlighted++] = r;
	}

	const SourceRange* beginHighlightedRanges() const {
		return highlightedRanges;
	}
	const SourceRange* endHighlightedRanges() const {
		return highlightedRanges + numHighlighted;
	}

	void addRelevantRange(SourceRange r) {
		assert(numRelevant < maxRanges);
		relevantRanges[numHighlighted++] = r;
	}

	const SourceRange* beginRelevantRanges() const {
		return relevantRanges;
	}
	const SourceRange* endRelevantRanges() const {
		return relevantRanges + numRelevant;
	}
};

} // namespace llhd
