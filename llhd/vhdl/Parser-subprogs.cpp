/* Copyright (c) 2014 Fabian Schuiki */
/// \file
/// Covers IEEE 1076-2000 §2 which deals with subprograms and packages.
#include "llhd/vhdl/Parser-private.hpp"

/// IEEE 1076-2000 §2.1
/// operator_symbol : string_literal
bool Parser::parseOperatorSymbol(TokenScanner& input, bool require) {
	if (!input.accept(kTokenStringLiteral)) {
		if (require) {
			addDiagnostic(input.getCurrentRange(), kError,
				"expected a string containing an operator symbol");
		}
		return false;
	}
	return true;
}
