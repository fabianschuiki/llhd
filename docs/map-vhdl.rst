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

    @str = const [9 x i8] "test.dat"
    @READ_MODE = const n2 0
    @WRITE_MODE = const n2 1
    decl func void @FILE_OPEN (i32*, i8*, n2)

    %F1 = var i32
    %F2 = var i32
    %F3 = var i32
    store i32 %F1 0
    call void @FILE_OPEN (i32* %F2, i8* @str, n2 @READ_MODE)
    call void @FILE_OPEN (i32* %F3, i8* @str, n2 @WRITE_MODE)


Sequential Statements
---------------------

Wait Statement:

.. code-block:: vhdl

    -- architecture
    signal A: INTEGER range 1 to 5;
    -- process
    wait on A;                   -- (A)
    wait until A > 2;            -- (B) implies `on A`
    loop                         -- (C) identical to `wait until A > 2;`
        wait on A;
        exit when A > 2;
    end loop;
    wait until A > 2 for 10 ns;  -- (D) implies `on A`
    wait until TRUE;             -- (E) identical to `wait;`
    wait;                        -- (F)

.. code-block:: llhd

    %A = sig i3

        ; (A)
        wait %bb0 (%A)
    bb0:

        ; (B,C)
        wait %bb1 (%A)
    bb1:
        %0 = probe i3$ %A
        %1 = cmp sgt i3 %0 2
        br %1 label %bb2 %bb0
    bb2:

        ; (D)
        %now = now
        %timeout = add time %now, 10ns
        br label %bb3
    bb3:
        wait %bb4 (%A) until %timeout
    bb4:
        %0 = probe i3$ %A
        %1 = cmp sgt i3 %0 2
        %now2 = now
        %2 = cmp eq time %timeout, %now2
        %3 = or i1 %2 %3
        br %3 label %bb5 %bb3
    bb5:

        ; (E,F)
        wait %bb6
    bb6:

Assertion Statement:

.. code-block:: vhdl

    signal A, B: INTEGER range -8 to 7;
    assert A > B report "Invalid value " & INTEGER'IMAGE(A) severity ERROR;

.. code-block:: llhd

    @str = const [15 x i8] "Invalid value "
    %ERROR = const n3 0
    %A = sig i4
    %B = sig i4

        %a = probe i4$ %A
        %b = probe i4$ %B
        %condition = cmp sgt i4 %a %b
        br %condition label %AssertTrue %AssertFail
    AssertFail:
        %image = call i8* @builtin_integer_image_i4 (i4 %a)
        %message = call i8* @builtin_strcat (i8* @str, i8* %image)
        call void @builtin_assert_trap (i8* %message, n3 %ERROR)
        free %message
        free %image
    AssertPass:

Simple Signal Assignment:

.. code-block:: vhdl

    -- Inertial delay:
    -- The following three assignments are equivalent to each other:
    Output_pin <= Input_pin after 10 ns; -- (A)
    Output_pin <= inertial Input_pin after 10 ns; -- (A)
    Output_pin <= reject 10 ns inertial Input_pin after 10 ns; -- (A)
    -- Assignments with a pulse rejection limit less than the time expression:
    Output_pin <= reject 5 ns inertial Input_pin after 10 ns; -- (B)
    Output_pin <= reject 5 ns inertial Input_pin after 10 ns,
                                       not Input_pin after 20 ns; -- (B)

    -- Transport delay:
    Output_pin <= transport Input_pin after 10 ns; -- (C)
    Output_pin <= transport Input_pin after 10 ns,
                            not Input_pin after 20 ns; -- (D)
    -- Their equivalent assignments:
    Output_pin <= reject 0 ns inertial Input_pin after 10 ns; -- (C)
    Output_pin <= reject 0 ns inertial Input_pin after 10 ns,
                                       not Input_pin after 20 ns; -- (D)

.. code-block:: llhd

    %Output_pin = sig l8
    %Input_pin = sig l8

    ; (A)
    %0 = probe l8 %Input_pin
    drive l8 %Output_pin clear 0ns, 10ns %0
    ; (B)
    %0 = probe l8$ %Input_pin
    %1 = not l8 %0
    drive l8 %Output_pin clear 5ns, 10ns %0
    drive l8 %Output_pin clear 5ns, 10ns %0, 20ns %1
    ; (C)
    %0 = probe l8$ %Input_pin
    drive l8 %Output_pin clear 10ns, 10ns %0
    ; (D)
    %0 = probe l8$ %Input_pin
    %1 = not l8 %0
    drive l8 %Output_pin clear 10ns, 10ns %0, 20ns %1

Conditional Signal Assignment:

.. code-block:: vhdl

    S <= unaffected when A = 42 else A after Buffer_Delay;

.. code-block:: llhd

    %S = sig i8
    %A = sig i8
    %Buffer_Delay = const time 10ns

        %0 = probe i8 %A
        %1 = cmp eq i8 %0 42
        br %1 label %Drive %Skip
    Drive:
        drive i8 %S clear 0ns, %Buffer_Delay %0
    Skip:

Concurrent Procedure Call:

.. code-block:: vhdl

    -- A concurrent procedure call statement. (A)
    CheckTiming (tPLH, tPHL, Clk, D, Q);
    -- The equivalent process. (B)
    process
    begin
        CheckTiming (tPLH, tPHL, Clk, D, Q);
        wait on Clk, D, Q;
    end process;

.. code-block:: llhd

    decl func void @CheckTiming (time, time, l1$, l8$, l8$)
    @tPLH = const time 5ns
    @tPHL = const time 4ns
    %Clk = sig l1
    %D = sig l8
    %Q = sig l8

    proc %proc0 (l1$ %Clk, l8$ %D, l8$ %Q) {
    entry:
        call void @CheckTiming (@tPLH, @tPHL, %Clk, %D, %Q)
        wait %entry (%Clk, %D, %Q)
    }
