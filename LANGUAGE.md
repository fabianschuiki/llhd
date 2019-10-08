# LLHD Language Reference

This document specifies the low-level hardware description language. It outlines the architecture, structure, and instruction set, and provides usage examples.

@[toc]

---


## Modules

At the root of the LLHD hierarchy, a module represents an entire design. It is equivalent to one single LLHD assembly file on disk, or one in-memory design graph. Modules consist of functions, processes, entities, and external unit declarations as outlined in the following sections. Two or more modules can be combined using the linker, which substitutes external declarations (`declare ...`) with an actual unit definition. A module is called *self-contained* if it contains no external unit declarations.


## Names

Names in LLHD follow a scheme similar to LLVM. The language distinguishes between global names, local names, and anonymous names. Global names are visible outside of the module. Local names are visible only within the module, function, process, or entity they are defined in. Anonymous names are purely numeric local names whose numbering is not preserved across IR in-memory and on-disk representations.

Example | Regex                | Description
------- | -------------------- | ---
`@foo`  | `@[a-zA-Z0-9_\.\\]+` | Global name visible outside of the module, function, process, or entity.
`%foo`  | `%[a-zA-Z0-9_\.\\]+` | Local name visible only within module, function, process, or entity.
`%42`   | `%[0-9]+`            | Anonymous local name.

Names are UTF-8 encoded. Arbitrary code points beyond letters and numbers may be represented as sequences of `\xx` bytes, where `xx` is the lower- or uppercase hexadecimal representation of the byte. E.g. the local name `foo$bar` is encoded as `%foo\24bar`.


## Units

Designs in LLHD are represented by three different constructs (called "units"): functions, processes, and entities. These capture different concerns arising from the need to model silicon hardware, and is in contrast to IRs targeting machine code generation, which generally only consist of functions.

The language differentiates between how instructions are executed in a unit:

- *Control-flow* units consist of basic blocks, where execution follows a clear control-flow path. This is equivalent to what one would find LLVM's IR.
- *Data-flow* units consist only of an unordered set of instructions which form a data-flow graph. Execution of instructions is implied by the propagation of value changes through the graph.

Furthermore it differentiates how time passes during the execution of a unit:

- *Immediate* units execute in zero time. They may not contain any instructions that suspend execution or manipulate signals. These units are ephemeral in the sense that their execution starts and terminates in between time steps. As such no immediate units coexist or persist across time steps.
- *Timed* units coexist and persist during the entire execution of the IR. They represent reactions to changes in signals and may suspend execution or interact with signals (probe value, schedule state changes).

The following table provides an overview of the three IR units, which are detailed in the following sections:

Unit         | Paradigm     | Timing    | Models
------------ | ------------ | --------- | ---
**Function** | control-flow | immediate | Ephemeral computation in zero time
**Process**  | control-flow | timed     | Behavioural circuit description
**Entity**   | data-flow    | timed     | Structural circuit description


### Functions

Functions represent *control-flow* executing *immediately* and consist of a sequence of basic blocks and instructions:

    func <name> (<ty1> <arg1>, ...) <retty> {
        <bb1>
        ...
        <bbN>
    }

A function has a local or global name, input arguments, and a return type. The first basic block in a function is the entry block. Functions must contain at least one basic block. Terminator instructions may either branch to another basic block or must be the `ret` instruction. The argument to `ret` must be of the return type `<retty>`. Functions are called using the `call` instruction. Functions may not contain instructions that suspend execution (`wait` and `halt`), may not interact with signals (`prb`, `drv`, `sig`), and may not instantiate entities/processes (`inst`).

#### Example

The following function computes the Fibonacci series for a 32 bit signed integer number N:

    func @fib (i32 %N) i32 {
    %entry:
        %one = const i32 1
        %0 = sle i32 %N, %one
        br %0, %recursive, %base
    %base:
        ret i32 %one
    %recursive:
        %two = const i32 2
        %1 = sub i32 %N, %one
        %2 = sub i32 %N, %two
        %3 = call i32 @fib (i32 %1)
        %4 = call i32 @fib (i32 %2)
        %5 = add i32 %3, %4
        ret i32 %5
    }


### Processes

Processes represent *control-flow* executing in a *timed* fashion and consist of a sequence of basic blocks and instructions. They are used to represent a procedural description of a how a circuit's output signals change in reaction to changing input signals.

    proc <name> (<in_ty1> <in_arg1>, ...) -> (<out_ty1> <out_arg1>, ...) {
        <bb1>
        ...
        <bbN>
    }

A process has a local or global name, input arguments, and output arguments. Input arguments may be used with the `prb` instruction. Output arguments must be of signal type (`T$`) and may be used with the `drv` instruction. The first basic block in a process is the entry block. Processes must contain at least one basic block. Terminator instructions may either branch to another basic block or must be the `halt` instruction. Processes are instantiated in entities using the `inst` instruction. Processes may not contain instructions that return execution (`ret`) and may not instantiate entities/processes (`inst`).

Processes may be used to behaviorally model a circuit, as is commonly done in higher-level hardware description languages such as SystemVerilog or VHDL. As such they may represent a richer and more abstract set of behaviors beyond what actual hardware can achieve. One of the tasks of a synthesizer is to transform processes into entities, resolving implicitly modeled state-keeping elements and combinatorial transfer functions into explicit register and gate instances. LLHD aims to provide a standard way for such transformations to occur.

#### Example

The following process computes the butterfly operation in an FFT combinatorially with a 1ns delay:

    proc @bfly (i32$ %x0, i32$ %x1) -> (i32$ %y0, i32$ %y1) {
    %entry:
        %x0v = prb i32$ %x0
        %x1v = prb i32$ %x1
        %0 = add i32 %x0v, %x1v
        %1 = sub i32 %x0v, %x1v
        %d = const time 1ns
        drv i32$ %y0, %0, %d
        drv i32$ %y1, %1, %d
        wait %entry, %x0, %x1
    }


### Entities

Processes represent *data-flow* executing in a *timed* fashion and consist of a set of instructions. They are used to represent hierarchy in a design, as well as a data-flow description of how a circuit's output signals change in reaction to changing input signals.

    entity <name> (<in_ty1> <in_arg1>, ...) -> (<out_ty1> <out_arg1>, ...) {
        <inst1>
        ...
        <instN>
    }

Eventually every design consists of at least one top-level entity, which may in turn call functions or instantiate processes and entities to form a design hierarchy. There are no basic blocks in an entity. All instructions are considered to execute in a schedule implicitly defined by their data dependencies. Dependency cycles are forbidden (except for the ones formed by probing and driving a signal). The order of instructions is purely cosmetic and does not affect behaviour.

#### Example

The following entity computes the butterfly operation in an FFT combinatorially with a 1ns delay:

    entity @bfly (i32$ %x0, i32$ %x1) -> (i32$ %y0, i32$ %y1) {
        %x0v = prb i32$ %x0
        %x1v = prb i32$ %x1
        %0 = add i32 %x0v, %x1v
        %1 = sub i32 %x0v, %x1v
        %d = const time 1ns
        drv i32$ %y0, %0, %d
        drv i32$ %y1, %1, %d
    }


### External Units

External units allow an LLHD module to refer to functions, processes, and entities declared outside of the module itself. The linker can then be used to resolve these declarations to actual definitions in another module.

    declare <name> (<in_ty1>, ...) <retty>              ; function declaration
    declare <name> (<in_ty1>, ...) -> (<out_ty1>, ...)  ; process/entity declaration


### Basic Blocks

A basic block has a name and consists of a sequence of instructions. The last instruction must be a terminator; all other instructions must *not* be a terminator. This ensures that no control flow transfer occurs within a basic block, but rather control enters at the top and leaves at the bottom. A basic block may not be empty. Functions and processes contain at least one basic block.

    %<bb_name>:
        <inst1>
        ...
        <instN>
        <terminator>


## Type System


### Overview

The following table shows the types available in LLHD. These are outlined in more detail in the following sections.

Type            | Description
--------------- | ---
`void`          | The unit type (e.g. instruction that yields no result).
`time`          | A simulation time value.
`iN`            | Integer of `N` bits, signed or unsigned.
`nN`            | Enumeration of `N` distinct values.
`lN`            | Logical value of `N` bits (IEEE 1164).
`T*`            | Pointer to a value of type `T`.
`T$`            | Signal of a value of type `T`.
`[N x T]`       | Array containing `N` elements of type `T`.
`{T1,T1,...}`   | Structured data containing fields of types `T0`, `T1`, etc.


### Void Type (`void`)

The `void` type is used to represent the absence of a value. Instructions that do not return a value are of type `void`. There is no way to construct a `void` value.


### Time Type (`time`)

The `time` type represents a simulation time value as a combination of a real time value in seconds, a delta value representing infinitesimal time steps, and an epsilon value representing an absolute time slot within a delta step (used to model SystemVerilog scheduling regions). It may be constructed using the `const time` instruction, for example:

    %0 = const time 1ns 2d 3e


### Integer Type (`iN`)

The `iN` type represents an integer value of `N` bits, where `N` can be any non-zero positive number. There is no sign associate with integer values. Rather, separate instructions are available to perform signed and unsigned operations, where applicable. Integer values may be constructed using the `const iN` instruction, for example:

    %0 = const i1 1
    %1 = const i32 9001
    %2 = const i1234 42


### Enumeration Type (`nN`)

The `nN` type represents an enumeration value which may take one of `N` distinct states. This type is useful for modeling sum types such as the enumerations in VHDL, and may allow for more detailed circuit analysis due to the non-power-of-two number of states the value can take. The values for `nN` range from `0` to `N-1`. Enumeration values may be constructed using the `const nN` instruction, for example:

    %0 = const n1 0  ; 0 is the only state in n1
    %1 = const n4 3  ; 3 is the last state in n4


### Logic Type (`lN`)

The `lN` type represents a collection of `N` wires each carrying one of the nine logic values defined by IEEE 1164. This type is useful to model the actual behavior of a logic circuit, where individual bits may be in other states than just `0` and `1`:

| Symbol | Meaning
| ------ | --------------------------------- |
| `U`    | uninitialized                     |
| `X`    | strong drive, unknown logic value |
| `0`    | strong drive, logic zero          |
| `1`    | strong drive, logic one           |
| `Z`    | high impedance                    |
| `W`    | weak drive, unknown logic value   |
| `L`    | weak drive, logic zero            |
| `H`    | weak drive, logic one             |
| `-`    | don't care                        |

This type allows for the modeling of high-impedance and wired-AND/-OR signal lines. It is not directly used in arithmetic, but rather various conversion instructions should be used to translate between `lN` and the equivalent `iN`, explicitly handling states not representable in `iN`. Typically this would involve mapping an addition result to `X` when any of the input bits is `X`. Logic values may be constructed using the `const lN` instruction, for example:

    %0 = const l1 "U"
    %1 = const l8 "01XZHWLU"


### Pointer Type (`T*`)

The `T*` type represents a pointer to a memory location which holds a value of type `T`. LLHD offers a very limited memory model where pointers may be used to load and store data in distinct memory slots. No bit casts or reinterpretation casts are possible. Pointers are obtained by allocating variables on the stack, which may then be accessed by load and store instructions:

    %init = const i8 42
    %ptr = var i8 %init
    %0 = ld i8* %ptr
    %1 = mul i8 %0, %0
    st i8* %ptr, %1

*Note:* It is not yet clear whether LLHD will provide `alloc` and `free` instructions to create and destroy memory slots in an infinite heap data structure.


### Signal Type (`T$`)

The `T$` type represents a physical signal which carries a value of type `T`. Signals correspond directly to wires in a physical design, and are used to model propagation delays and timing. Signals are used to carry values across time steps in the LLHD execution model. Signals are obtained by creating them in an entity, which may then be probed for the current value and driven to a new value:

    %init = const i8 42
    %wire = sig i8 %init
    %0 = prb i8$ %wire
    %1 = mul i8 %0, %0
    %1ns = const time 1ns
    drv i8$ %wire, %1, %1ns


### Array Type (`[N x T]`)

The `[N x T]` type represents a collection of `N` values of type `T`, where `N` can be any positive number, including zero. All elements of an array have the same type. An array may be constructed using the `[...]` instruction:

    %0 = const i16 1
    %1 = const i16 42
    %2 = const i16 9001
    %3 = [i16 %0, i16 %1, i16 %2]  ; [1, 42, 9001]
    %4 = [3 x i16 %0]              ; [1, 1, 1]

Individual values may be obtained or modified with the `extf`/`insf` instructions. Subranges of the array may be obtained or modified with the `exts`/`inss` instructions.


### Struct Type (`{T0,T1,...}`)

The `{T0,T1,...}` type represents a struct of field types `T0`, `T1`, etc. Fields in LLHD structs are unnamed and accessed by their respective index, starting from zero. A struct may be constructed using the `{...}` instruction:

    %0 = const i1 1
    %1 = const i8 42
    %2 = const time 10ns
    %3 = {i1 %0, i8 %1, time %2}  ; {1, 42, 10ns}

Individual fields may be obtained or modified with the `extf`/`insf` instructions.


## Instructions

### Overview

The following table shows the full instruction set of LLHD. Instructions have limitations as to whether they can appear in an function ("F"), process (P), or entity ("E").

Instruction                 | In    | Description
--------------------------- | ----- | ---
**Values**                  |       |
`const`                     | F P E | Construct a constant value
`alias`                     | F P E | Assign a new name to a value
`[...]`                     | F P E | Construct an array
`{...}`                     | F P E | Construct a struct
`insf` `inss`               | F P E | Insert elements, fields, or bits
`extf` `exts`               | F P E | Extract elements, fields, or bits
`mux`                       | F P E | Choose from an array of values
**Bitwise**                 |       |
`not`                       | F P E | Unary logic
`and` `or` `xor`            | F P E | Binary logic
`shl` `shr`                 | F P E | Shift left or right
**Arithmetic**              |       |
`neg`                       | F P E | Unary arithmetic
`add` `sub`                 | F P E | Binary arithmetic
`smul` `sdiv` `smod` `srem` | F P E | Binary signed arithmetic
`umul` `udiv` `umod` `urem` | F P E | Binary unsigned arithmetic
**Comparison**              |       |
`eq` `neq`                  | F P E | Equality operators
`slt` `sgt` `sle` `sge`     | F P E | Signed relational operators
`ult` `ugt` `ule` `uge`     | F P E | Unsigned relational operators
**Control Flow**            |       |
`phi`                       | F P   | Reconvergence node
`br`                        | F P   | Branch to a different block
`call`                      | F P   | Call a function
`ret`                       | F P   | Return from a function
`wait`                      | P     | Suspend execution
`halt`                      | P     | Terminate execution
**Memory**                  |       |
`var`                       | F P   | Allocate memory
`ld`                        | F P   | Load value from memory
`st`                        | F P   | Store value in memory
**Signals**                 |       |
`sig`                       | E     | Create a signal
`prb`                       | E P   | Probe value on signal
`drv`                       | E P   | Drive value of signal
**Structural**              |       |
`reg`                       | E     | Create a storage element
`del`                       | E     | Delay a signal
`con`                       | E     | Connect two signals
`inst`                      | E     | Instantiate a process/entity


### `const` - Constant Value

The `const` instruction is used to introduce a constant value into the IR. The first version constructs a constant integer value, the second a constant integer signal, and the third a constant time value.

    %a = const iN <int>
    %a = const iN$ <int>
    %a = const time <time>

- `int` is an integer literal
- `time` is a time literal

#### Examples

A constant 32 bit integer with value 42 may be constructed as follows:

    %0 = const i32 42
    %1 = const i32$ 42
    ; type(%0) = i32
    ; type(%1) = i32$

A constant time with value 1s+3d+7e may be constructed as follows:

    %0 = const time 1s 3d 7e


### `alias` - Rename Value

The `alias` instruction is used to assign a new name to a value.

    %a = alias <ty> <value>

- `ty` is the type of the resulting value `%a` and must match the type of `value`.
- `value` is the value to be aliased.

#### Example

A value `%0` may be aliased under name `foo` as follows:

    %0 = const i32 42
    %foo = alias i32 %0


### `[...]` - Construct Array

Array values may be constructed in two ways. The first constructs a uniform array where each element has the same value. The second constructs an array with different values for each element. Every element in an array must have the same type.

    %a = [<N> x <ty> <value>]
    %a = [<ty> <value1>, ..., <ty> <valueN>]

- `N` is the number of elements in the array.
- `ty` is the type of each element. All elements must have the same type.
- `value` is the value each element will have.
- `value1` to `valueN` are the values for each individual element.

#### Example

An array of 9001 zeros of 8 bits each may be constructed as follows:

    %0 = const i8 0
    %1 = [9001 x i8 %0]
    ; type(%1) = [9001 x i8]

An array with three different 16 bit values may be constructed as follows:

    %0 = const i16 9001
    %1 = const i16 42
    %2 = const i16 1337
    %3 = [i16 %0, i16 %1, i16 %2]
    ; type(%3) = [3 x i16]


### `{...}` - Construct Struct

Struct values may be constructed in the following way:

    %a = {<ty1> <value1>, ..., <tyN> <valueN>}

- `ty1` to `tyN` is the type of each field in the struct.
- `value1` to `valueN` is the value of each field in the struct.

#### Example

A struct with three fields of different types may be constructed as follows:

    %0 = const i1 0
    %1 = const i42 9001
    %2 = const time 1337s
    %3 = {i1 %0, i42 %1, time %2}
    ; type(%3) = {i1, i42, time}


### `not`, `neg` - Unary Arithmetic

The `not` operation flips each bit of a value. The `neg` operation computes the two's complement of a value, effectively flipping its sign.

    %a = not <ty> <value>
    %a = neg <ty> <value>

- `ty` must be `iN` or `iN$`.
- `value` is the input argument. Must be of type `ty`.

#### Example

The bits of an integer value may be flipped as follows:

    %0 = const i8 0x0F
    %1 = not i8 %0
    ; %1 = 0xF0

The sign of an integer may be flipped as follows:

    %0 = const i8 42
    %1 = neg i8 %0
    ; %1 = -42


###  `add`, `sub`, `and`, `or`, `xor`, `smul`, `sdiv`, `smod`, `srem`, `umul`, `udiv`, `umod`, `urem` - Binary Arithmetic

The `add` and `sub` operation add or subtract two values.

    %a = add  <ty> <lhs>, <rhs>
    %a = sub  <ty> <lhs>, <rhs>

The `and`, `or`, and `xor` operation compute the bitwise AND, OR, and XOR of two values.

    %a = and  <ty> <lhs>, <rhs>
    %a = or   <ty> <lhs>, <rhs>
    %a = xor  <ty> <lhs>, <rhs>

The multiplicative operations are available in a signed (`s...`) and unsigned (`u...`) flavor. Choosing one or the other alters how the input operands are interpreted. The `mul` operation multiplies two values. The `div` operation divides the left-hand side by the right-hand side value. The `mod` and `rem` operation compute the modulo and remainder of the division.

    %a = smul <ty> <lhs>, <rhs>
    %a = umul <ty> <lhs>, <rhs>
    %a = sdiv <ty> <lhs>, <rhs>
    %a = udiv <ty> <lhs>, <rhs>
    %a = smod <ty> <lhs>, <rhs>
    %a = umod <ty> <lhs>, <rhs>
    %a = srem <ty> <lhs>, <rhs>
    %a = urem <ty> <lhs>, <rhs>

- `ty` must be `iN` or `iN$`.
- `lhs` and `rhs` are the left- and right-hand side arguments. Must be of type `ty`.


### `eq`, `neq` - Equality Comparison of two Values

The `eq` and `neq` operation checks for equality or inequality of two values.

    %a = eq  <ty> <lhs>, <rhs>
    %a = neq <ty> <lhs>, <rhs>

- `ty` can be any type.
- `lhs` and `rhs` are the left- and right-hand side arguments of the comparison. Must be of type `ty`.
- The result of the operation is of type `i1`.


### `slt`, `sgt`, `sle`, `sge`, `ult`, `ugt`, `ule`, `uge` - Relational Comparison of two Values

The relational operators are available in a signed (`s...`) and unsigned (`u...`) flavor. Choosing one or the other alters how the input operands are interpreted. The `lt`, `gt`, `le`, and `ge` operation checks for less-than, greater-than, less-than-or-equal, and greater-than-or-equal ordering.

    %a = slt <ty> <lhs>, <rhs>
    %a = ult <ty> <lhs>, <rhs>
    %a = sgt <ty> <lhs>, <rhs>
    %a = ugt <ty> <lhs>, <rhs>
    %a = sle <ty> <lhs>, <rhs>
    %a = ule <ty> <lhs>, <rhs>
    %a = sge <ty> <lhs>, <rhs>
    %a = uge <ty> <lhs>, <rhs>

- `ty` must be `iN` or `iN$`.
- `lhs` and `rhs` are the left- and right-hand side arguments of the comparison. Must be of type `ty`.
- The result of the operation is of type `i1`.


### `shl`, `shr` - Shift a Value

The `shl` and `shr` operation shifts a value to the left or right by a given amount.

    %a = shl <ty1> <base>, <ty2> <hidden>, <ty3> <amount>
    %a = shr <ty1> <base>, <ty2> <hidden>, <ty3> <amount>

- `ty1` is the type of the base value and the type of the shift result. Can be `iN` or any array, or a signal/pointer of thereof.
- `base` is the base value that is produced if the shift amount is 0.
- `ty2` is the type of the hidden value. Must be of the same as `ty1` but may have a different number of bits or elements.
- `hidden` is the hidden value that is uncovered by non-zero shift amounts.
- `ty3` is the type of the shift amount. Must be `iN`.
- `amount` determines by how many positions the value is to be shifted. Must be in the range 0 to N, where N is the width of the `hidden` value.
- The result of the shift operation has the same type as the base value.

#### Example

The operation can be visualized as concatenating the base and the hidden value, then selecting a slice of the resulting bits starting at the offset determined by the `amount` value.

    %0 = const i8 0b10011001
    %1 = const i12 0b010110100101
    %2 = const i3 6

    %3 = shl i8 %0, i12 %1, i3 %2
    ; %3 = 0b01010110

    ; |%0----||%1--------|
    ; 10011001010110100101
    ; >>>>>>|%3----|

    %4 = shr i8 %0, i12 %1, i3 %2
    ; %4 = 0b10010110

    ; |%1--------||%0----|
    ; 01011010010110011001
    ;       |%4----|<<<<<<


### `mux` - Pick a Value from an Array

The `mux` operation chooses one of an array of values based on a given selector value.

    %a = mux <ty> <array>, <ty_sel> <sel>

- `ty` is the type of the `array`.
- `array` is a list of values from which the multiplexer selects.
- `ty_sel` is the type of the selector. Must be `iN` or `iN$`.
- `sel` is the selector and must be in the range 0 to M-1, where M is the number of elements in the array.
- The result of the operation is the element type of the array `ty`.


### `reg` - Register to Store a Value

The `reg` instruction provides a storage element for a value. It may only be used inside an entity.

    %a = reg <ty> <init>, <value> <mode> <trigger_ty> <trigger>, ...

- `ty` is the type of the stored value. Must be a signal.
- `init` is the initial value. Must be of type `ty`.
- Each comma-separated triple following the initial value specifies a storage trigger:
    - `value` is the value to be stored in the register. Must be of type `ty`.
    - `mode` is the trigger mode and may be one of the following:
        - `low` stores `value` while the trigger is low. Models active-low resets and low-transparent latches.
        - `high` stores `value` while the trigger is high. Models active-high resets and high-transparent latches.
        - `rise` stores `value` upon the rising edge of the trigger. Models rising-edge flip-flops.
        - `fall` stores `value` upon the falling edge of the trigger. Models falling-edge flip-flops.
        - `both` stores `value` upon either a rising or a falling edge of the trigger. Models dual-edge flip-flops.
    - `trigger_ty` is the trigger type. Must be `i1`.
    - `trigger` is the trigger value. Must be of type `trigger_ty`.
    - In case multiple triggers apply the left-most takes precedence.

#### Examples

A rising, falling, and dual-edge triggered flip-flop:

    %Q = reg i8$ %init, %D rise i1$ %CLK
    %Q = reg i8$ %init, %D fall i1$ %CLK
    %Q = reg i8$ %init, %D both i1$ %CLK

A rising-edge triggered flip-flop with active-low reset:

    %Q = reg i8$ %init, %init low i1$ %RSTB, %D rise i1$ %CLK

A transparent-low and transparent-high latch:

    %Q = reg i8$ %init, %D low i1$ %CLK
    %Q = reg i8$ %init, %D high i1$ %CLK

An SR latch:

    %0 = const i1$ 0
    %1 = const i1$ 1
    %Q = reg i1$ %0, %0 high i1$ %R, %1 high i1$ %S


### `br` - Branch

    br <block>
    br <value>, <block_if_0>, <block_if_1>


### `insert` — Insert Value

The `insert` instruction may be used to change the value of fields of structs, elements of arrays, or bits of integers. It comes in two variants: `insert element` operates on single elements, while `insert slice` operates on a slice of consecutive elements.

    %r = insert element <ty> <target>, <index>, <value>
    %r = insert slice <ty> <target>, <start>, <length>, <value>

- `ty` is the type of the target struct, array, or integer. A type.
- `target` is the struct, array, or integer to be modified. A value.
- `index` is the index of the field, element, or bit to be modified. An unsigned integer.
- `start` is the index of the first element or bit to be modified. An unsigned integer.
- `length` is the number of elements or bits after `start` to be modified. An unsigned integer.
- `value` is the value to be assigned o the selected field, elements, or bits. Its type must correspond to a single field, element, or bit in case of the `insert element` variant, or an array or integer of length `length` in case of the `insert slice` variant. A value.

Note that `index`, `start`, and `length` must be integer constants. You cannot pass dynamically calculated integers for these fields.

#### Result

The `insert` instruction yields the modified `target` as a result, which is of type `ty`.

#### Example

A field of a struct may be modified as follows:

    ; %0 = {i32 0, i16 0}
    %1 = insert element {i32, i16} %0, 0, 42
    ; %1 = {i32 42, i16 0}

An element of an array may be modified as follows:

    ; %0 = [i32 0, 0, 0, 0]
    %1 = insert element [4 x i32] %0, 2, 42
    ; %1 = [i32 0, 0, 42, 0]

A bit of an integer may be modified as follows:

    ; %0 = i32 3
    %1 = insert element i32 %0, 3, 1
    ; %1 = i32 11

A slice of array elements may be modified as follows:

    ; %0 = [i32 0, 0, 0, 0]
    %1 = insert slice [4 x i32] %0, 1, 2, [i32 42, 9001]
    ; %1 = [i32 0, 42, 9001, 0]

A slice of integer bits may be modified as follows:

    ; %0 = i32 8
    %1 = insert slice i32 %0, 0, 2, 3
    ; %1 = i32 11


### `extract` — Extract Value

The `extract` instruction may be used to obtain the value of fields of structs, elements of arrays, or bits of integers. It comes in two variants: `extract element` operates on single elements, while `extract slice` operates on a slice of consecutive elements.

    %r = extract element <ty> <target>, <index>
    %r = extract slice <ty> <target>, <start>, <length>

- `ty` is the type of the target struct, array, or integer. A type.
- `target` is the struct, array, or integer to be accessed. A struct may only be used in `extract element`. A value.
- `index` is the index of the field, element, or bit to be accessed. An unsigned integer.
- `start` is the index of the first element or bit to be accessed. An unsigned integer.
- `length` is the number of elements or bits after `start` to be accessed. An unsigned integer.

Note that `index`, `start`, and `length` must be integer constants. You cannot pass dynamically calculated integers for these fields.

#### Types

The basic operation of `extract` is defined on integer, struct, and array types. If the target is a signal or pointer type around a struct or array, the instruction returns a signal or pointer of the selected field or elements.

#### Result

The `extract element` instruction yields the value of the selected field, element, or bit. If the target is a struct the returned type is the `index` field of the struct. If it is an array the returned type is the array's element type. If it is an integer the returned type is the single bit variant of the integer (e.g. `i1`).

The `extract slice` instruction yields the values of the selected elements or bits. If the target is an array the returned type is the same array type but with length `length`. If the target is an integer the returned type is the same integer type but with width `length`.

#### Example

A field of a struct may be accessed as follows:

    ; %0 = {i32 42, i16 9001}
    %1 = extract element {i32, i16} %0, 0
    ; %1 = i32 42

An element of an array may be accessed as follows:

    ; %0 = [i32 0, 0, 42, 0]
    %1 = extract element [4 x i32] %0, 2
    ; %1 = i32 42

A bit of an integer may be accessed as follows:

    ; %0 = i32 11
    %1 = extract element i32 %0, 3
    ; %1 = i1 1

A slice of array elements may be accessed as follows:

    ; %0 = [i32 0, 42, 9001, 0]
    %1 = extract slice [4 x i32] %0, 1, 2
    ; %1 = [i32 42, 9001]

A slice of integer bits may be accessed as follows:

    ; %0 = i32 11
    %1 = extract slice i32 %0, 0, 2
    ; %1 = i2 3

##### Signals

The `extract` instruction may be used to dissect integer, struct, and array signals into smaller subsignals that alias the selected bits, field, or elements and may then be driven individually.

A subsignal of an integer signal may be obtained as follows:

    ; %0 = sig i32
    %1 = extract element i32$ %0, 3
    %2 = extract slice i32$ %0, 0, 2
    ; typeof(%1) = i1$
    ; typeof(%2) = i2$

A subsignal of a struct signal may be obtained as follows:

    ; %0 = sig {i32, i16}
    %1 = extract element {i32, i16}$ %0, 0
    ; typeof(%1) = i32$

A subsignal of an array signal may be obtained as follows:

    ; %0 = sig [4 x i32]
    %1 = extract element [4 x i32]$ %0, 2
    %2 = extract slice [4 x i32]$ %0, 1, 2
    ; typeof(%1) = i32$
    ; typeof(%2) = [2 x i32]$

##### Pointers

The `extract` instruction may be used to obtain a pointer to a struct field, or a pointer to one or more array fields. These pointers alias the selected field or elements and may be used in load and store operations.

A pointer to specific bits of an integer may be obtained as follows:

    ; %0 = var i32
    %1 = extract element i32* %0, 3
    %2 = extract slice i32* %0, 0, 2
    ; typeof(%1) = i1*
    ; typeof(%2) = i2*

A pointer to the field of a struct may be obtained as follows:

    ; %0 = var {i32, i16}
    %1 = extract element {i32, i16}* %0, 0
    ; typeof(%1) = i32*

A pointer to specific elements of an array may be obtained as follows:

    ; %0 = var [4 x i32]
    %1 = extract element [4 x i32]* %0, 2
    %2 = extract slice [4 x i32]$ %0, 1, 2
    ; typeof(%1) = i32*
    ; typeof(%2) = [2 x i32]*


### `shl`,`shr` — Shift a Value

The `shl` and `shr` instructions may be used to shift the bits of a number or the elements of an array to the left or right.

    %r = shl <ty> <target>, <lsb>, <amount>
    %r = shr <ty> <target>, <msb>, <amount>

- `ty` is the type of the target number or array.
- `target` is the number or array to be modified.
- `lsb` and `msb` determines the value of the bit or element that is revealed due to the shift.
- `amount` is the number of bits or elements the value is shifted. Must be an integer and is interpreted as unsigned.

The returned value is of type `ty`. The shift `amount` may exceed the bit width or number of elements of `target`, in which case the result has all bits or elements set to `lsb` or `msb`.

#### Types

The type of the value for inserted bits or elements, `lsb` and `msb`, must be the single-bit equivalent of `ty` if it is a number, or the element type of `ty` if it is an array.

#### Examples

A logical left or right shift of an integer may be performed as follows:

    ; %0 = i32 42
    %1 = shl i32 %0, i1 0, i32 3
    %2 = shr i32 %0, i1 0, i32 3
    ; %1 = i32 5
    ; %2 = i32 336

An arithmetic right shift of an integer which maintains sign extension may be performed as follows:

    ; %0 = i32 -42
    %sign = extract element i32 %0, 31
    %1 = shr i32 %0, %sign, i32 3
    ; %1 = i32 -6

The elements of an array may be shifted to the left or right as follows:

    ; %0 = [i32 1, 2, 3, 4]
    %1 = shl [4 x i32] %0, i32 9, i32 3
    %2 = shr [4 x i32] %0, i32 9, i32 3
    ; %1 = [i32 4, 9, 9, 9]
    ; %2 = [i32 9, 9, 9, 1]


## Call (*fpe*)

    call <target> (<args,...>)

The call instruction transfers control to a function declared in the module and yields the function's return value. If used in an entity, the function is reevaluated whenever its input arguments change.


## Instance (*e*)

    inst <target> (<inputs,...>) (<outputs,...>)

The instance instruction instantiates a process or entity declared in the module and connects its inputs and outputs.


## Unary Integer Arithmetic (*fpe*)

    not <ty> <arg>

These instructions perform unary arithmetic operations on a single integer value `arg`. The operand and the result of the instruction are of type `ty`, which must be an integer type `iN`. The operations performed are as follows:

- `not`: Bitwise negation of the value. Every bit that is 1 becomes a 0, and vice versa.


## Binary Integer Arithmetic (*fpe*)

    add <ty> <lhs> <rhs>
    sub <ty> <lhs> <rhs>
    mul <ty> <lhs> <rhs>
    div <ty> <lhs> <rhs>
    mod <ty> <lhs> <rhs>
    rem <ty> <lhs> <rhs>
    and <ty> <lhs> <rhs>
    or  <ty> <lhs> <rhs>
    xor <ty> <lhs> <rhs>

These instructions perform binary arithmetic operations on two integer values `lhs` and `rhs`. Both operands and the result of the instruction are of type `ty`, which must be an integer type `iN`. The operations performed are as follows:

- `add`: Addition
- `sub`: Subtraction
- `mul`: Multiplication
- `div`: Division rounding towards negative infinity.
- `mod`: Modulo. The result of the operation is `lhs + x * rhs` for the smallest `x` that makes the result positive.
- `rem`: Remainder.
- `and`: Bitwise logic AND. Each resulting bit is 1 iff both of the argument bits are 1.
- `or`: Bitwise logic OR. Each resulting bit is 1 iff either of the argument bits are 1.
- `xor`: Bitwise logic XOR. Each resulting bit is 1 iff the argument bits differ.


## Integer Comparison (*fpe*)

    cmp eq  <ty> <lhs> <rhs>
    cmp neq <ty> <lhs> <rhs>
    cmp slt <ty> <lhs> <rhs>
    cmp sgt <ty> <lhs> <rhs>
    cmp sle <ty> <lhs> <rhs>
    cmp sge <ty> <lhs> <rhs>
    cmp ult <ty> <lhs> <rhs>
    cmp ugt <ty> <lhs> <rhs>
    cmp ule <ty> <lhs> <rhs>
    cmp uge <ty> <lhs> <rhs>

These isntructions compare two integer values and yield a `i1` result. The operands are of type `ty`, which must be an integer type `iN`. The operations performed are as follows:

- `eq`: Equality
- `neq`: Inequality
- `slt`: `lhs < rhs`, both arguments treated as signed values.
- `sgt`: `lhs > rhs`, both arguments treated as signed values.
- `sle`: `lhs <= rhs`, both arguments treated as signed values.
- `sge`: `lhs >= rhs`, both arguments treated as signed values.
- `ult`: `lhs < rhs`, both arguments treated as unsigned values.
- `ugt`: `lhs > rhs`, both arguments treated as unsigned values.
- `ule`: `lhs <= rhs`, both arguments treated as unsigned values.
- `uge`: `lhs >= rhs`, both arguments treated as unsigned values.
