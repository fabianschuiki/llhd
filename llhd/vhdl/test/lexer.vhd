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
1.34E-12  190  1.0E+6  6.023E+24 -- Real literals with exponents.

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
"Characters such as $, %, "", and } are allowed in string literals."
"Backslash \\"  "Escaped \" and \so \on"

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

ABS ACCESS AFTER ALIAS ALL AND ARCHITECTURE ARRAY ASSERT ATTRIBUTE BEGIN BLOCK
BODY BUFFER BUS CASE COMPONENT CONFIGURATION CONSTANT LABEL DISCONNECT DOWNTO
MAP ELSE ELSIF END ENTITY EXIT FILE FOR FUNCTION GENERATE GENERIC GROUP
GUARDED IF IMPURE IN INERTIAL INOUT IS LIBRARY LINKAGE LITERAL LOOP MOD NAND
NEW NEXT NOR NOT NULL OF ON OPEN OR OTHERS OUT PACKAGE PORT POSTPONED
PROCEDURAL PROCEDURE PROCESS PROTECTED PURE RANGE RECORD REFERENCE REGISTER
REJECT REM REPORT RETURN ROL ROR SELECT SEVERITY SHARED SIGNAL SLA SLL SRA SRL
SUBTYPE THEN TO TRANSPORT TYPE UNAFFECTED UNITS UNTIL USE VARIABLE WAIT WHEN
WHILE WITH XNOR XOR

-- §13.10  Allowable replacements of characters
2:1111_1111:  16:FF:  016:0FF:     -- Integer literals of value 255
16:E:E1  2:1110_0000:              -- Integer literals of value 254
16:F.FF:E+2  2:1.1111_1111_111:E11 -- Real literals of value 4095.0

%Setup time is too short% -- An error message.
%%                        -- An empty string literal.
% %  %A%  %"%  %%%%       -- String literals of length 1.
%Characters such as $, %%, ", and } are allowed in string literals.%

-- §13.2  Delimiters
& () + , - . ; | ! [] > < / * -- Regular Delimiters
=> ** := /= >= <= <>          -- Compound Delimiters

-- Partial Reserved Words.
a b c d e f g i l m n o p r s t u v w x

ab ac af al an ar as at be bl bo bu ca co di do el en ex fi fo fu ge gr gu if im
in is la li lo ma mo na ne no nu of on op or ot ou pa po pr pu ra re ro se sh si
sl sr su th to tr ty un us va wa wh wi xn xo

acc aft ali arc arr ass att beg blo bod buf cas com con dis dow els ent exi fil
fun gen gro gua imp ine ino lab lib lin lit loo nan nex nul ope oth pac por pos
pro pur ran rec ref reg rej rep ret sel sev sha sig sub the tra typ una uni unt
var wai whe whi wit xno

acce afte alia arch arra asse attr begi bloc buff comp conf cons labe disc down
elsi enti func gene grou guar impu iner inou libr link lite othe pack post proc
prot rang reco refe regi reje repo retu sele seve shar sign subt tran unaf unit
unti vari whil

acces archi asser attri buffe compo confi const disco downt entit funct gener
gener guard impur inert libra linka liter other packa postp proce proce prote
recor refer regis rejec repor retur selec sever share signa subty trans unaff
varia

archit attrib compon config consta discon functi genera generi guarde inerti
librar linkag litera packag postpo proced proced proces protec refere regist
severi subtyp transp unaffe variab

archite attribu compone configu constan disconn functio generat inertia postpon
procedu procedu protect referen registe severit transpo unaffec variabl

architec attribut componen configur disconne postpone procedur procedur protecte
referenc transpor unaffect

architect configura disconnec procedura unaffecte

architectu configurat

architectur configurati

-- Bit string literal with escape sequence. Not standard-compliant.
B"1111\"_1111"
B%1111\%1111%
