/* Copyright (c) 2014 Fabian Schuiki */
/// \file
/// Covers IEEE 1076-2000 §6 which deals with names.
#include "llhd/vhdl/Parser-private.hpp"


/// IEEE 1076-2000 §6.1
/// name : simple_name
///      | operator_symbol
///      | selected_name
///      | indexed_name
///      | slice_name
///      | attribute_name
bool Parser::parseName(TokenScanner& input, bool require) {
	auto scn = input.branch();
	if (parseSimpleName(scn, false)) {
	} else if (parseOperatorSymbol(scn, false)) {
	} else if (parseSelectedName(scn, false)) {
	} else if (parseIndexedName(scn, false)) {
	} else if (parseSliceName(scn, false)) {
	} else if (parseAttributeName(scn, false)) {
	} else {
		if (require) {
			addDiagnostic(scn.getRange(), kError,
				"expected simple, selected, indexed, slice, or attribute name, or operator symbol");
		}
		return false;
	}

	addDiagnostic(scn.getRangeToHere(), kNote, "name");
	scn.commit();
	return true;
}

/// IEEE 1076-2000 §6.1
/// prefix : name | function_call
bool Parser::parsePrefix(TokenScanner& input, bool require) {
	auto scn = input.branch();
	if (parseName(scn, false)) {
	// } else if (parseFunctionCall(scn, false)) {
	} else {
		if (require) {
			addDiagnostic(scn.getRange(), kError,
				"expected name or function call");
		}
		return false;
	}

	addDiagnostic(scn.getRangeToHere(), kNote, "prefix");
	scn.commit();
	return true;
}

/// IEEE 1076-2000 §6.2
/// simple_name : identifier
bool Parser::parseSimpleName(TokenScanner& input, bool require) {
	if (input.accept(kTokenBasicIdentifier)) {
	} else if (input.accept(kTokenExtendedIdentifier)) {
	} else {
		if (require) {
			addDiagnostic(input.getCurrentRange(), kError,
				"expected simple name");
		}
		return false;
	}
	return true;
}

/// IEEE 1076-2000 §6.3
/// selected_name : prefix "." suffix
/// suffix : simple_name | character_literal | operator_symbol | "all"
bool Parser::parseSelectedName(TokenScanner& input, bool require) {
	auto scn = input.branch();
	if (!parsePrefix(scn, require))
		return false;

	if (!scn.accept(kTokenPeriod)) {
		if (require) {
			addDiagnostic(scn.getCurrentRange(), kError,
				"period and suffix required after prefix")
				.highlight(scn.getRangeToHere());
		}
		return false;
	}
	auto tkn_period = *scn.getCurrent();

	if (parseSimpleName(scn, false)) {
	} else if (scn.accept(kTokenCharacterLiteral)) {
	} else if (parseOperatorSymbol(scn, false)) {
	} else if (scn.accept(kKeywordAll)) {
	} else {
		if (require) {
			addDiagnostic(tkn_period->range, kError,
				"expected simple name, character literal, operator symbol, or the "
				"'all' keyword after period")
				.highlight(scn.getRangeToHere());
		}
		return false;
	}

	addDiagnostic(scn.getRangeToHere(), kNote, "selected_name");
	scn.commit();
	return true;
}

/// IEEE 1076-2000 §6.4
/// indexed_name : prefix "(" expression {"," expression} ")"
bool Parser::parseIndexedName(TokenScanner& input, bool require) {
	not_implemented;
}

/// IEEE 1076-2000 §6.5
/// slice_name : prefix "(" discrete_range ")"
bool Parser::parseSliceName(TokenScanner& input, bool require) {
	not_implemented;
}

/// IEEE 1076-2000 §6.6
/// attribute_name : prefix [signature] "'" attribute_designator ["(" expression ")"]
bool Parser::parseAttributeName(TokenScanner& input, bool require) {
	not_implemented;
}
