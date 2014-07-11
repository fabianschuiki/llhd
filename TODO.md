# Current Development Leads

- `llhd/vhdl`
	- Parser uses a NullTerminatedIterator to process the input tokens. It might
	  be better to create a separate token processing class that accepts a range
	  of tokens and provides various `accept` functions on it. It should also
	  allow to keep track of *safely parsed tokens* to enable highlighting of
	  entire sequences of nonsensical tokens. Such a class may be instantiated
	  for every call to an `accept` function, or slightly less frequently.
