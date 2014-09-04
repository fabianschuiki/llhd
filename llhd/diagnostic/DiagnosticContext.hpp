/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/NullTerminatedIterator.hpp"
#include "llhd/range.hpp"
#include "llhd/allocator/PoolAllocator.hpp"
#include "llhd/diagnostic/Diagnostic.hpp"
#include <vector>

namespace llhd {

/// A container for diagnostic messages to be shown to the user.
class DiagnosticContext {
	/// Sequence of diagnostics, stored as pointers into memory provided by the
	/// allocator.
	std::vector<const Diagnostic*> diagnostics;

	unsigned numFatal;
	unsigned numError;
	unsigned numWarning;
	unsigned numNote;
	unsigned numFixit;

public:
	/// Allocator that provides garbage collected memory for Diagnostic objects.
	/// May also be used for other things that ought to be deallocated when this
	/// DiagnosticContext is destroyed.
	PoolAllocator<> alloc;

	DiagnosticContext():
		numFatal(0),
		numError(0),
		numWarning(0),
		numNote(0),
		numFixit(0) {}

	void addDiagnostic(const Diagnostic* diag) {
		diagnostics.push_back(diag);
		numFatal += diag->getNumFatal();
		numError += diag->getNumError();
		numWarning += diag->getNumWarning();
		numNote += diag->getNumNote();
		numFixit += diag->getNumFixit();
	}

	/// Returns the range of diagnostics attached to this context.
	Range<std::vector<const Diagnostic*>::const_iterator> getDiagnostics() const {
		return range(diagnostics.begin(), diagnostics.end());
	}

	/// Returns the number of diagnostics.
	unsigned getNumDiagnostics() const {
		return diagnostics.size();
	}

	/// Returns true if the context contains a fatal error.
	bool isFatalSeverity() const {
		return numFatal > 0; }
	/// Returns true if the context contains an error or a fatal error.
	bool isErrorSeverity() const {
		return isFatalSeverity() || numError > 0; }
	/// Returns true if the context contains a warning, an error, or a fatal
	/// error.
	bool isWarningSeverity() const {
		return isErrorSeverity() || numWarning > 0; }

	unsigned getNumFatal() const { return numFatal; }
	unsigned getNumError() const { return numError; }
	unsigned getNumWarning() const { return numWarning; }
	unsigned getNumNote() const { return numNote; }
	unsigned getNumFixit() const { return numFixit; }
};

} // namespace llhd
