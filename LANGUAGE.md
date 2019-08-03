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


### `[N x T]` — The Array Type

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


### `{T0,T1,...}` — The Struct Type

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
`const` | EFP | Construct a constant value
`alias` | EFP | Assign a new name to a value
`[...]` | EFP | Construct an array
`{...}` | EFP | Construct a struct
`not`   | EFP | Bitwise NOT
`neg`   | EFP | Two's complement
`add`   | EFP | Addition
`sub`   | EFP | Subtraction
`and`   | EFP | Bitwise AND
`or`    | EFP | Bitwise OR
`xor`   | EFP | Bitwise XOR
`smul`  | EFP | Signed multiplication
`umul`  | EFP | Unsigned multiplication
`sdiv`  | EFP | Signed division
`udiv`  | EFP | Unsigned division
`smod`  | EFP | Signed modulo
`umod`  | EFP | Unsigned modulo
`srem`  | EFP | Signed remainder
`urem`  | EFP | Unsigned remainder
`eq`    | EFP | Check for equality
`neq`   | EFP | Check for inequality
`slt`   | EFP | Check for signed less-than ordering
`ult`   | EFP | Check for unsigned less-than ordering
`sgt`   | EFP | Check for signed greater-than ordering
`ugt`   | EFP | Check for unsigned greater-than ordering
`sle`   | EFP | Check for signed less-than-or-equal ordering
`ule`   | EFP | Check for unsigned less-than-or-equal ordering
`sge`   | EFP | Check for signed greater-than-or-equal ordering
`uge`   | EFP | Check for unsigned greater-than-or-equal ordering
`shl`   | EFP | Shift a value to the left
`shr`   | EFP | Shift a value to the right
`mux`   | EFP | Choose from an array of values
`reg`   | E   | A register to provide storage for a value
`insert`  | EFP | Change the value of one or more fields, elements, or bits.
`extract` | EFP | Retrieve the value of one or more fields, elements, or bits.
`shl`, `shr` | EFP | Shift a value to the left or right.


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
