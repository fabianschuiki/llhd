/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/vhdl/Parser.hpp"
#include "llhd/vhdl/TokenGroup.hpp"
#include "llhd/vhdl/TokenType.hpp"

using namespace llhd::vhdl;

/// Performs the first parsing stage. Groups the tokens according to braces,
/// brackets, and paranthesis to simplify further parsing.
bool Parser::parseFirstStage(
	Token**& start,
	Token** end,
	TokenGroup& into,
	unsigned terminator) {

	while (start != end) {
		auto before = start;

		if ((*start)->type == terminator) {
			into.range.e = (*start)->range.e;
			start++;
			return true;
		} else if ((*start)->type == kTokenRParen) {
			addDiagnostic((*start)->range, kError, "missing opening '('").end();
			start++;
		} else if ((*start)->type == kTokenRBrack) {
			addDiagnostic((*start)->range, kError, "missing opening '['").end();
			start++;
		} else if ((*start)->type == kTokenLParen) {
			auto opentkn = *start;
			auto grp = into.makeGroup(kTokenParenGroup);
			grp->range.s = (*start)->range.s;
			into.addToken(grp);
			start++;
			if (!parseFirstStage(start, end, *grp, kTokenRParen)) {
				addDiagnostic(opentkn->range, kError,
					"missing closing ')'").end();
			}
		} else if ((*start)->type == kTokenLBrack) {
			auto opentkn = *start;
			auto grp = into.makeGroup(kTokenBrackGroup);
			grp->range.s = (*start)->range.s;
			into.addToken(grp);
			start++;
			if (!parseFirstStage(start, end, *grp, kTokenRBrack)) {
				addDiagnostic(opentkn->range, kError,
					"missing closing ']'").end();
			}
		} else {
			into.addToken(*start);
			start++;
		}

		// Sentinel that prevents infinite loops.
		assert(start > before && "parse loop did not progress");
	}
	return false;
}
