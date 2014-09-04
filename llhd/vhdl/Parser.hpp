/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/NullTerminatedIterator.hpp"
#include "llhd/TokenBuffer.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"

namespace llhd {

class DiagnosticContext;

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
	bool acceptUseClause(Iterator& input);
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
};

} // namespace vhdl
} // namespace llhd
