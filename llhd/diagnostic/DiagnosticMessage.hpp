/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/range.hpp"
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
	unsigned numArgs;
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
		numArgs(0),
		numHighlighted(0),
		numRelevant(0) {}

	DiagnosticType getType() const {
		return type;
	}

	const char* getMessage() const {
		return message;
	}

	const DiagnosticMessageArgument& getArgument(unsigned idx) const {
		assert(idx < numArgs);
		return args[idx];
	}

	template<typename T>
	void addArgument(T v) {
		assert(numArgs < maxArgs);
		args[numArgs++] = v;
	}

	Range<const DiagnosticMessageArgument*> getArguments() const {
		return range(args, args+numArgs);
	}

	void setMainRange(SourceRange r) { mainRange = r; }
	SourceRange getMainRange() const { return mainRange; }

	void addHighlightedRange(SourceRange r) {
		assert(numHighlighted < maxRanges);
		highlightedRanges[numHighlighted++] = r;
	}

	Range<const SourceRange*> getHighlightedRanges() const {
		return range(highlightedRanges, highlightedRanges+numHighlighted);
	}

	void addRelevantRange(SourceRange r) {
		assert(numRelevant < maxRanges);
		relevantRanges[numRelevant++] = r;
	}

	Range<const SourceRange*> getRelevantRanges() const {
		return range(relevantRanges, relevantRanges+numRelevant);
	}
};

} // namespace llhd
