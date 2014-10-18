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
	AssemblySignal& slot;
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

bool AssemblyParser::error(const char* msg) {
	if (diag) {
		DiagnosticBuilder(*diag, kError, msg)
		.main(lex.getRange());
	}
	return false;
}

bool AssemblyParser::error(SourceRange range, const char* msg) {
	if (diag) {
		DiagnosticBuilder(*diag, kError, msg)
		.main(range);
	}
	return false;
}

bool AssemblyParser::parseTopLevel() {
	switch (lex.getToken()) {
	default: return error("expected top-level entity");
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
	default: return error("expected global or local name");
	}
	ModuleContext ctx { *M.get(), lex.getRange(), lex.getText() };

	// module body
	if (lex.next().getToken() != AssemblyLexer::kSymbolLBrace)
		return error("expected opening braces '{' after module name");
	if (!parseModuleBody(ctx))
		return false;

	// add the module to the appropriate symbol table
	if (global) {
		auto& slot = into.modules[M->name];
		if (slot) {
			return error(ctx.range, "symbol name already used");
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
		return error("expected closing braces '}' at end of module");
	}
	return true;
}

bool AssemblyParser::parseModuleInstruction(ModuleContext& ctx) {

	if (lex.getToken() == AssemblyLexer::kIdentifierReserved) {
		auto ins = lex.getText();

		if (oneof(ins, "in", "out", "sig", "reg")) {
			std::shared_ptr<AssemblySignal> S(new AssemblySignal);
			if (ins == "in") S->dir = AssemblySignal::kPortIn;
			if (ins == "out") S->dir = AssemblySignal::kPortOut;
			if (ins == "sig") S->dir = AssemblySignal::kSignal;
			if (ins == "reg") S->dir = AssemblySignal::kRegister;

			SlotContext sctx { ctx, *S };
			if (!parseModuleSlot(sctx))
				return false;

			auto& slot = ctx.module.signals[S->name];
			if (slot) {
				return error(sctx.namerange, "symbol name already used");
			}
			slot = std::move(S);
			return true;
		}
	}

	return error("expected module instruction");
}

bool AssemblyParser::parseModuleSlot(SlotContext& ctx) {
	lex.next();

	// type
	if (lex.getToken() != AssemblyLexer::kIdentifierReserved) {
		return error("expected type name");
	}
	// ctx.slot.type = lex.getText();
	lex.next();

	// name
	switch (lex.getToken()) {
	case AssemblyLexer::kIdentifierLocal: break;
	case AssemblyLexer::kIdentifierGlobal:
	case AssemblyLexer::kIdentifierReserved:
		return error("name must be local (start with %)");
	default:
		return error("expected name");
	}
	ctx.namerange = lex.getRange();
	ctx.slot.name = lex.getText();

	return true;
}
