/* Copyright (c) 2014 Fabian Schuiki */
#include "llhd/Tokenizer.hpp"
using namespace llhd;


Tokenizer::~Tokenizer()
{
	for (StateSet::iterator i = states.begin(); i != states.end(); i++)
		delete *i;
	states.clear();
}


bool Tokenizer::read(Token &token)
{
	StateSet activeStates(states);
	std::string buffer;
	std::string::iterator bl_pos = backlog.begin();

	while (true)
	{
		// Fetch the next character either from the backlog if that's not empty,
		// or the input stream until its end is reached.
		int c;
		if (bl_pos != backlog.end()) {
			c = *bl_pos++;
		} else {
			c = fin.get();
			if (!fin.good())
				break;
		}

		// Move the character onto the buffer and check what the states make of
		// it.
		buffer += c;

		for (StateSet::iterator i = activeStates.begin(); i != activeStates.end(); i++) {
			TokenizerState *state = *i; i++; // i++ makes sure that the iterator is valid after deletion
			TokenizerState::MatchResult m = state->match(buffer, token);
			if (m == TokenizerState::kDiscard) {
				state->reset();
				activeStates.erase(state);
			} else if (m == TokenizerState::kReduce) {

			}
		}
	}

	// Throw an error here if the buffer is not empty, which means that certain
	// characters could not have been parsed.
	return false;
}
