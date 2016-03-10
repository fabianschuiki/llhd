LLHD C Interface
================

## Use cases
- Link two modules together
- Constant folding
- Desequentialize
- Structuralize


## Type
- make, copy, destroy
- dump, print
- equality
- get kind, check kind
- struct: get fields, get num fields
- array: get type, get length
- ptr: get type

## Value
- copy, destroy
- dump, print
- equality
- get name, get type

## Unit
- get parent, get next, get prev
- append bb, get first bb, get last bb, get num bb

## Function
- make

## Process
- make

## Entity
- make

## Basic Block
- make
- get parent, get next, get prev
- remove from parent, insert after, insert before
- append inst
- get first inst, get last inst, get num insts

## Instruction
- make
- get parent, get next, get prev
- remove from parent, insert after, insert before
- get first use, get last use, get num uses
