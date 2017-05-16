# LLHD Language Reference

This document outlines the architecture and instructions of the Low Level Hardware Description language.

# Instructions

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
