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
	// if (global)
	// 	into.addSymbol(ctx.name, std::move(M));
	// else
	// 	local.add(ctx.name, std::move(M));
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
		if (oneof(lex.getText(), "in", "out", "reg", "wir"))
			return parseModuleStructureInstruction(ctx);
	}

	if (diag) {
		DiagnosticBuilder(*diag, kError, "expected module instruction")
		.main(lex.getRange());
	}
	return false;
}

bool AssemblyParser::parseModuleStructureInstruction(ModuleContext& ctx) {
	assert(lex.getToken() == AssemblyLexer::kIdentifierReserved);
	auto insrng = lex.getRange();
	lex.next();

	// type
	if (lex.getToken() != AssemblyLexer::kIdentifierReserved) {
		if (diag) {
			DiagnosticBuilder(*diag, kError, "expected type name")
			.main(lex.getRange());
		}
		return false;
	}
	auto type = lex.getText();
	lex.next();

	// name
	switch (lex.getToken()) {
	case AssemblyLexer::kIdentifierLocal:
		std::cout << "structure " << lex.getText() << '\n';
		if (diag) {
			DiagnosticBuilder(*diag, kWarning,
			"ignoring instruction, ports not implemented")
			.main(insrng);
		}
		return true;
	case AssemblyLexer::kIdentifierGlobal:
	case AssemblyLexer::kIdentifierReserved:
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"type name must be a local, i.e begin with '%'")
			.main(lex.getRange());
		}
		return false;
	default:
		if (diag) {
			DiagnosticBuilder(*diag, kError,
			"expected structure name")
			.main(lex.getRange());
		}
		return false;
	}
}
