/* Copyright (c) 2015 Fabian Schuiki */
#include "llhd/assembly/lexer.hpp"
#include <iostream>

namespace llhd {


/// Returns true if \a c is a whitespace character.
static bool is_whitespace(char c) {
	return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}


/// Returns true if \a c is a character that starts a comment.
static bool is_comment(char c) {
	return c == '#' || c == ';';
}


/// Returns true if \a is a valid hexadecimal digit. The numbers 0 through 9 and
/// the uppercase letters A through F are considered valid hexadecimal digits in
/// LLHD.
static bool is_hexadecimal(char c) {
	return (c >= '0' && c <= '9') ||
	       (c >= 'A' && c <= 'F');
}


/// Returns true if \a c is a valid name character, i.e. if it matches the
/// regular expression `[0-9a-zA-Z_.]`.
static bool is_name_char(char c) {
	return (c >= '0' && c <= '9') ||
	       (c >= 'a' && c <= 'z') ||
	       (c >= 'A' && c <= 'Z') ||
	       (c == '_' || c == '\\' || c == '.');
}


/// Returns true if \a c is a valid number literal character, i.e. if it matches
/// the regular expression `[0-9A-Z-]`.
static bool is_number_literal_char(char c) {
	return (c >= '0' && c <= '9') ||
	       (c >= 'A' && c <= 'Z') ||
	       (c == '-');
}


/// Returns true if the sequence [c,e) is non-empty (c != e) and contains only
/// digits, i.e. if it matches the regular expression `[0-9]+`.
static bool is_digits_only(const char *c, const char *e) {
	if (c == e)
		return false;
	for (; *c != *e; ++c) {
		if (*c < '0' || *c > '9')
			return false;
	}
	return true;
}


/// Returns true if the sequence [c,e) is a valid type name, i.e. if it matches
/// the regular expression `(i|l|ls)[0-9]+`.
static bool is_type_name(const char *c, const char *e) {
	llhd_assert(c < e);
	if (c == e)
		return false;

	if (*c == 'i') {
		++c;
	} else if (*c == 'l') {
		++c;
		if (c != e && *c == 's')
			++c;
	}
	return is_digits_only(c,e);
}


/// \needsdoc
AssemblyLexer::AssemblyLexer(Range<const char*> input, SourceLocation loc, DiagnosticContext *dctx)
	:	m_input(input), m_loc(loc), m_dctx(dctx) {
	m_base = m_ptr = m_input.begin();
}


/// \needsdoc
AssemblyLexer& AssemblyLexer::next() {

	if (m_token == TOKEN_INVALID)
		return *this;
	m_token = TOKEN_INVALID;

	auto end = [&](){ return m_ptr == m_input.end(); };
	auto match = [&](const char *s){
		auto p = m_base;
		for (; p != m_ptr && *s; ++p, ++s) {
			if (*s != *p)
				return false;
		}
		return p == m_ptr && *s == 0;
	};

	while (!end()) {
		// Skip whitespaces.
		if (is_whitespace(*m_ptr)) {
			++m_ptr;
			continue;
		}

		// Skip comments.
		if (is_comment(*m_ptr)) {
			while (!end() && *m_ptr != '\n')
				++m_ptr;
			continue;
		}

		// Global names.
		if (*m_ptr == '@') {
			m_base = m_ptr;
			m_token = TOKEN_NAME_GLOBAL;
			++m_ptr;
			if (!read_name())
				m_token = TOKEN_INVALID;
			return *this;
		}

		// Local names.
		if (*m_ptr == '%') {
			m_base = m_ptr;
			m_token = TOKEN_NAME_LOCAL;
			++m_ptr;
			if (!read_name())
				m_token = TOKEN_INVALID;
			return *this;
		}

		// All that's left are either keywords, types, or number literals.
		m_base = m_ptr;
		while (!end() && is_name_char(*m_ptr))
			++m_ptr;

		// Try to match type names.
		if (m_ptr-m_base > 1 && is_type_name(m_base, m_ptr))
			m_token = TOKEN_TYPE;

		// If the following character is an apostrophe, this is a number literal
		// token.
		if (!end() && m_token == TOKEN_TYPE && *m_ptr == '\'') {
			++m_ptr;
			if (end()) {
				/// \todo Emit a diagnostic message.
				std::cerr << "expected character in number literal\n";
				m_token = TOKEN_INVALID;
				return *this;
			}
			while (!end() && is_number_literal_char(*m_ptr))
				++m_ptr;
			m_token = TOKEN_NUMBER_LITERAL;
		}

		if (m_token != TOKEN_INVALID)
			return *this;

		// All that's left are keywords.
		#define KEYWORD(name, tkn) if (match(name)) {\
			m_token = tkn; return *this;\
		}
		KEYWORD("mod",  TOKEN_KW_MOD);
		KEYWORD("proc", TOKEN_KW_PROC);
		KEYWORD("func", TOKEN_KW_FUNC);
		#undef KEYWORD

		// If we get here, whatever we read was invalid and did not match
		// anything. Emit a diagnostic message and abort.
		/// \todo Emit a diagnostic message.
		std::cerr << "garbage in input file\n";
		return *this;
	}

	if (end())
		m_token = TOKEN_EOF;
	return *this;
}


/// \needsdoc
bool AssemblyLexer::read_name() {
	auto offset = m_ptr;
	while (m_ptr != m_input.end() && is_name_char(*m_ptr)) {
		if (*m_ptr == '\\') {
			++m_ptr;

			if (m_ptr == m_input.end() || *m_ptr != 'u') {
				/// \todo Emit diagnostic message here.
				std::cerr << "expected 'u' in escape sequence\n";
				return false;
			}
			++m_ptr;

			for (unsigned i = 0; i < 2; ++i) {
				if (m_ptr == m_input.end() || !is_hexadecimal(*m_ptr)) {
					/// \todo Emit diagnostic message here.
					std::cerr << "exepcted hexadecimal digit in escape sequence\n";
					return false;
				}
				++m_ptr;
			}
		} else {
			++m_ptr;
		}
	}

	// If we haven't read anything, emit a diagnostic message and return false.
	// Otherwise return true, as we have successfully read a name.
	if (m_ptr == offset) {
		/// \todo Emit diagnostic message.
		std::cerr << "expected valid name character\n";
		return false;
	} else {
		return true;
	}
}


/// \needsdoc
AssemblyLexer::operator bool() const {
	return !is_at_end() && !is_invalid();
}


/// \needsdoc
bool AssemblyLexer::is_at_end() const {
	return m_token == TOKEN_EOF;
}


/// \needsdoc
bool AssemblyLexer::is_invalid() const {
	return m_token == TOKEN_INVALID;
}


/// Returns the current token.
AssemblyLexerToken AssemblyLexer::current_token() const {
	return m_token;
}


/// Returns the SourceRange of the current token.
SourceRange AssemblyLexer::current_range() const {
	return SourceRange(
		m_loc + (m_base - m_input.begin()),
		m_loc + (m_ptr  - m_input.begin())
	);
}


/// Returns the Range of the input that contains the current token.
Range<const char*> AssemblyLexer::current_text() const {
	return make_range(m_base, m_ptr);
}


} // namespace llhd
