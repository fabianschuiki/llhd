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

	/// Set to true as soon as a fatal diagnostic is added.
	bool fatal;

public:
	/// Allocator that provides garbage collected memory for Diagnostic objects.
	/// May also be used for other things that ought to be deallocated when this
	/// DiagnosticContext is destroyed.
	PoolAllocator<> alloc;

	DiagnosticContext() {
		fatal = false;
	}

	void addDiagnostic(const Diagnostic* diag) {
		diagnostics.push_back(diag);
		if (diag->isFatal())
			fatal = true;
	}

	/// Returns the range of diagnostics attached to this context.
	Range<std::vector<const Diagnostic*>::const_iterator> getDiagnostics() const {
		return range(diagnostics.begin(), diagnostics.end());
	}

	/// Returns the number of diagnostics.
	unsigned getNumDiagnostics() const {
		return diagnostics.size();
	}

	/// Returns true if this context contains a fatal error diagnostic.
	bool isFatal() const { return fatal; }
};

} // namespace llhd
