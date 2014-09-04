-- Copyright (c) 2014 Fabian Schuiki

-- §13.3.1  Basic identifiers
COUNT X c_out FFT Decoder
VHSIC X1 PageCount STORE_NEXT_ITEM

-- §13.3.2  Extended identifiers
\BUS\  \bus\         -- Two different identifiers, neither of which is the reserved word bus.
\a\\b\               -- An identifier containing three characters.
VHDL  \VHDL\  \vhdl\ -- Three distinct identifiers.

-- §13.2  Delimiters
& () + , - . ; | ! [] > < / * -- Regular Delimiters
=> ** := /= >= <= <>          -- Compound Delimiters
