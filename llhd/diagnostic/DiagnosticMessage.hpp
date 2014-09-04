/* Copyright (c) 2014 Fabian Schuiki */
/// \file
#pragma once
#include "llhd/range.hpp"
#include "llhd/SourceLocation.hpp"
#include <cassert>

namespace llhd {

/// Types of diagnostic messages. Sorted according to severity.
enum DiagnosticType {
	/// Processing needs to abort as soon as possible. Useful if an error is so
	/// severe that it doesn't make sense to even finish the current processing
	/// step. E.g. if a file cannot be found.
	kFatal,
	/// Processing needs to abort after the current stage. Useful if the issue
	/// prevents further processing stages from working properly, but can be
	/// overlooked in the current stage in order to finish processing the input.
	/// E.g. a syntax error like missing paranthesis.
	kError,
	/// Processing may continue, but the user is advised to review the issue.
	/// Useful for syntax errors that can be compensated, e.g. an unknown input
	/// character which can be ignored.
	kWarning,
	/// A note further detailing one of the above issues. May also be used as a
	/// standalone annotation for parts of the input.
	kNote,
	/// Contains a hint on how to automatically fix the issue at hand. Use this
	/// only when there is a clear, apparent solution to an issue, e.g. a
	/// semicolon missing in an obvious place.
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

	DiagnosticMessage(SourceRange mr, DiagnosticType t, const char* msg):
		DiagnosticMessage(t, msg) {
		mainRange = mr;
	}

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
