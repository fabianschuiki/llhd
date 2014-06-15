/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/allocator/PoolAllocator.hpp"

namespace llhd {

/// A container for diagnostic messages to be shown to the user.
class DiagnosticContext {
public:
	/// Allocator that provides garbage collected memory for Diagnostic objects.
	/// May also be used for other things that ought to be deallocated when this
	/// DiagnosticContext is destroyed.
	PoolAllocator<> alloc;
};

} // namespace llhd
