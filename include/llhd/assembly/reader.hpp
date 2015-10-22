/* Copyright (c) 2015 Fabian Schuiki */
#pragma once
#include "llhd/utils/range.hpp"
#include "llhd/location.hpp"

namespace llhd {


class Assembly;
class AssemblyLexer;
class DiagnosticContext;

/// \ingroup assembly
/// \needsdoc
class AssemblyReader {
	Assembly &m_assembly;
public:
	AssemblyReader(Assembly &assembly);
	AssemblyReader& operator()(Range<const char*> input, SourceLocation loc, DiagnosticContext *dctx = nullptr);
	AssemblyReader& operator()(AssemblyLexer &input, DiagnosticContext *dctx = nullptr);
};


} // namespace llhd
