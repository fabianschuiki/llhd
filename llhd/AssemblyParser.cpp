/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Assembly.hpp"
#include "llhd/AssemblyLexer.hpp"
#include "llhd/AssemblyParser.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"
#include <iostream>
#include <memory>
using namespace llhd;


struct AssemblyParser::ModuleContext {
	AssemblyModule& module;
	SourceRange range;
	const std::string& name;
};

struct AssemblyParser::SlotContext {
	ModuleContext& modctx;
	AssemblySlot& slot;
	SourceRange namerange;
};


template<typename T, typename U>
bool oneof(const T& lhs, const U& rhs) {
	return lhs == rhs;
}

template<typename T, typename U, typename... Args>
bool oneof(const T& lhs, const U& rhs, Args... rest) {
	return oneof(lhs,rhs) || oneof(lhs, rest...);
}


AssemblyParser::AssemblyParser(
	Assembly& into,
	AssemblyLexer& lex,
	DiagnosticContext* diag):

	into(into),
	lex(lex),
	diag(diag) {

	valid = true;
	while (lex.next()) {
		if (!parseTopLevel()) {
			valid = false;
			break;
		}
	}
}

bool AssemblyParser::parseTopLevel() {
	switch (lex.getToken()) {
	default:
		if (diag) {
			DiagnosticBuilder(*diag, kError, "expected top-level entity")
			.main(lex.getRange());
		}
		return false;

	case AssemblyLexer::kInvalid: return false;
	case AssemblyLexer::kKeywordDefine: return parseDefine();
	}
}

bool AssemblyParser::parseDefine() {
	assert(lex.getToken() == AssemblyLexer::kKeywordDefine);
	lex.next();
	std::shared_ptr<AssemblyModule> M(new AssemblyModule);

	// module name
	bool global;
	switch (lex.getToken()) {
	case AssemblyLexer::kIdentifierGlobal: global = true; break;
	case AssemblyLexer::kIdentifierLocal: global = false; break;
	default:
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"expected global or local module name")
			.main(lex.getRange());
		}
		return false;
	}
	ModuleContext ctx { *M.get(), lex.getRange(), lex.getText() };

	// module body
	if (lex.next().getToken() != AssemblyLexer::kSymbolLBrace) {
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"expected opening braces '{' after module name")
			.main(lex.getRange())
			.highlight(ctx.range);
		}
		return false;
	}
	if (!parseModuleBody(ctx))
		return false;

	// add the module to the appropriate symbol table
	if (global) {
		auto& slot = into.modules[M->name];
		if (slot) {
			if (diag) {
				DiagnosticBuilder(*diag, kError,
				"symbol name already used")
				.main(ctx.range);
			}
			return false;
		}
		slot = std::move(M);
	}
	return true;
}

bool AssemblyParser::parseModuleBody(ModuleContext& ctx) {
	assert(lex.getToken() == AssemblyLexer::kSymbolLBrace);

	while (lex.next() && lex.getToken() != AssemblyLexer::kSymbolRBrace) {
		if (!parseModuleInstruction(ctx))
			return false;
	}

	if (lex.getToken() != AssemblyLexer::kSymbolRBrace) {
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"expected closing braces '}' at end of module")
			.main(lex.getRange());
		}
		return false;
	}
	return true;
}

bool AssemblyParser::parseModuleInstruction(ModuleContext& ctx) {

	if (lex.getToken() == AssemblyLexer::kIdentifierReserved) {
		auto ins = lex.getText();

		if (oneof(ins, "in", "out", "sig", "reg")) {
			std::shared_ptr<AssemblySlot> S(new AssemblySlot);
			if (ins == "in") S->dir = AssemblySlot::kPortIn;
			if (ins == "out") S->dir = AssemblySlot::kPortOut;
			if (ins == "sig") S->dir = AssemblySlot::kSignal;
			if (ins == "reg") S->dir = AssemblySlot::kRegister;

			SlotContext sctx { ctx, *S };
			if (!parseModuleSlot(sctx))
				return false;

			auto& slot = ctx.module.slots[S->name];
			if (slot) {
				if (diag) {
					DiagnosticBuilder(*diag, kError,
					"symbol name already used")
					.main(sctx.namerange);
				}
				return false;
			}
			slot = std::move(S);
			return true;
		}
	}

	if (diag) {
		DiagnosticBuilder(*diag, kError, "expected module instruction")
		.main(lex.getRange());
	}
	return false;
}

bool AssemblyParser::parseModuleSlot(SlotContext& ctx) {
	lex.next();

	// type
	if (lex.getToken() != AssemblyLexer::kIdentifierReserved) {
		if (diag) {
			DiagnosticBuilder(*diag, kError, "expected type name")
			.main(lex.getRange());
		}
		return false;
	}
	ctx.slot.type = lex.getText();
	lex.next();

	// name
	switch (lex.getToken()) {
	case AssemblyLexer::kIdentifierLocal: break;
	case AssemblyLexer::kIdentifierGlobal:
	case AssemblyLexer::kIdentifierReserved:
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"name must be local (start with %)")
			.main(lex.getRange());
		}
		return false;
	default:
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"expected name")
			.main(lex.getRange());
		}
		return false;
	}
	ctx.namerange = lex.getRange();
	ctx.slot.name = lex.getText();

	return true;
}
