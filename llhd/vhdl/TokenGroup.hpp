/* Copyright (c) 2014 Fabian Schuiki */
#pragma once
#include "llhd/Token.hpp"
#include <memory>
#include <vector>

namespace llhd {
namespace vhdl {

class TokenGroup : public Token {
	std::vector<Token*> tokens;
	std::vector<std::unique_ptr<TokenGroup>> ownedGroups;

public:

	template<typename... Args>
	TokenGroup(Args&&... args): Token(args...) {}

	/// Adds the Token \a tkn to this group. The whole of all calls to this
	/// function forms a sequence of Token, which may be accessed by calling
	/// the getBuffer() function.
	void addToken(Token* tkn) {
		tokens.push_back(tkn);
	}

	/// Returns a TokenBuffer that covers all tokens that were added to this
	/// group via addToken().
	///
	/// \warning The returned buffer is invalidated by subsequent calls to
	/// addToken().
	TokenBuffer getBuffer() {
		return TokenBuffer(&tokens[0], tokens.size());
	}

	/// Takes ownership of the given token group. It is deallocated when this
	/// token group is deallocated.
	void takeOwnership(std::unique_ptr<TokenGroup>&& tg) {
		ownedGroups.push_back(std::move(tg));
	}

	/// Allocates a new token group associated with this one. The returned group
	/// is deallocated when this token group is deallocated.
	template<typename... Args>
	TokenGroup* makeGroup(Args&&... args) {
		ownedGroups.emplace_back(new TokenGroup(args...));
		return ownedGroups.back().get();
	}
};

} // namespace vhdl
} // namespace llhd
