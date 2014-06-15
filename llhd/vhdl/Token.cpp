/* Copyright (c) 2014 Fabian Schuiki */
#include "Token.hpp"

namespace llhd {
namespace vhdl {

// std::ostream& operator<< (std::ostream& o, const TokenPosition& p) {
// 	o << p.line << '.' << p.column;
// 	return o;
// }

// std::ostream& operator<< (std::ostream& o, const TokenRange& r) {
// 	o << r.start << '-';
// 	if (r.start.line != r.end.line)
// 		o << r.end.line << '.';
// 	o << r.end.column;
// 	return o;
// }

// const char* tokenTypeToString(TokenType t) {
// 	switch (t) {
// 		case kTokenComment: return "comment";
// 		case kTokenWhitespace: return "whitespace";
// 		case kTokenIdentifier: return "identifier";
// 		case kTokenSymbol: return "symbol";
// 		case kTokenEOF: return "EOF";
// 		default: return "unknown";
// 	}
// }

// std::ostream& operator<< (std::ostream& o, const Token& tkn) {
// 	o << tokenTypeToString(tkn.type) << " '";
// 	for (std::string::const_iterator i = tkn.value.begin(); i != tkn.value.end(); i++) {
// 		if (*i == '\n') {
// 			o << "\\n";
// 		} else {
// 			o << *i;
// 		}
// 	}
// 	o << "' " << tkn.range;
// 	return o;
// }

} // namespace vhdl
} // namespace llhd
