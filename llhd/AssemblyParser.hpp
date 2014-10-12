/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/SourceLocation.hpp"

namespace llhd {

class Assembly;
class DiagnosticContext;

class AssemblyParser {
	Assembly& into;
	AssemblyLexer& lex;
	DiagnosticContext* diag;
	bool valid;

	struct ModuleContext;
	struct SlotContext;

	bool parseTopLevel();
	bool parseDefine();
	bool parseModuleBody(ModuleContext& ctx);
	bool parseModuleInstruction(ModuleContext& ctx);
	bool parseModuleSlot(SlotContext& ctx);

public:
	AssemblyParser(Assembly& into, AssemblyLexer& lex, DiagnosticContext* diag);
	operator bool() const { return valid; }
};

} // namespace llhd
