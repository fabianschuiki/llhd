Mapping VHDL to LLHD
====================

This section outlines how various VHDL constructs can be mapped to the LLHD language. Many of the code snippets have been adapted from examples in the language reference manual (IEEE 1076-2008).

Types
-----

Enumeration Types:

.. code-block:: vhdl

	type MULTI_LEVEL_LOGIC is (LOW, HIGH, RISING, FALLING, AMBIGUOUS);
	type BIT is ('0','1');
	type SWITCH_LEVEL is ('0','1','X');

.. code-block:: llhd

	%MULTI_LEVEL_LOGIC = type n5 ; metadata can specify variant names
	%BIT = type n2
	%SWITCH_LEVEL = type n3

Integer Types:

.. code-block:: vhdl

	type TWOS_COMPLEMENT_INTEGER is range -32768 to 32767;
	type BYTE_LENGTH_INTEGER is range 0 to 255;
	type WORD_INDEX is range 31 downto 0;
	subtype HIGH_BIT_LOW is BYTE_LENGTH_INTEGER range 0 to 127;

.. code-block:: llhd

	%TWOS_COMPLEMENT_INTEGER = type i16
	%BYTE_LENGTH_INTEGER = type i8
	%WORD_INDEX = type i5
	%HIGH_BIT_LOW = type i7

Array Types:

.. code-block:: vhdl

	type MY_WORD is array (0 to 31) of BIT;
	type DATA_IN is array (7 downto 0) of FIVE_LEVEL_LOGIC;
	type MEMORY is array (INTEGER range <>) of MY_WORD;
	type SIGNED_FXPT is array (INTEGER range <>) of BIT;
	type SIGNED_FXPT_VECTOR is array (NATURAL range <>) of SIGNED_FXPT;

.. code-block:: llhd

	%MY_WORD = type [32 x %BIT]
	%DATA_IN = type [8 x %FIVE_LEVEL_LOGIC]
	; MEMORY cannot be expressed (unconstrained)
	; SIGNED_FXPT cannot be expressed (unconstrained)
	; SIGNED_FXPT_VECTOR cannot be expressed (unconstrained)

Record Types:

.. code-block:: vhdl

	type DATE is record
		DAY : INTEGER range 1 to 31;
		MONTH : MONTH_NAME;
		YEAR : INTEGER range 0 to 4000;
	end record;

	type SIGNED_FXPT_COMPLEX is record
		RE : SIGNED_FXPT;
		IM : SIGNED_FXPT;
	end record;

.. code-block:: llhd

	%DATE = type { i5, %MONTH_NAME, i12 }
	%SIGNED_FXPT_COMPLEX = type { %SIGNED_FXPT, %SIGNED_FXPT }

Access Types:

.. code-block:: vhdl

	type ADDRESS is access MEMORY;
	type BUFFER_PTR is access TEMP_BUFFER;

.. code-block:: llhd

	%ADDRESS = type %MEMORY*
	%BUFFER_PTR = type %TEMP_BUFFER*


Declarations
------------

Constant Declarations:

.. code-block:: vhdl

	constant TOLER: DISTANCE := 1.5 nm;
	constant PI: REAL := 3.141592;
	constant CYCLE_TIME: TIME := 100 ns;
	constant Propagation_Delay: DELAY_LENGTH; -- A deferred constant.

.. code-block:: llhd

	@TOLER = const i64 15
	; PI cannot be expressed (floating point values not supported)
	@CYCLE_TIME = const i64 100000000
	; Propagation_Delay cannot be expressed (constants cannot be declared)

Signal Declarations:

.. code-block:: vhdl

	signal S: STANDARD.BIT_VECTOR (1 to 10);
	signal CLK1, CLK2: TIME;
	signal OUTPUT: WIRED_OR MULTI_VALUED_LOGIC;

.. code-block:: llhd

	%S = sig [10 x %STANDARD.BIT]
	%CLK1 = sig time
	%CLK2 = sig time
	%OUTPUT = sig %MULTI_VALUED_LOGIC

Variable Declarations:

.. code-block:: vhdl

	subtype ShortRange is INTEGER range -1 to 1;
	variable Local: ShortRange := 0;
	variable V: ShortRange;

.. code-block:: llhd

	%Local = var i2
	store i2 %Local 0
	%V = var i2

File Declarations:

.. code-block:: vhdl

	type IntegerFile is file of INTEGER;
	file F1: IntegerFile;
	file F2: IntegerFile is "test.dat";
	file F3: IntegerFile open WRITE_MODE is "test.dat";

.. code-block:: llhd

	@str = const [i8 x 9] "test.dat"
	@READ_MODE = const n2 0
	@WRITE_MODE = const n2 1
	decl func @FILE_OPEN (i32*, i8*, n2) void

	%F1 = var i32
	%F2 = var i32
	%F3 = var i32
	store i32 %F1 0
	call (i32*, i8*, n2) void @FILE_OPEN (%F2, @str, @READ_MODE)
	call (i32*, i8*, n2) void @FILE_OPEN (%F3, @str, @WRITE_MODE)
