/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/Buffer.hpp"
#include "llhd/SourceLocation.hpp"

namespace llhd {

class DiagnosticContext;

class AssemblyLexer {
	SourceLocation loc;
	Buffer<const char> buffer;
	DiagnosticContext* diag;

	const char *ptr, *start;
	unsigned token;

public:
	enum {
		kInvalid = 0,
		kIdentifierGlobal,
		kIdentifierLocal,
		kIdentifierReserved,
		kKeywordDefine,
		kKeywordInitial,
		kKeywordProcess,
		kKeywordStorage,
		kNumberLiteral,
		kSymbolLBrace,
		kSymbolRBrace,
		kSymbolSlash,
		kSymbolEqual,
	};


	AssemblyLexer(
		SourceLocation loc,
		const Buffer<const char>& buffer,
		DiagnosticContext* diag);

	AssemblyLexer& next();
	operator bool() const { return token != kInvalid; }

	unsigned getToken() const { return token; }
	SourceRange getRange() const;
	std::string getText() const;
};

} // namespace llhd
