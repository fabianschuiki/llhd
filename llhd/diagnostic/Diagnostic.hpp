/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/diagnostic/DiagnosticMessage.hpp"
#include <cassert>

namespace llhd {

/// A diagnostic message intended to be shown to the user. Usually refers to
/// some location in a source file.
class Diagnostic {
	const static unsigned maxMessages = 8;

	/// Set to true as soon as a kFatal message is added to this diagnostic.
	bool fatal;

	/// Messages attached to this diagnostic. The first element is always set
	/// and is treated as the "main" message.
	const DiagnosticMessage* messages[maxMessages];
	/// Number of messages attached to this diagnostic.
	unsigned numMessages;

public:
	/// Adds the message \a msg to this diagnostic.
	void addMessage(const DiagnosticMessage* msg) {
		assert(numMessages < maxMessages);
		messages[numMessages++] = msg;
		if (msg->getType() == kFatal)
			fatal = true;
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

	/// Returns true if this diagnostic contains a fatal error message.
	bool isFatal() const { return fatal; }
};

} // namespace llhd
