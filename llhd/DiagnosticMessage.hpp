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

public:
	DiagnosticMessage(DiagnosticType t, const char* msg):
		type(t),
		message(msg) {}

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
};

} // namespace llhd
