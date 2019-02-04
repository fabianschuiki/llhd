# LLHD Language Reference

This document specifies the low-level hardware description language. It outlines the architecture, structure, and instruction set, and provides usage examples.

## Type System

### Overview

The following table shows the types available in LLHD. These are outlined in more detail in the following sections.

Type            | Description
--------------- | ---
`iN`            | Integer of `N` bits, signed or unsigned.
`nN`            | Enumeration of `N` distinct values.
`T*`            | Pointer to value of type `T`.
`T$`            | Signal carrying a value of type `T`.
`[N x T]`       | Array containing `N` elements of type `T`.
`{T0,T1,...}`   | Struct containing fields of types `T0`, `T1`, etc.
`T (T0,T1,...)` | Function returning value `T`, taking arguments of type `T0`, `T1`, etc.
`void`          | The type of an instruction that yields no result.
`label`         | A basic block.
`time`          | A simulation time value.


### `[N x T]` -- The Array Type

An array type may be constructed as follows:

    [N x T]

Where `N` is the number of elements in the array and `T` the type of each element. All elements have the same type.

For example:

    [16 x i8]
    [9001 x {i32, i1}]

#### Constant

An array constant value may be constructed as follows:

    (1) [T V0, V1, ...]
    (2) [V0, V1, ...]
    (3) [N x T V]
    (4) [N x V]

Variants 1 and 2 explicitly list the value of each element in the array. The length of the array is inferred from the number of elements provided. 3 and 4 create an array of length `N` and assigns each element the same value. Variants 1 and 3 explicitly list the value type, thus allowing for values with implicit types. Varaints 2 and 4 require the values to have explicit type. This allows for compact notation of integer arrays.

For example:

    [i32 42, 9001, 65]  ; type [3 x i32]
    [{i32 16, i64 9001}, {i32 42, i64 65}]  ; type [2 x {i32, i64}]
    [16 x i32 42]  ; type [16 x i32]


### `{T0,T1,...}` -- The Struct Type

A struct type may be constructed as follows:

    {T0, T1, ...}

Where each type corresponds to a field in the struct. The fields are anonymous and thus don't carry names.

For example:

    {i32, void, {i64, i1}}

#### Constant

A struct constant value may be constructed as follows:

    {V0, V1, ...}

For example:

    {i32 4, i32 9001, {i64 42}}  ; type {i32, i32, {i64}}

Note that each field must have an explicit type. This means that integer constants must be prefixed with their type: `i32 4` instead of just `4`.


## Instructions

### Overview

The following table shows the full instruction set of LLHD. Instructions have limitations as to whether they can appear in an entity, function, or process.

Instruction | Allowed In | Description
--- | --- | ---
`insert`  | EFP | Change the value of one or more fields, elements, or bits.
`extract` | EFP | Retrieve the value of one or more fields, elements, or bits.


### `insert` -- Insert Value

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


### `extract` -- Extract Value

The `extract` instruction may be used to obtain the value of fields of structs, elements of arrays, or bits of integers. It comes in two variants: `extract element` operates on single elements, while `extract slice` operates on a slice of consecutive elements.

    %r = extract element <ty> <target>, <index>
    %r = extract slice <ty> <target>, <start>, <length>

- `ty` is the type of the target struct, array, or integer. A type.
- `target` is the struct, array, or integer to be accessed. A struct may only be used in `extract element`. A value.
- `index` is the index of the field, element, or bit to be accessed. An unsigned integer.
- `start` is the index of the first element or bit to be accessed. An unsigned integer.
- `length` is the number of elements or bits after `start` to be accessed. An unsigned integer.

Note that `index`, `start`, and `length` must be integer constants. You cannot pass dynamically calculated integers for these fields.

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
    shl <ty> <lhs> <rhs>
    shr <ty> <lhs> <rhs>
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
- `shl`: Left bit shift. Upper bits are discarded, lower bits filled with 0.
- `shr`: Right bit shift. Lower bits are discarded, upper bits filled with 0.
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
