/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/NullTerminatedIterator.hpp"
#include "llhd/allocator/PoolAllocator.hpp"
#include <vector>

namespace llhd {

/// A container for diagnostic messages to be shown to the user.
class DiagnosticContext {
	/// Sequence of diagnostics, stored as pointers into memory provided by the
	/// allocator.
	std::vector<const Diagnostic*> diagnostics;

public:
	/// Allocator that provides garbage collected memory for Diagnostic objects.
	/// May also be used for other things that ought to be deallocated when this
	/// DiagnosticContext is destroyed.
	PoolAllocator<> alloc;

	void addDiagnostic(const Diagnostic* diag) {
		diagnostics.push_back(diag);
	}

	/// Returns the diagnostic at \a index.
	const Diagnostic* getDiagnostic(unsigned index) const {
		assert(index < diagnostics.size());
		return diagnostics[index];
	}

	/// Returns the number of diagnostics.
	unsigned getNumDiagnostics() const {
		return diagnostics.size();
	}
};

} // namespace llhd
