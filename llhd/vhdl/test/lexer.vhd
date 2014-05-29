-- Copyright (c) 2014 Fabian Schuiki

-- §13.3.1  Basic identifiers
COUNT X c_out FFT Decoder
VHSIC X1 PageCount STORE_NEXT_ITEM

-- §13.3.2  Extended identifiers
\BUS\  \bus\         -- Two different identifiers, neither of which is the reserved word bus.
\a\\b\               -- An identifier containing three characters.
VHDL  \VHDL\  \vhdl\ -- Three distinct identifiers.

-- §13.4.1  Decimal literals
12  0  1E6  123_456              -- Integer literals.
12.0  0.0  0.456  3.14159_26     -- Real literals.
1.34E–12  190  1.0E+6  6.023E+24 -- Real literals with exponents.

-- §13.4.2  Based literals
2#1111_1111#  16#FF#  016#0FF#     -- Integer literals of value 255
16#E#E1  2#1110_0000#              -- Integer literals of value 254
16#F.FF#E+2  2#1.1111_1111_111#E11 -- Real literals of value 4095.0

-- §13.5  Character literals
'A'  '*'  ' '

-- §13.6  String literals
"Setup time is too short" -- An error message.
""                        -- An empty string literal.
" "  "A"  """"  "%"       -- String literals of length 1.
"Characters such as $, %, and } are allowed in string literals."

-- §13.7  Bit string literals
B"1111_1111_1111" -- Equivalent to the string literal "111111111111".
X"FFF"            -- Equivalent to B"1111_1111_1111".
O"777"            -- Equivalent to B"111_111_111".
X"777"            -- Equivalent to B"0111_0111_0111".

-- §13.9  Reserved words
abs access after alias all and architecture array assert attribute begin block
body buffer bus case component configuration constant label disconnect downto
map else elsif end entity exit file for function generate generic group
guarded if impure in inertial inout is library linkage literal loop mod nand
new next nor not null of on open or others out package port postponed
procedural procedure process protected pure range record reference register
reject rem report return rol ror select severity shared signal sla sll sra srl
subtype then to transport type unaffected units until use variable wait when
while with xnor xor

-- §13.10  Allowable replacements of characters
2:1111_1111:  16:FF:  016:0FF:     -- Integer literals of value 255
16:E:E1  2:1110_0000:              -- Integer literals of value 254
16:F.FF:E+2  2:1.1111_1111_111:E11 -- Real literals of value 4095.0

%Setup time is too short% -- An error message.
%%                        -- An empty string literal.
% %  %A%  %"%  %%%%       -- String literals of length 1.
%Characters such as $, %, and } are allowed in string literals.%
