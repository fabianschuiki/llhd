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

Note that basic block names are local names introduced without an explicit leading `%` but are otherwise referred to as other local names (with the leading `%`).

Names are UTF-8 encoded. Arbitrary code points beyond letters and numbers may be represented as sequences of `\xx` bytes, where `xx` is the lower- or uppercase hexadecimal representation of the byte. E.g. the local name `foo$bar` is encoded as `%foo\24bar`.


## Units

Designs in LLHD are represented by three different constructs (called "units"): functions, processes, and entities. These capture different concerns arising from the need to model silicon hardware, and is in contrast to IRs targeting machine code generation, which generally only consist of functions.

The language differentiates between how instructions are executed in a unit:

- *Control-flow* units consist of basic blocks, where execution follows a clear control-flow path. This is equivalent to what one would find in LLVM's IR.
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

##### Example

The following function computes the Fibonacci series for a 32 bit signed integer number N:

    func @fib (i32 %N) i32 {
    entry:
        %one = const i32 1
        %0 = sle i32 %N, %one
        br %0, %recursive, %base
    base:
        ret i32 %one
    recursive:
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

##### Example

The following process computes the butterfly operation in an FFT combinatorially with a 1ns delay:

    proc @bfly (i32$ %x0, i32$ %x1) -> (i32$ %y0, i32$ %y1) {
    entry:
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

Entities represent *data-flow* executing in a *timed* fashion and consist of a set of instructions. They are used to represent hierarchy in a design, as well as a data-flow description of how a circuit's output signals change in reaction to changing input signals.

    entity <name> (<in_ty1> <in_arg1>, ...) -> (<out_ty1> <out_arg1>, ...) {
        <inst1>
        ...
        <instN>
    }

Eventually every design consists of at least one top-level entity, which may in turn call functions or instantiate processes and entities to form a design hierarchy. There are no basic blocks in an entity. All instructions are considered to execute in a schedule implicitly defined by their data dependencies. Dependency cycles are forbidden (except for the ones formed by probing and driving a signal). The order of instructions is purely cosmetic and does not affect behaviour.

##### Example

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

A basic block has a name and consists of a sequence of instructions. The `<bb_name>` name introduced is a local name that must match `[a-zA-Z0-9_\.\\]+` without an explicit leading `%`. The created basic block is referred to by the `phi`, `br` and `wait` instructions using the full `%<bb_name>` form of the label. The last instruction in a basic block must be a terminator; all other instructions must *not* be a terminator. This ensures that no control flow transfer occurs within a basic block, but rather control enters at the top and leaves at the bottom. A basic block may not be empty. Functions and processes contain at least one basic block.

    <bb_name>:
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
`{T0,T1,...}`   | Structured data containing fields of types `T0`, `T1`, etc.

Note that arbitrary combinations of signal types `T$` and pointer types `T*` are allowed. These should help support higher level HDLs advanced features and map to defined simulation behaviours. Not all such combinations are expected to describe synthesizable circuits.


### Void Type (`void`)

The `void` type is used to represent the absence of a value. Instructions that do not return a value are of type `void`. There is no way to construct a `void` value.


### Time Type (`time`)

The `time` type represents a simulation time value as a combination of a real time value in seconds, a delta value representing infinitesimal time steps, and an epsilon value representing an absolute time slot within a delta step (used to model SystemVerilog scheduling regions). It may be constructed using the `const time` instruction, for example:

    %0 = const time 1ns 2d 3e


### Integer Type (`iN`)

The `iN` type represents an integer value of `N` bits, where `N` can be any non-zero positive number. There is no sign associated with an integer values. Rather, separate instructions are available to perform signed and unsigned operations, where applicable. Integer values may be constructed using the `const iN` instruction, for example:

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
    %3 = [i16 %0, %1, %2]  ; [1, 42, 9001]
    %4 = [3 x i16 %0]      ; [1, 1, 1]

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

The following table shows the full instruction set of LLHD. The flags indicate if an instruction

- **F**: can appear in a function,
- **P**: can appear in a process,
- **E**: can appear in an entity, or
- **T**: is a terminator.

Instruction                 | Flags   | Description
----------------------------|---------|------------
**Values**                  |         |
`const`                     | F P E   | Construct a constant value
`alias`                     | F P E   | Assign a new name to a value
`[...]`                     | F P E   | Construct an array
`{...}`                     | F P E   | Construct a struct
`insf` `inss`               | F P E   | Insert elements, fields, or bits
`extf` `exts`               | F P E   | Extract elements, fields, or bits
`mux`                       | F P E   | Choose from an array of values
**Bitwise**                 |         |
`not`                       | F P E   | Unary logic
`and` `or` `xor`            | F P E   | Binary logic
`shl` `shr`                 | F P E   | Shift left or right
**Arithmetic**              |         |
`neg`                       | F P E   | Unary arithmetic
`add` `sub`                 | F P E   | Binary arithmetic
`smul` `sdiv` `smod` `srem` | F P E   | Binary signed arithmetic
`umul` `udiv` `umod` `urem` | F P E   | Binary unsigned arithmetic
**Comparison**              |         |
`eq` `neq`                  | F P E   | Equality operators
`slt` `sgt` `sle` `sge`     | F P E   | Signed relational operators
`ult` `ugt` `ule` `uge`     | F P E   | Unsigned relational operators
**Control Flow**            |         |
`phi`                       | F P     | Reconvergence node
`br`                        | F P T   | Branch to a different block
`call`                      | F P E   | Call a function
`ret`                       | F P T   | Return from a function
`wait`                      | P T     | Suspend execution
`halt`                      | P T     | Terminate execution
**Memory**                  |         |
`var`                       | F P     | Allocate memory
`ld`                        | F P     | Load value from memory
`st`                        | F P     | Store value in memory
**Signals**                 |         |
`sig`                       | E       | Create a signal
`prb`                       | E P     | Probe value on signal
`drv`                       | E P     | Drive value of signal
**Structural**              |         |
`reg`                       | E       | Create a storage element
`del`                       | E       | Delay a signal
`con`                       | E       | Connect two signals
`inst`                      | E       | Instantiate a process/entity


### Working with Values


#### Constant Value (`const`)

The `const` instruction is used to introduce a constant value into the IR. The first version constructs a constant integer value, the second a constant integer signal, and the third a constant time value.

    %result = const time <time>
    %result = const iN <int>
    %result = const nN <enum>
    %result = const lN <logic>

- `time` is a time literal such as `1s`, `1s 2d`, or `1s 2d 3e`, where the real component may carry an SI suffix such as `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`.
- `int` is an integer literal such as `0b0101`, `0o1247`, `129`, or `0x14F3E`
- `enum` is an integer literal similar to `int` but which ranges from `0` to `N-1`
- `logic` is a string of `N` logic value characters (one of `U`, `X`, `0`, `1`, `Z`, `W`, `L`, `H`, `-`)

##### Example

A constant time with value 1s+3d+7e may be constructed as follows:

    %0 = const time 1s 3d 7e

A constant 32 bit integer with value 42 may be constructed as follows:

    %0 = const i32 42
    ; type(%0) = i32

A constant integer ranging from `0` to `99` with value 13 may be constructed as follows:

    %0 = const n100 13
    ; type(%0) = n100

A constant value for 4 logic wires may be constructed as follows (note: here, the _lowest_ indexed wire is a logic `Z` and the _highest_ indexed wire is a logic `L`):

    %0 = const l4 "L0LZ"
    ; type(%0) = l4

#### Value Renaming (`alias`)

The `alias` instruction is used to assign a new name to a value.

    %result = alias T %value

- `T` is the type of the aliased `%value`.
- `%value` is the value to be aliased and must be of type `T`.
- `%result` is of type `T`.

##### Example

A value `%0` may be aliased under name `foo` as follows:

    %0 = const i32 42
    %foo = alias i32 %0


#### Array Construction (`[...]`)

Array values may be constructed in two ways. The first constructs a uniform array where each element has the same value. The second constructs an array with different values for each element. Every element in an array must have the same type.

    %result = [N x T %value]
    %result = [T %value1, ..., %valueN]

- `N` is the number of elements in the array.
- `T` is the type of each element. All elements must have the same type.
- `%value` is the value each element will have, and is of type `T`.
- `%value1` to `%valueN` are the values for each individual element, each of type `T`.
- `%result` is of type `[N x T]`

##### Example

An array of 9001 zeros of 8 bits each may be constructed as follows:

    %0 = const i8 0
    %1 = [9001 x i8 %0]
    ; type(%1) = [9001 x i8]

An array with three different 16 bit values may be constructed as follows:

    %0 = const i16 9001
    %1 = const i16 42
    %2 = const i16 1337
    %3 = [i16 %0, %1, %2]
    ; type(%3) = [3 x i16]


#### Struct Construction (`{...}`)

Struct values may be constructed in the following way:

    %result = {T1 %value1, ..., TN %valueN}

- `T1` to `TN` are the types of each field in the struct.
- `%value1` to `%valueN` are the values for each field in the struct.
- `%result` is of type `{T1, ..., TN}`.

##### Example

A struct with three fields of different types may be constructed as follows:

    %0 = const i1 0
    %1 = const i42 9001
    %2 = const time 1337s
    %3 = {i1 %0, i42 %1, time %2}
    ; type(%3) = {i1, i42, time}


#### Inserting Elements, Fields, or Bits (`insf` `inss`)

> TODO(fschuiki): Update to current names/semantics.

The `insert` instruction may be used to change the value of fields of structs, elements of arrays, or bits of integers. It comes in two variants: `insert element` operates on single elements, while `insert slice` operates on a slice of consecutive elements.

    %r = insf Tt %target, Tv %value, Nindex
    %r = inss Tt %target, Tv %value, Nstart, Nlength

- `ty` is the type of the target struct, array, or integer. A type.
- `target` is the struct, array, or integer to be modified. A value.
- `index` is the index of the field, element, or bit to be modified. An unsigned integer.
- `start` is the index of the first element or bit to be modified. An unsigned integer.
- `length` is the number of elements or bits after `start` to be modified. An unsigned integer.
- `value` is the value to be assigned to the selected field, elements, or bits. Its type must correspond to a single field, element, or bit in case of the `insert element` variant, or an array or integer of length `length` in case of the `insert slice` variant. A value.

Note that `index`, `start`, and `length` must be integer constants. You cannot pass dynamically calculated integers for these fields.

##### Result

The `insert` instruction yields the modified `target` as a result, which is of type `ty`.

##### Example

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


#### Extracting Elements, Fields, or Bits (`extf` `exts`)

> TODO(fschuiki): Update to current names/semantics.

The `extract` instruction may be used to obtain the value of fields of structs, elements of arrays, or bits of integers. It comes in two variants: `extract element` operates on single elements, while `extract slice` operates on a slice of consecutive elements.

    %r = extf Tv, Tt %target, Nindex
    %r = exts Tv, Tt %target, Nstart, Nlength

- `ty` is the type of the target struct, array, or integer. A type.
- `target` is the struct, array, or integer to be accessed. A struct may only be used in `extract element`. A value.
- `index` is the index of the field, element, or bit to be accessed. An unsigned integer.
- `start` is the index of the first element or bit to be accessed. An unsigned integer.
- `length` is the number of elements or bits after `start` to be accessed. An unsigned integer.

Note that `index`, `start`, and `length` must be integer constants. You cannot pass dynamically calculated integers for these fields.

##### Types

The basic operation of `extract` is defined on integer, struct, and array types. If the target is a signal or pointer type around a struct or array, the instruction returns a signal or pointer of the selected field or elements.

##### Result

The `extract element` instruction yields the value of the selected field, element, or bit. If the target is a struct the returned type is the `index` field of the struct. If it is an array the returned type is the array's element type. If it is an integer the returned type is the single bit variant of the integer (e.g. `i1`).

The `extract slice` instruction yields the values of the selected elements or bits. If the target is an array the returned type is the same array type but with length `length`. If the target is an integer the returned type is the same integer type but with width `length`.

##### Example

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

###### Signals

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

###### Pointers

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


#### Value Multiplexing (`mux`)

    %result = mux Ta %array, Ts %sel

The `mux` operation chooses one of an array of values based on a given selector value.

- `Ta` is the type of the `%array`.
- `%array` is a list of values from which the multiplexer selects.
- `Ts` is the type of the selector. Must be `iN`.
- `%sel` is the selector and must be in the range 0 to M-1, where M is the number of elements in `%array`.
- The result of the operation is the element type of `Ta`.


### Bitwise Operators


#### Unary Logic (`not`)

    %result = not T %value

The `not` operation flips each bit of a value.

- `T` must be `iN` or `lN`.
- `%value` is the input argument of type `T`.
- `%result` is of type `T`.

##### Example

    %0 = const i1 0
    %1 = not i1 %0  ; %1 = 1

##### Truth Table for `iN`

| `not` | **0** | **1** |
| ----- | ----- | ----- |
|       | 1     | 0     |

##### Truth Table for `lN`

> TODO


#### Binary Logic (`and` `or` `xor`)

    %result = and T %lhs, %rhs
    %result = or  T %lhs, %rhs
    %result = xor T %lhs, %rhs

The `and`, `or`, and `xor` instructions compute the bitwise AND, OR, and XOR of two values, respectively.

- `T` must be `iN` or `lN`.
- `%lhs` and `%rhs` are the input arguments of type `T`.
- `%result` is of type `T`.

##### Example

    %0 = const i4 0b0011
    %1 = const i4 0b0101
    %2 = and i4 %0, %1  ; %2 = 0b0001
    %3 = or  i4 %0, %1  ; %3 = 0b0111
    %4 = xor i4 %0, %1  ; %4 = 0b0110

##### Truth Table for `iN`

`and` | **0** | **1**
----- | ----- | -----
**0** | 0     | 0
**1** | 0     | 1

`or ` | **0** | **1**
----- | ----- | -----
**0** | 0     | 1
**1** | 1     | 1

`xor` | **0** | **1**
----- | ----- | -----
**0** | 0     | 1
**1** | 1     | 0

##### Truth Table for `lN`

> TODO


#### Shift Left/Right (`shl` `shr`)

    %result = shl T %base, Th %hidden, Ta %amount
    %result = shr T %base, Th %hidden, Ta %amount

The `shl` and `shr` instruction shifts a value to the left or right by a given amount. The instruction is transparent to signals and pointers. For example, passing a signal as argument will shift the underlying value and return a signal to the shifted value.

- `T` must be `iN`, `lN`, or an array; or a signal/pointer thereof.
- `Th` must be of the same type as `T`, but may have a different number of bits or elements.
- The *maximum shift amount* is determined by the number of bits or elements in `Th`.
- `Ta` must be `iN`.
- `%base` is the base value that is produced if the shift amount is 0, and must be of type `T`.
- `%hidden` is the hidden value that is uncovered by non-zero shift amounts, and must be of type `Th`.
- `%amount` is the unsigned shift amount and determines by how many positions the value is to be shifted. Must be of type `Ta`. Behavior for values `%amount > N` is undefined.
- `%result` is of type `T`.

##### Example

The operation can be visualized as concatenating the `%base` and `%hidden` values, then selecting a slice of the resulting bits or elements starting at the offset determined by the `%amount` value. For example:

    %base = const i8 0b10011001         ; base
    %hidden = const i12 0b010110100101  ; hidden
    %amount = const i3 6                ; amount

Left shift:

    %L = shl i8 %base, i12 %hidden, i3 %amount  ; %L = 0b01010110

    ; |-base-||--hidden--|
    ; 10011001010110100101
    ; >>>>>>|--%L--|

Right shift:

    %R = shr i8 %base, i12 %hidden, i3 %amount  ; %R = 0b10010110

    ; |--hidden--||-base-|
    ; 01011010010110011001
    ;       |--%R--|<<<<<<


### Arithmetic Operators


#### Unary Arithmetic (`neg`)

    %result = neg T %value

The `neg` operation computes the two's complement of a value, effectively flipping its sign.

- `T` must be `iN`.
- `%value` is the input argument of type `T`.
- `%result` is of type `T`.

##### Example

    %0 = const i8 42
    %1 = neg i8 %0  ; %1 = -42


#### Binary Arithmetic (`add` `sub` `mul` `udiv` `sdiv` `umod` `smod` `srem`)

    %result = add  T %lhs, %rhs
    %result = sub  T %lhs, %rhs
    %result = mul T %lhs, %rhs

    %result = udiv T %lhs, %rhs
    %result = sdiv T %lhs, %rhs

    %result = umod T %lhs, %rhs
    %result = smod T %lhs, %rhs
    %result = srem T %lhs, %rhs

The `add` and `mul` instructions respectively add and multiply two values. The `sub` instruction subtract the `%rhs` from the `%lhs`.
The `udiv` and `sdiv` instructions divide the `%lhs` by the `%rhs`, interpreting the values as either unsigned or signed values, respectively.
The `umod`, `smod`, and `srem` instructions compute the modulus and remainder of dividing the `%lhs` by `%rhs`. The semantics for signed numbers are as follows:

    x = (x smod y) + round_towards_ninf(x / y) * y
    x = (x srem y) + round_towards_zero(x / y) * y

`%lhs` | `%rhs` | `smod` | `srem`
------ | ------ | ------ | ------
9      | 5      | 4      | 4
9      | -5     | -1     | 4
-9     | 5      | 1      | -4
-9     | -5     | -4     | -4

- `T` must be `iN`.
- `%lhs` and `%rhs` must be of type `T`.
- `%result` is of type `T`.


### Comparison Operators


#### Equality Comparison (`eq` `neq`)

    %result = eq  T %lhs, %rhs
    %result = neq T %lhs, %rhs

The `eq` and `neq` instructions check for equality or inequality of two values.

- `T` can be any type.
- `%lhs` and `%rhs` are the left- and right-hand side arguments of the comparison and must be of type `T`.
- `%result` is of type `i1`.


#### Relational Comparison (`slt` `sgt` `sle` `sge` `ult` `ugt` `ule` `uge`)

    %result = slt T %lhs, %rhs
    %result = sgt T %lhs, %rhs
    %result = sle T %lhs, %rhs
    %result = sge T %lhs, %rhs

    %result = ult T %lhs, %rhs
    %result = ugt T %lhs, %rhs
    %result = ule T %lhs, %rhs
    %result = uge T %lhs, %rhs

The relational operators are available in a signed (`s` prefix) and unsigned (`u` prefix) flavor. Input operands are interpreted according to this prefix. The operations performed are as follows:

| Signed | Unsigned | Operation |
|--------|----------|-----------|
| `slt`  | `ult`    | `<`       |
| `sgt`  | `ugt`    | `>`       |
| `sle`  | `ule`    | `<=`      |
| `sge`  | `uge`    | `>=`      |

- `T` must be `iN`.
- `%lhs` and `%rhs` are the left- and right-hand side arguments of the comparison and must be of type `T`.
- `%result` is of type `i1`.


### Control Flow


#### Phi Node (`phi`)

    %result = phi T [%v1, %bb1], ..., [%vN, %bbN]

The `phi` instruction is used to implement the φ node in the SSA graph representing the function or process. It produces one of its arguments `%v1` to `%vN` as a result depending on which basic block control flow originated from upon entering the `phi` instruction's basic block.

- `T` can be any type.
- `%v1` to `%vN` must be of type `T`.
- `%bb1` to `%bbN` must be basic block labels.
- `%result` is of type `T`.
- The instruction must provide a value for every predecessor of its containing basic block.


#### Branch (`br`)

    br %target                            ; unconditional
    br %cond, %target_if_0, %target_if_1  ; conditional

The `br` instruction transfers control flow to another basic block. In the unconditional case, control flow jumps to the `%target`. In the conditional case, `%cond` determines if control is transferred to `%target_if_0` (on 0) or `%target_if_1` (on 1).

- `%cond` must be of type `i1`.
- `%target`, `%target_if_0`, and `%target_if_1` must be basic block labels.
- This is a terminator instruction.


#### Call (`call`)

    %result = call Tr <name> (T1 %arg1, ..., TN %argN)

The `call` instruction transfers control to a function and yields its return value. If used in an entity, the function is re-evaluated whenever any of the input arguments change.

- `Tr` is the return type.
- `T1` to `TN` are the argument types.
- `%arg1` to `%argN` are the function arguments and must be of types `T1` to `TN`, respectively.
- `<name>` must be a local or global name of a function with signature `(T1, ..., TN) Tr`.
- `%result` is of type `Tr`. May be omitted if the function returns `void`.


#### Return from a Function (`ret`)

    ret           ; return void
    ret T %value  ; return a value

The `ret` instruction returns from a function by transferring control flow to the caller. A function with `void` return type must contain `ret` instructions without arguments.

- `T` is the return type and must match the enclosing function's return type.
- `%value` is the return value and must be of type `T`.
- This is a terminator instruction.


#### Suspend Process Execution (`wait`)

    wait %resume_bb, %obs1, ..., %obsN
    wait %resume_bb for %time, %obs, ..., %obsN

The `wait` instruction suspends execution of a process until any of the observed signals `%obs1` to `%obsN` change or optionally a fixed time interval `%time` has passed. Execution resumes at the basic block `%resume_bb`.

- `%resume_bb` must be a basic block label.
- `%obs1` to `%obsN` must be of signal type `T$`.
- `%time` must be of type `time`.
- This is a terminator instruction.


#### Terminate Process Execution (`halt`)

    halt

The `halt` instruction terminates execution of a process. All processes must eventually halt or consist of an infinite loop.

- This is a terminator instruction.

##### Example

This instruction can be used to model HDL processes that will eventually finish executing. For example the SystemVerilog

    initial begin : p0
        // ...
    end

or VHDL

    p0: process
    begin
        -- ...
        wait;
    end process;

would eventually translate to the following in LLHD:

    proc %p0 () -> () {
    entry:
        ; ...
        halt
    }


### Memory


#### Stack Allocation (`var`)

    %result = var T %init

The `var` instruction allocates memory on the stack with the initial value `%init` and returns a pointer to that location.

- `T` may be any type.
- `%init` is the initial value of the memory location and must be of type `T`.
- `%result` is of type `T*`.


#### Loading from Memory (`ld`)

    %result = ld T* %ptr

The `ld` instruction loads a value from the memory location `%ptr`.

- `T` may be any type.
- `%ptr` must be of type `T*`.
- `%result` is of type `T`.


#### Storing to Memory (`st`)

    st T* %ptr, %value

The `st` instruction stores a `%value` to the memory location `%ptr`.

- `T` may be any type.
- `%ptr` must be of type `T*`.
- `%value` must be of type `T`.


### Signals


#### Creating a Signal (`sig`)

    %result = sig T %init

The `sig` instruction creates a signal in an entity with the initial value `%init` and returns that signal.

- `T` may be any type.
- `%init` is the initial value of the signal and must be of type `T`.
- `%result` is of type `T$`.


#### Probing the Value on a Signal (`prb`)

    %result = prb T$ %signal

The `prb` instruction probes the current value of a signal `%signal`.

- `T` may be any type.
- `%signal` must be of type `T$`.
- `%result` is of type `T`.


#### Driving a Value onto a Signal (`drv`)

    drv T$ %signal, %value after %delay
    drv T$ %signal, %value after %delay if %cond

The `drv` instruction schedules signal `%signal` to change to a new value `%value` after the delay `%delay` has passed. In presence of the optional gating condition `%cond`, the instruction acts as a no-op if `%cond` is 0.

- `T` may be any type.
- `%signal` must be of type `T$`.
- `%value` must be of type `T`.
- `%delay` must be of type `time`.
- `%cond` must be of type `i1`.


### Structure and Hierarchy


#### Storage Element (`reg`)

    reg T$ %signal, [%value, <mode> %trigger], ...
    reg T$ %signal, [%value, <mode> %trigger if %gate], ...

The `reg` instruction provides a storage element which drives its output onto `%signal`. The storage element transitions to a new `%value` when the corresponding trigger (given by a value and mode) fires, and optionally if a gating condition is true. It may only be used inside an entity.

- `T` is the type of the stored value.
- `%signal` is the signal that carries the stored value. Must be of type `T$`.
- Each comma-separated triple following the initial value specifies a storage trigger:
    - `%value` is the value to be stored in the register. Must be of type `T` or `T$`.
    - `<mode>` is the trigger mode and may be one of the following:
        - `low` stores `%value` while the trigger is low. Models active-low resets and low-transparent latches.
        - `high` stores `%value` while the trigger is high. Models active-high resets and high-transparent latches.
        - `rise` stores `%value` upon the rising edge of the trigger. Models rising-edge flip-flops.
        - `fall` stores `%value` upon the falling edge of the trigger. Models falling-edge flip-flops.
        - `both` stores `%value` upon either a rising or a falling edge of the trigger. Models dual-edge flip-flops.
    - `%trigger` is the trigger value and must be of type `i1`.
    - `%gate` is the gate value and must be of type `i1`.
    - In case multiple triggers apply the left-most takes precedence.

##### Example

A rising, falling, and dual-edge triggered flip-flop:

    reg i8$ %Q, [%D, rise %CLK]
    reg i8$ %Q, [%D, fall %CLK]
    reg i8$ %Q, [%D, both %CLK]

A rising-edge triggered flip-flop with active-low reset:

    reg i8$ %Q, [%init, low %RSTB], [%D, rise %CLK]

A rising-edge triggered enable flip-flop with active-low reset:

    reg i8$ %Q, [%init, low %RSTB], [%D, rise %CLK if %EN]

A transparent-low and transparent-high latch:

    reg i8$ %Q, [%D, low %CLK]
    reg i8$ %Q, [%D, high %CLK]

An SR latch:

    %0 = const i1 0
    %1 = const i1 1
    reg i1$ %Q, [%0, high %R], [%1, high %S]


#### Wire Delay (`del`)

    del T$ %target, %source, %delay

The `del` instruction delays a signal `%source` by a `%delay`, driving the delayed value on `%target`. It models a transport delay, meaning that all strictly monotonically increasing events on `%source` will eventually be reproduced on `%target`.

- `T` is the type carried by the signal.
- `%target` and `%source` must be of type `T$`.
- `%delay` must be of type `time`.


#### Short (`con`)

    con T$ %sigA, %sigB

The `con` instruction connects two signals such that they essentially become one signal. All driven values on one signal will be reflected on the other.

- `T` can be any type.
- `%sigA` and `%sigB` must be of type `T$`.


#### Instantiate Process/Entity (`inst`)

    inst <target> (Ti1 %in1, ..., TiN %inN) (To1 %out1, ..., ToN %outN)

The `inst` instruction instantiates a process or entity within the current entity. The target's input and output signals are connected to `%in1, ...` and `%out1, ...`, respectively. This instruction builds design hierarchies.

- `Ti1` to `TiN` are the input argument types.
- `To1` to `ToN` are the output argument types.
- `%in1` to `%inN` are the input arguments and must be of types `Ti1` to `TiN`, respectively.
- `%out1` to `%outN` are the output arguments and must be of types `To1` to `ToN`, respectively.
- `<target>` must be a local or global name referring to a process or entity with signature `(Ti1, ..., TiN) -> (To1, ..., ToN)`.
