/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/NullTerminatedIterator.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"

namespace llhd {

class DiagnosticContext;
class TokenBuffer;

namespace vhdl {

class Parser {
	DiagnosticContext& diactx;
	typedef NullTerminatedIterator<Token*> Iterator;

	void parseDesignUnit(Iterator& input);
	bool acceptLibraryClause(Iterator& input);
	bool acceptUseClause(Iterator& input);
	bool acceptEntityDeclaration(Iterator& input);
	bool acceptConfigurationDeclaration(Iterator& input);
	bool acceptPackageDeclaration(Iterator& input);
	bool acceptArchitectureBody(Iterator& input);
	bool acceptPackageBody(Iterator& input);

	template<typename... Args>
	DiagnosticBuilder addDiagnostic(Args... args) {
		return std::move(DiagnosticBuilder(diactx, args...));
	}

public:
	Parser(DiagnosticContext& diactx): diactx(diactx) {}

	void parse(const TokenBuffer& input);
};

} // namespace vhdl
} // namespace llhd
