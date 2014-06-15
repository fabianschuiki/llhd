/* Copyright (c) 2014 Fabian Schuiki */
#pragma once

namespace llhd {

/// A diagnostic message intended to be shown to the user. Usually refers to
/// some location in a source file.
class Diagnostic {
	const DiagnosticMessage* messages;
	unsigned numMessages;

public:
	void addMessage(const DiagnosticMessage* msg);
};

} // namespace llhd
