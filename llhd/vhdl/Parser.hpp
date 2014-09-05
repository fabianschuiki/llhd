/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/NullTerminatedIterator.hpp"
#include "llhd/TokenBuffer.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"

namespace llhd {

class DiagnosticContext;
class TokenScanner;

namespace vhdl {

class TokenGroup;

// Forward declaration of the ast::Context class.
namespace ast {
	class Context;
} // namespace ast

/// Parses a sequence of tokens into a valid VHDL abstract syntax tree. See the
/// llhd::vhdl::ast namespace for an overview of the relevant classes. The AST
/// is emitted into an ast::Context.
class Parser {
	ast::Context& ctx;
	DiagnosticContext& diag;
	typedef NullTerminatedIterator<Token*> Iterator;

	bool accept(Iterator& input, unsigned type, Token*& token);
	bool accept(Iterator& input, unsigned type);
	bool acceptIdentifier(Iterator& input, Token*& token);

	void parseDesignUnit(Iterator& input);
	bool acceptLibraryClause(Iterator& input);
	bool acceptEntityDeclaration(Iterator& input);
	bool acceptConfigurationDeclaration(Iterator& input);
	bool acceptPackageDeclaration(Iterator& input);
	bool acceptArchitectureBody(Iterator& input);
	bool acceptPackageBody(Iterator& input);
	bool acceptSelectedName(Iterator& input);

	template<typename... Args>
	DiagnosticBuilder addDiagnostic(Args... args) {
		return std::move(DiagnosticBuilder(diag, args...));
	}

public:
	Parser(ast::Context& ctx, DiagnosticContext& diag): ctx(ctx), diag(diag) {}

	void parse(const TokenBuffer& input);

private:
	bool parseFirstStage(
		Token**& start,
		Token** end,
		TokenGroup& into,
		unsigned terminator = 0);
	bool parseSecondStage(
		Token**& start,
		Token** end,
		TokenGroup& into);

	bool requireDesignUnit(TokenScanner& input);
	bool requireLibraryUnit(TokenScanner& input);
	bool acceptLibraryClause(TokenScanner& input);
	bool requireContextClause(TokenScanner& input);
	bool acceptContextItem(TokenScanner& input);
	bool acceptUseClause(TokenScanner& input);

	// Subprograms and Packages (§2)
	bool parseOperatorSymbol(TokenScanner& input, bool require);

	// Names (§6)
	bool parseName(TokenScanner& input, bool require);
	bool parsePrefix(TokenScanner& input, bool require);
	bool parseSimpleName(TokenScanner& input, bool require);
	bool parseSelectedName(TokenScanner& input, bool require);
	bool parseIndexedName(TokenScanner& input, bool require);
	bool parseSliceName(TokenScanner& input, bool require);
	bool parseAttributeName(TokenScanner& input, bool require);
};

} // namespace vhdl
} // namespace llhd
