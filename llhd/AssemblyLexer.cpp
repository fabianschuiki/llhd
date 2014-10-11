/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/AssemblyLexer.hpp"
#include "llhd/diagnostic/DiagnosticBuilder.hpp"
#include <cstring>
using namespace llhd;


AssemblyLexer::AssemblyLexer(
	SourceLocation loc,
	const Buffer<const char>& buffer,
	DiagnosticContext* diag):

	loc(loc),
	buffer(buffer),
	diag(diag) {

	ptr = buffer.getStart();
	start = ptr;
	token = kInvalid;
}

AssemblyLexer& AssemblyLexer::next() {
	token = kInvalid;
	auto end = buffer.getEnd();

	auto readIdentifier = [&](){
		while (ptr != end && (
			(*ptr >= '0' && *ptr <= '9') ||
			(*ptr >= 'a' && *ptr <= 'z') ||
			(*ptr >= 'A' && *ptr <= 'Z') ||
			strchr("_.\\", *ptr))) {
			if (*ptr == '\\') {
				while (++ptr != end && *ptr >= '0' && *ptr <= '9');
			} else {
				ptr++;
			}
		}
	};

	auto symbol = [&](unsigned tkn) -> AssemblyLexer& {
		token = tkn;
		start = ptr;
		++ptr;
		return *this;
	};

	auto keyword = [&](unsigned tkn) -> AssemblyLexer& {
		token = tkn;
		return *this;
	};

	auto match = [&](const char *str) {
		for (const char* p = start; p != ptr; ++p)
			if (*str++ != *p)
				return false;
		return *str == 0;
	};

	while (ptr != end) {
		// Skip whitespaces.
		if (strchr(" \t\n\r", *ptr)) {
			++ptr;
			continue;
		}

		// Skip comments.
		if (*ptr == '#') {
			while (ptr != end && *ptr++ != '\n');
			continue;
		}

		// Global identifiers.
		if (*ptr == '@') {
			start = ptr;
			++ptr;
			readIdentifier();
			token = kIdentifierGlobal;
			return *this;
		}

		// Local identifiers.
		if (*ptr == '%') {
			start = ptr;
			++ptr;
			readIdentifier();
			token = kIdentifierLocal;
			return *this;
		}

		// Number literals.
		if (*ptr == '$') {
			start = ptr;
			++ptr;
			while (ptr != end && (
				(*ptr >= '0' && *ptr <= '9') ||
				strchr("bdh", *ptr))) {
				++ptr;
			}
			token = kNumberLiteral;
			return *this;
		}

		// Symbols.
		if (*ptr == '{') return symbol(kSymbolLBrace);
		if (*ptr == '}') return symbol(kSymbolRBrace);
		if (*ptr == '/') return symbol(kSymbolSlash);
		if (*ptr == '=') return symbol(kSymbolEqual);

		// Whatever we did not match until now has to be an identifier or a
		// keyword. We treat both as identifiers and match keywords in a second
		// step.
		start = ptr;
		readIdentifier();

		// In case readIdentifier() did not match anthing, i.e. the current
		// character in the input does not belong to an identifier, we skip
		// ahead to the next whitespace as a recovery mechanism and emit a
		// warning that we ignored part of the input.
		if (start == ptr) {
			while (ptr != end && !strchr(" \t\n\r", *ptr))
				++ptr;
			if (diag) {
				DiagnosticBuilder(*diag, kWarning, "garbage '$0', ignored")
					.main(getRange())
					.arg(start, ptr)
					.end();
			} else {
				std::cerr << "garbage '" << std::string(start,ptr)
					<< "' ignored\n";
			}
			continue;
		}

		// Try to match the identifier as a keyword.
		if (match("define"))  return keyword(kKeywordDefine);
		if (match("initial")) return keyword(kKeywordInitial);
		if (match("process")) return keyword(kKeywordProcess);
		if (match("storage")) return keyword(kKeywordStorage);
		token = kIdentifierReserved;
		return *this;
	}

	// Since the token type is set to invalid, the lexer itself will read as
	// false when cast to a boolean.
	return *this;
}

SourceRange AssemblyLexer::getRange() const {
	const char* base = buffer.getStart();
	return SourceRange(
		loc + (unsigned)(start-base),
		loc + (unsigned)(ptr-base));
}

std::string AssemblyLexer::getText() const {
	return std::string((const char*)start, (const char*)ptr);
}

