/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/diagnostic/DiagnosticMessage.hpp"
#include <cassert>

namespace llhd {

/// A diagnostic message intended to be shown to the user. Usually refers to
/// some location in a source file.
class Diagnostic {
	const static unsigned maxMessages = 8;

	unsigned numFatal;
	unsigned numError;
	unsigned numWarning;
	unsigned numNote;
	unsigned numFixit;

	/// Messages attached to this diagnostic. The first element is always set
	/// and is treated as the "main" message.
	const DiagnosticMessage* messages[maxMessages];
	/// Number of messages attached to this diagnostic.
	unsigned numMessages;

public:
	Diagnostic():
		numFatal(0),
		numError(0),
		numWarning(0),
		numNote(0),
		numFixit(0),
		numMessages(0) {}

	/// Adds the message \a msg to this diagnostic.
	void addMessage(const DiagnosticMessage* msg) {
		assert(numMessages < maxMessages);
		messages[numMessages++] = msg;
		switch (msg->getType()) {
			case kFatal: numFatal++; break;
			case kError: numError++; break;
			case kWarning: numWarning++; break;
			case kNote: numNote++; break;
			case kFixit: numFixit++; break;
		}
	}

	/// Returns the message at \a index.
	const DiagnosticMessage* getMessage(unsigned index) const {
		assert(index < numMessages);
		return messages[index];
	}

	/// Returns the number of messages.
	unsigned getNumMessages() const {
		return numMessages;
	}

	unsigned getNumFatal() const { return numFatal; }
	unsigned getNumError() const { return numError; }
	unsigned getNumWarning() const { return numWarning; }
	unsigned getNumNote() const { return numNote; }
	unsigned getNumFixit() const { return numFixit; }
};

} // namespace llhd
