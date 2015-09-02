# LLHD Assembly
This document describes the structure and syntax of the LLHD assembly language.

## Top Level

### `mod` Command
A module represents a subcircuit that assigns values to its outputs based on changes to the inputs. It consists of instructions that are not executed in sequence, but rather declare relationships between signals and other modules, processes, and functions. Only pure instructions are allowed within a module. The syntax looks as follows:

    mod <name> (<input signals>) (<output signals>) {
        <pure instructions>
    }

The instructions within the module are re-evaluated whenever their inputs change. The module is expected to use the `drv` instruciton to drive its output signals to a specific value, as follows:

    mod @not_gate (l1 %a) (l1 %b) {
        drv %b %a
    }

Modules are used by instantiating them within other modules by using the `inst` instruction as follows:

    mod @buffer_gate (l1 %a) (l1 %b) {
        %0 = sig l1
        inst @not_gate (%a) (%0)
        inst @not_gate (%0) (%b)
    }

The grammar for the `mod` command is as follows:

    # module grammar
    module_def  := "mod" name
                   "(" input'module_args? ")"
                   "(" output'module_args? ")"
                   "{" module_body "}"

    module_args := module_arg ("," module_arg)*
    module_arg  := type local_name

    module_body := module_ins*
    module_ins  := pure_ins | stateful_ins


### `proc` Command
A process represents a generalized subcircuit that assigns values to its outputs based on changes to the inputs. In contrast to a module, the instructions in a process are executed in sequence and may contain branches and conditional jumps. As such, a process shall be Turing-complete and thus allows any input-to-output relation to be described. All instructions are allowed within a process. The syntax looks as follows:

    proc <name> (<input signals>) (<output signals>) {
        <instructions>
    }

Whenever one of the processes input signals changes, its instructions are executed until either a `wait` instruction suspends or stops execution, or all instructions are executed. Thus a process can be in one of three states: *ready*, *suspended*, or *stopped*. Processes that are ready start execution at the first instruction. Processes that are suspended resume execution at the resume location indicated by the `wait` instruction, and stopped processes do not execute at all.

Process are expected to use the `drv` instruction to drive their output signals to a specific value, as follows:

    proc @latch (l1 %d, l1 %clk) (l1 %q) {
        %0 = cmp %clk l1'1
        br %0 if0, endif0
    if0:
        drv %q %d
    endif0:
    }

Processes are instantiated using the `inst` instruction as follows:

    mod @ff (l1 %d, l1 %clk) (l1 %q) {
        %p = sig l1
        %nclk = not %clk
        inst @latch (%d, %clk) (%p)
        inst @latch (%p, %nclk) (%q)
    }

The grammar for the `proc` command is as follows:

    # process grammar
    process_def  := "proc" name
                    "(" input'process_args? ")"
                    "(" output'process_args? ")"
                    "{" process_body "}"

    process_args := process_arg ("," process_arg)*
    process_arg  := type local_name

    process_body := process_ins*
    process_ins  := all_ins


### `func` Command
A function represents a sequence of instructions that map a set of input values to output values. In contrast to modules and processes, a function does not tie into the event queue and operates on values rather than signals. The output values are returned to the caller rather than driven onto a set of signals. All instructions are allowed within a function. The syntax is as follows:

    func <name> (<input arguments>) (<output arguments>) {
        <instructions>
    }

Instructions in a function are executed in sequence whenever the function is called. Stateful instructions are disallowed. A function must store a value in each of its output variables using the `st` instruction under all circumstances and paths of execution. For example:

    func @and_or (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %0 = and %a %b
        %1 = or %a %b
        st %x %0
        st %y %1
    }


A function is used through the `call` instruction, which executes the instructions in the function body and returns the output arguments as a result. For example:

    mod @and_or_gate (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %0, %1 = call @and_or (%a, %b)
        drv %x %0
        drv %y %1
    }

The grammar for the `func` command is as follows:

    # function grammar
    function_def := "func" name
                    "(" input'func_args? ")"
                    "(" output'func_args? ")"
                    "{" func_body "}"

    function_args := function_arg ("," function_arg)*
    function_arg  := type local_name

    function_body := function_ins*
    function_ins  := stateless_ins



## Instructions

### Types of Instructions

#### Pure vs. Impure Instructions
Pure functions influence the execution of other functions only by the value they return as a result. An instruction is considered impure if it also influences other instructions through side effects such as conditional execution or changing a memory location. The following instructions are considered *impure*:

-   memory load and store
-   branches and loops

#### Stateful Instructions
Instructions that maintain state are called *stateful*. For example, instantiating a module or process creates an state that is carried between multiple executions of the instruction. The following instructions are considered *stateful*:

-   instantiation of modules and processes
-   declaration of signals

Note that declaration of variables is considered *stateless*, since the lifetime of the memory allocated for the variable is tied to the lifetime of the enclosing module, process, or function.


### `st` Instruction [proc,func]
*Store to Memory*

    st <addr> <value>

Stores the value *<value>* at the memory location pointed to by *<addr>*.


### `ld` Instruction [proc,func]
*Load from Memory*

    <result> = ld <ty> <addr>

Loads a value of type *<ty>* from the memory location pointed to by *<addr>* and stores it in *<result>*.


### `wait` Instruction [proc]

    wait                ; (1) unconditional wait
    wait <time>         ; (2) relative timed wait
    wait abs <time>     ; (3) absolute timed wait
    wait <cond> <dest>  ; (4) conditional wait

The `wait` instruction is used to temporarily suspend the execution of a process. It comes in four variants differing in when and how execution of the process is resumed:

1.  The unconditional wait suspends execution of the process until it is resumed due to a change to its inputs.

2.  The relative timed wait suspends execution of the process until an amount of time has passed.

3.  The absolute timed wait suspends execution of the process until an absolute point in time has been reached.

4.  The conditional wait suspends execution of the process if a condition evaluates to 0. As soon as one of the inputs changes, execution of the process resumes at a different location.

Note that during a timed wait the process is insensitive to its inputs.

#### Examples

    wait 10ns        ; wait for 10ns
    wait abs 4580ns  ; wait until the sim reaches 4580ns

A process can be made to wait for a certain condition among its inputs to become true as follows:

    proc @foo (l1 %a) () {
    retest:
        %cond = cmp eq %a 0
        wait %cond %retest
        ; execution resumes here as soon as $a = 0
    }


### `br` Instruction [proc,func]
*Branch*

    br <dest>                       ; (1) unconditional branch
    br <cond>, <iftrue>, <iffalse>  ; (2) conditional branch

The `br` instruction transfers control flow to a different basic block. Two variants exist:

1.  The unconditional branch transfers control flow to another basic block.

2.  The conditional branch accepts an `i1` condition value and transfers control flow to one of two basic blocks based on whether that condition evaluates to 0 or 1.

#### Examples

        %cond = cmp eq %a 0
        br %cond, %ifTrue, %ifFalse
    ifTrue:
        ; control flow jumps here if %a = 0
    ifFalse:
        ; control flow jumps here if %a != 0


### `drv` Instruction [mod,proc]
*Drive Signal*

    drv <signal> <value>               ; (1) delta-step drive
    drv <signal> <value> <time>        ; (2) time-step drive
    drv <signal> clear <value>         ; (3) delta-step clearing drive
    drv <signal> clear <value> <time>  ; (4) time-step clearing drive

The `drv` instruction drives a signal to a specified value. It has two orthogonal options that result in four variations:

1.  The delta-step drive schedules a change in value for the driven signal at the next delta time step.

2.  The time-step drive schedules a change in value for the driven signal after an amount of time has passed.

3.  The delta-step clearing drive behaves the same as the delta-step drive, but clears all pending events on the driven signal before scheduling the change.

4.  The time-step clearing drive behaves the same as the time-step drive, but clears all pending events on the driven signal before scheduling the change.

The `clear` option allows for both an *inertia delay model* as well as a *transport delay model* to be expressed: In an inertia model, an input value has to be applied long enough for its effect to become visible at the output. If the change in input occurs for less than the delay time, it does not have enough inertia to change the output. In a transport model, the effects of a change to a gate's inputs becomes visible after a delay. Short pulses are preserved and appear at the output after the delay.

For example, the following inertial delay model causes a pulse of 5ns to be supressed since the "inertia" a pulse must have to have an effect is at least 10ns:

    ; 5ns pulse, causes %a = [0 @0ns]
    %a = sig l1
    drv %a l1'0
    drv %a l1'1 10ns
    wait 5ns
    drv %a clear l1'0 10ns

    ; 15ns pulse, causes %b = [0 @0ns, 1 @10ns, 0 @25ns]
    %b = sig l1
    drv %b l1'0
    drv %b l1'1 10ns
    wait 15ns
    drv %b clear l1'0 10ns

In contrast, the following transport delay model causes a pulse of 5ns to be propagated to the output, even though the gate is modelled as having a delay of 10ns:

    ; 5ns pulse, causes %c = [0 @0ns, 1 @10ns, 0 @15ns]
    %c = sig l1
    drv %c l1'0
    drv %c l1'1 10ns
    wait 5ns
    drv %c l1'0 10ns


### `sig` Instruction [mod,proc]
*Signal Declaration*

    <result> = sig <ty>            ; (1) uninitialized declaration
    <result> = sig <ty> <initial>  ; (2) initialized declaration

The `sig` instruction declares a new signal that is integrated into the event loop and may be driven using the `drv` instruction. Two variants of the instruction exist:

1.  The uninitialized declaration declares a new signal that is initially set to the undefined value of its type, or an arbitrary value if the type provides no representation of an undefined value.

2.  The initialized declaration declares a new signal that is set to an initial value.

Note that logic values provide a representation of an undefined value ("U"), whereas integer type do not. For types that do not provide an explicit representation of an undefined value, the initial value depends on the previous contents of the memory location used on the simulation machine. However, assembly that relies on an uninitialized signal assuming a specific value, or values with a specific random distribution, is not well-formed.

#### Examples

    %a = sig l1    ; %a = U
    %b = sig i1    ; %b = random{0,1}
    %c = sig l1 1  ; %c = 1
    %d = sig i1 1  ; %d = 1


### `alloc` Instruction [proc,func]
*Allocate Memory*

    <result> = alloc <ty>            ; (1) uninitialized allocation
    <result> = alloc <ty> <initial>  ; (2) initialized allocation

The `alloc` instruction allocates memory to hold a value of a specific type and returns the address to that memory. Two variants of the instruction exist:

1.  The uninitialized allocation allocates memory that is initially set to the undefined value of its type, or an arbitrary value if the type provides no representation of an undefined value.

2.  The initialized allocation allocates memory that is set to an initial value.

The same rules apply as with the `sig` instruction.


### `cmp` Instruction [mod,proc,func]
*Compare*

    <result> = icmp <op> <valueA> <valueB>

The `cmp` instruction compares two values of the same type and returns the result as an `i1` value indicating that the comparison yielded *true* or *false* as a result. The comparison performed depends on the value of the `<op>` field and can be one of the following:

-   `eq` equal
-   `ne` not equal
-   `sgt` signed greater than
-   `slt` signed less than
-   `sge` signed greater than or equal to
-   `sle` signed less than or equal to
-   `ugt` unsigned greater than
-   `ult` unsigned less than
-   `uge` unsigned greater than or equal to
-   `ule` unsigned less than or equal to

If the values have a logic type, additional rules have to be observed. If any bit is `U` (uninitialized), `X` (strong unknown), `Z` (high impedance), or `W` (weak unknown), the comparison returns false. If a bit is `-` (don't care) in one of the values, that bit is skipped in the comparison.


### `and` Instruction [mod,proc,func]
*Bitwise Logical AND*

    <result> = and <valueA> <valueB>

The `and` instruction performs a bitwise logical AND operation on two values of identical type. If the operands are of `iN` type, the result is of `iN` type as well. If the operands are of `lN ` or `lsN` type, the result is of `lsN` type.


### `or` Instruction [mod,proc,func]
*Bitwise Logical OR*

    <result> = or <valueA> <valueB>

The `or` instruction performs a bitwise logical OR operation on two values of identical type. If the operands are of `iN` type, the result is of `iN` type as well. If the operands are of `lN` or `lsN` type, the result is of `lsN` type.


### `xor` Instruction [mod,proc,func]
*Bitwise Logical XOR*

    <result> = xor <valueA> <valueB>

The `xor` instruction performs a bitwise logical XOR operation on two values of identical type. If the operands are of `iN` type, the result is of `iN` type as well. If the operands are of `lN` or `lsN` type, the result is of `lsN` type.


### `not` Instruction [mod,proc,func]
*Bitwise Logical NOT*

    <result> = not <value>

The `not` instruction performs a bitwise logical NOT operation on a value. If the operand is of `iN` type, the result is of `iN` type as well. If the operand is of `lN` or `lsN` type, the result is of `lsN` type.


### `add` Instruction [mod,proc,func]
*Addition*

    <result> = add <valueA> <valueB>

The `add` instruction arithmetically adds two operands of the same type. The result has the same type as the operands. This is a modulo addition, meaning that the result is truncated to fit into the same type as the operands, thus causing over- and underflow.


### `sub` Instruction [mod,proc,func]
*Subtraction*

    <result> = add <valueA> <valueB>

the `sub` instruction arithmetically subtracts one operand from another operand of the same type. The result has the same type as the operands. This is a modulo subtraction, meaning that the result is truncated to fit into the same type as the operands, thus causing over- and underflow.


### `mul` Instruction [mod,proc,func]
*Multiplication*

    <result> = mul <sign> <valueA> <valueB>

The `mul` instruction arithmetically multiplies two operands of the same type. The result has the same type as the operands. This is a modulo multiplication, meaning that the result is truncated to fit into the same type as the operands, thus causing over- and underflow. The `<sign>` argument determines the way the instruction handles signedness:

-   `signed` causes the instruction to treat the operands as signed values such that opposing signs yield a negative result.
-   `unsigned` causes the instruction to simply multiply the operands.


### `div` Instruction [mod,proc,func]
*Division*

    <result> = div <sign> <valueA> <valueB>  ; (1) division
    <result> = div mod <valueA> <valueB>     ; (2) modulo
    <result> = div rem <valueA> <valueB>     ; (3) remainder

The `div` instruction performs operations related to arithmetic division. Both operands and the result have the same type. It comes in three different flavors:

1.  The division instruction divides the first operand by the second, discarding any fractional part. The `<sign>` argument determines the way the instruction handles signedness:
    - `signed` causes the instruction to treat the operands as signed values such that opoosing signs yield a negative result.
    - `unsigned` causes the instruction to simply divide the first operand by the latter.

2.  The modulo instruction calculates `valueA mod valueB`. The result is a value in the range [0,valueB-1].

3.  The remainder instruction calculates `valueA rem valueB`. The result is a value in the range [-valueB+1,valueB-1].

If the second operand is 0, the result is undefined. This has different implications depending on the type of the operation. If the operands are integers, this is a fatal error that aborts execution. If the operands are logic values, the result is a logic value with every bit set to `X`.

#### Examples

    %0 = div unsigned 7 2  ; %0 = 3
    %1 = div signed 7 -2   ; %1 = -3
    %2 = div mod -21 4     ; %2 = 3
    %3 = div mod  21 4     ; %3 = 3
    %4 = div rem -21 4     ; %4 = -1
    %5 = div rem  21 4     ; %5 = 3


### `bsel` Instruction []
### `bcat` Instruction []
### `trunc` Instruction []
### `ext` Instruction []


### `lmap` Instruction [mod,proc,func]
*Logic Map*

    <result> = lmap <ty> <value>

The `lmap` operation maps a general logic value to a strong logic value, and vice versa. The function comes in two variants, as dictated by the requested output type `<ty>`:

-   `l<N>` maps the value to a general logic value if it is of type `ls<N>`.
-   `ls<N>` maps the value to a strong logic value if it is of type `l<N>`.

If the value is already of type `<ty>`, the function simply returns the value.

#### Examples

    %0 = lmap ls9 l9'UX01ZWLH-  ; %0 = ls9'UX01XX01X
    %1 = lmap l4 ls4'UX01       ; %1 = l4'UX01



## Types

### Integer Types
The integer types represent a generic integer of an arbitrary bit length. The integer is interpreted as signed or unsigned depending on the instructions used.

    iN  ; N integer bits


### Logic Types
The logic types are modeled after the IEEE 1164-1993 standard which represents a logic value that results from a digital circuit as one of nine possible values:

-   `U` uninitialized
-   `X` forcing unknown value
-   `0` forcing 0
-   `1` forcing 1
-   `Z` high impedance
-   `W` weak unknown value
-   `L` weak 0
-   `H` weak 1
-   `-` don't care

The LLHD assembly language defines two logic types:

    lN   ; (1) N logic bits with values in [UX01ZWLH-]
    lsN  ; (2) N logic bits with values in [UX01]

The first type **(1)** is the general logic type which may assume any of the nine values represented by the IEEE standard. The second type **(2)** is the strong logic type which may assume any of the four strong values. The distinction between the two types is made to allow for better optimizations in simulation since the `lsN` type may be represented as 2 bits. A logic gate will always produce an `lsN` as a result, since its output transistors cause a strong drive. If logic gates are chained together in sequence, intermediate results will not assume the full range of logic values anymore, but only one of the four the previous gate may produce.

#### Mappings between `lN` and `lsN`
The mapping from a `l1` value to a `ls1` value is defined as follows:

    U X 0 1 Z W L H -
    -----------------
    U X 0 1 X X 0 1 X

The mapping from a `ls1` value to a `l1` value is defined as follows:

    U X 0 1
    -------
    U X 0 1

#### AND Truth Table
The logic and operation on two `l1` values is defined as described in the table below. The operation is defined in a reduced form when applied to two `ls1` values.

      | U X 0 1 Z W L H -
    --+------------------
    U | U U 0 U U U 0 U U
    X | U X 0 X X X 0 X X
    0 | 0 0 0 0 0 0 0 0 0
    1 | U X 0 1 X X 0 1 X
    Z | U X 0 X X X 0 X X
    W | U X 0 X X X 0 X X
    L | 0 0 0 0 0 0 0 0 0
    H | U X 0 1 X X 0 1 X
    - | U X 0 X X X 0 X X

#### OR Truth Table
The logic or operation on two `l1` values is defined as described in the table below. The operation is defined in a reduced form when applied to two `ls1` values.

      | U X 0 1 Z W L H -
    --+------------------
    U | U U U 1 U U U 1 U
    X | U X X 1 X X X 1 X
    0 | U X 0 1 X X X 1 X
    1 | 1 1 1 1 1 1 1 1 1
    Z | U X X 1 X X X 1 X
    W | U X X 1 X X X 1 X
    L | 1 X X 1 X X 0 1 X
    H | 1 1 1 1 1 1 1 1 1
    - | U X X 1 X X X 1 X

#### XOR Truth Table
The logic xor operation on two `l1` values is defined as described in the table below. The operation is defined in a reduced form when applied to two `ls1` values.

      | U X 0 1 Z W L H -
    --+------------------
    U | U U U U U U U U U
    X | U X X X X X X X X
    0 | U X 0 1 X X 0 1 X
    1 | U X 1 0 X X 1 0 X
    Z | U X X X X X X X X
    W | U X X X X X X X X
    L | U X 0 1 X X 0 1 X
    H | U X 1 0 X X 1 0 X
    - | U X X X X X X X X

#### NOT Truth Table
The logic not operation on a `l1` value is defined as described in the table below. The operation is defined in a reduced form when applied to a `ls1` value.

    U X 0 1 Z W L H -
    -----------------
    U X 1 0 X X 1 0 X


### Pointer Type
The pointer type represents the address of a value in memory that is of a known type. It is formed by suffixing any type with an asterisk, as follows:

    T *  ; address of a value of type T

Pointer types are primarily used for loading and storing values. They do not have any correspondence in actual hardware.


### Future Extensions

    <N x T>          ; vector of N values of type T
    { T1, T2, ... }  ; struct of values of type T1, T2, ...


### Grammar
The grammar of the LLHD assembly types is as follows:

    type := integer_type
          | logic_type
          | strong_logic_type
          | pointer_type

    pointer_type      := type "*"
    vector_type       := "<" /[0-9]+/ "x" type ">"
    struct_type       := "{" type ("," type)* "}"

    integer_type      := /i[1-9][0-9]*/
    logic_type        := /l[1-9][0-9]*/
    strong_logic_type := /ls[1-9][0-9]*/



## Structure

### Instructions
At the most basic level, LLHD assembly consists of instructions that are executed in the order in which they appear. They may have none, one, or multiple return values that are stored in local variables.

    br %a
    %0 = add %a %b
    %1, %2 = call %func (%a, %b)

This makes it easy to identify dependencies since all written variables appear on the left of the "=" sign, and all read variables appear on the right.

All instructions start with the instruction name followed by the non-optional arguments. All subsequent optional arguments are prefixed with a "," to facilitate parsing.


### Modules
Hierarchy and encapsulation is provided through the notion of modules, which are groups of instructions that act on a set of input signals and modify a set of output signals in return. Upon instantiation, multiple modules may produce output values for the same signal. The signal's final value is the result of a conflict resolution step that combines all output values.

    module %foo (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %0 = and %a %b
        %1 = or %a %b
        drv %x %0
        drv %y %1
    }

Modules may be instantiated with the `inst` instruction which takes the module name, the set of input signals, and the set of output signals as arguments.

    inst %foo (%i, %j) (%u, %v)

A module is reevaluated whenever one of its input signals changes, which means that execution resumes from where it was suspended. If the module is finished or has not yet run, execution starts from the top. Execution may be suspended through the use of the `wait` instruction. Note that the `inst`, `sig`, and `alloc` instructions are executed when the module is instantiated, and are skipped during regular execution. The wait instruction requires a boolean argument that indicates whether the wait should happen, and a jump-back label pointing to where execution should resume in order to reevaluate the wait condition.

A module may make use of the `time` instruction, which always evaluates to the current simulation time. Modules that make use of this instruction have their sensitivity list extended by the simulation time, which makes the reevaluate after every simulation step. However, since the wait instruction in combination with its jump-back label defines the block of instructions that influence wait condition, a simple optimization would be to disable all entries in the sensitivity list that are not read in the current wait's block.

    module %clkgen () (l1 %clk) {
        drv %clk l1'1;
    wait0:
        %0 = time
        %1 = cmpge %1 500ps
        wait %1 wait0

        drv %clk l1'0;
    wait1:
        %2 = time
        %3 = cmpge %2 1000ps
        wait %3 wait1
    }


### Functions
Functions act in a similar way to modules, but act on variables rather than on signals. This allows common sequences of instructions to be factored into separate blocks.

    func %bar (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %x = and %a %b
        %y = or %a %b
    }

Functions must be called with the `call` instruction for them to be evaluated.

    %u, %v = call %bar (%i, %j)


### Variables
Variables can only be assigned once, thus making the entire LLHD assembly of single-assignment form. This makes the code easier to reason about.

#### Signals
Signals are represented as a special kind of variable that allows the `drv` instruction to act upon it. Signals have to be declared before they can be driven:

    %a = sig l1
    drv %a %0

#### Memory
Memory is represented as a special kind of variable that allows reading through the `ld` and writing through the `st` instruction. This allows for loops and stateful variables even though the assembly at its core is in single-assignment form. Memory has to be allocated before it can be used:

    %a = alloc l1
    st %a %0
    %0 = ld %a


### Features
Instructions, modules, functions, and variables carry a set of feature flags that indicate what parts of the assembly language are used. This includes flags that indicate the use of branches, loops, memory, signals, function calls, implicit storage, and the like. A module's feature flags equals the feature flags of all its instructions ORed together. It should be possible to transform assembly into a representation that has certain feature flags cleared. This shifts part of the burden of making sense of a design assembly to the LLHD library.

For example, a combinatorial process in a piece VHDL code might be translated into an LLHD assembly module that contains a loop in order for the code to be as compact as possible. A synthesizer might not want to cope with this kind of optimization and thus requests a transformed version of the assembly that has no loops. This would transform the design into a representation without any loops, potentially at the expense of a lot of code.

Another example concerns implicit storage elements. When a module reacts to a change in its input values and does not issue a `drv` instruction in response, it effectively retains the previously driven state. This is an implicit storage device, since only certain combinations of input signals cause a change of the output signal. A synthesizer might not want to make the effort to analyze modules for their implicit storage devices in order to determine the registers a design requires. It thus requests a version of the assembly that contains explicit storage elements, where `drv` instructions are replaced with `reg` instructions that are always executed, thus making it easier to analyze the kind of storage element in use.



## HDL Examples
This section outlines the mapping of common HDL structures to LLHD.

### Structural SR Latch
**VHDL:**

    entity nor_gate is
        port (a,b: in bit;
              z: out bit);
    end nor_gate;

    architecture structural of nor_gate is
    begin
        z <= a nor b;
    end structural;

    entity latch is
        port (s,r: in bit;
              q,nq: out bit);
    end latch;

    architecture structural of latch is
        component nor_gate
            port (a,b: in bit;
                  z: out bit);
        end component;
    begin
        n1: nor_gate port map (r,nq,q);
        n2: nor_gate port map (s,q,nq);
    end structural;

**LLHD:**

    module %nor_gate (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        %n1 = inst %nor_gate (%r, %nq) (%q)
        %n2 = inst %nor_gate (%s, %q) (%nq)
    }


### Data Flow SR Latch
**VHDL:**

    entity latch is
        port (s,r: in bit,
              q,nq: out bit);
    end latch;

    architecture dataflow of latch is
    begin
        q <= r nor nq;
        nq <= s nor q;
    end dataflow;

**LLHD:**

    module %latch_p0 (u1 %r, u1 %nq) (u1 %q) {
        %0 = nor %r %nq
        drv %q %0
    }

    module %latch_p1 (u1 %s, u1 %q) (u1 %nq) {
        %0 = nor %s %q
        drv %nq %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        inst %latch_p0 (%r, %nq) (%q)
        inst %latch_p1 (%s, %q) (%nq)
    }

An optimization step can be performed that collapses structurally identical modules into one:

    module %latch_p0 (u1 %r, u1 %nq) (u1 %q) {
        %0 = nor %r %nq
        drv %q %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        inst %latch_p0 (%r, %nq) (%q)
        inst %latch_p0 (%s, %q) (%nq)
    }


### Inertial Delay vs. Transport Delay
**VHDL:**

    entity nor_gate is
        port (a,b: in bit,
              z: out bit);
    end nor_gate;

    architecture idm of nor_gate is
    begin
        z <= a nor b after 1ns;
    end idm;

    architecture tdm of nor_gate is
    begin
        z <= transport a nor b after 1ns;
    end tdm;

**LLHD:** The distinction between intertial and transport delay model shall be made in the `drv` instruction. By prepending the value with the `clear` keyword, all pending events on `%z` are cleared. If `%0` assumes a transient value that lasts less than 1ns, the first change schedules an event to the new value after 1ns, and the second change schedules an event to the old value. If the `clear` keyword is set, the event scheduled by the first change shall be cancelled, thus effectively supressing the value change.

    module @nor_gate_idm (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z clear %0 1ns ; clears any pending events
    }

    module @nor_gate_tdm (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z %0 1ns
    }


### Counter
**VHDL:**

    entity counter is
        port (clk: in bit);
    end counter;

    -- Counter in Variable
    architecture var of counter is
    begin
        count: process (clk)
            variable cnt : integer := 0;
        begin
            if (clk = '1' and clk'last_value = '0') then
                cnt := cnt + 1;
            end if;
        end process;
    end var;

    -- Counter in Signal
    architecture sig of counter is
        signal cnt : integer := 0;
    begin
        count: process (clk)
        begin
            if (clk = '1' and clk'last_value = '0') then
                cnt <= cnt + 1;
            end if;
        end process;
    end sig;

**LLHD:**

    module @counter_var (u1 %clk) () {
        %clk_last_value = alloc u1
        %cnt = alloc i32 i32'0

        %0 = cmpeq %clk u1'1
        %1 = cmpeq %clk_last_value u1'0
        %2 = and %0 %1

        br %2, if_0, endif_0
    if_0:
        %3 = add %cnt i32'1
        st %cnt %3
    endif_0:

        st %clk_last_value %clk
    }

    module @counter_sig (u1 %clk) () {
        %clk_last_value = alloc u1
        %cnt = sig i32 i32'0

        %0 = cmpeq %clk u1'1
        %1 = cmpeq %clk_last_value u1'0
        %2 = and %0 %1

        br %2, if_0, endif_0
    if_0:
        %3 = add %cnt i32'1
        drv %cnt %3
    endif_0:

        st %clk_last_value %clk
    }

An optimization pass is possible that makes the clock edge detection explicit:

    module @counter_var (u1 %clk) () {
        %clk_edge = edge u1'0 u1'1 %clk
        %cnt = alloc i32 i32'0

        br %clk_edge, if_0, endif_0
    if_0:
        %0 = add %cnt i32'1
        st %cnt %0
    endif_0:
    }

    module @counter_sig (u1 %clk) () {
        %clk_edge = edge u1'0 u1'1 %clk
        %cnt = sig i32 i32'0

        br %clk_edge, if_0, endif_0
    if_0:
        %0 = add %cnt i32'1
        drv %cnt %0
    endif_0:
    }
