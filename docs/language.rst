Language Reference
==================

Types
-----

===============  ====
``void``         Nothing. E.g. if an instruction yields no result.
``metadata``     Additional information attached to various things in the assembly.
``label``        Basic block.
``time``         Simulation time.
``iN``           Integer of N bits.
``lN``           Logic type with 9 possible values ``UX01ZWLH-`` (see IEEE 1164)
``sN``           Strong logic type with 4 possible values ``01ZX`` (see IEEE 1800)
``T*``           Pointer to value of type T.
``T$``           Signal carrying a value of type T.
``[N x T]``      Array containing N elements of type T.
``{T0,T1,…}``    Struct containing fields of types T0, T1, etc.
``T (T0,T1,…)``  Function returning value T, taking arguments of type T0, T1, etc.
===============  ====


Constants
---------

=====  ====
false  Alias for ``i1 0``
true   Alias for ``i1 1``
=====  ====


Instructions
------------

========  ====  ====
Mnemonic        Description
========  ====  ====
type      M     Global type alias
const     M     Global constant
func      M     Function definition
proc      M     Process definition
entity    M     Entity definition
decl      M     Function, process, or entity declaration
**Memory**
--------------------
load      FP    Load a value from memory.
store     FP    Store a value to memory.
var       FP    Allocate memory on the stack.
alloc     FP    Allocate memory on the heap.
free      FP    Free memory on the heap.
**Signals**
--------------------
probe     PE    Sample the value of a signal.
drive     PE    Change the value of a signal
sig       E     Allocate a new signal.
**Control Flow**
--------------------
wait      P
halt      FP
br        FP
ret       F
call      FPE
inst      E
**Comparison**
--------------------
icmp      FPE   Integer comparison
lcmp      FPE   Logic comparison
**Logic**
--------------------
and       FPE
or        FPE
xor       FPE
not       FPE
**Arithmetic**
--------------------
add       FPE
sub       FPE
mul       FPE
div       FPE
**Restructuring**
--------------------
trunc     FPE   Reduce width of integer.
ext       FPE   Increase width of integer.
insert    FPE   Change the value of an array/structure element by index.
extract   FPE   Obtain the value of an array/structure element by index.
cat       FPE   Concatenate values.
slice     FPE   Obtain a part of an integer.
**Data Flow**
--------------------
mux       FPE   Select among a list of values, based on a "selection" value.
reg       E     Storage element that keeps state.
========  ====  ====


Missing
-------

* conversion lN to sN and back
* conversion iN to lN/sN and back
* working with time values
* choosing strong/weak drive on lN, choosing high impedance or weak
* a way to declare funcs, procs, and entities


Details
-------

* ``mul [signed|unsigned]``
* ``div [signed|unsigned] [rem|mod]``
* ``icmp [eq|ne|sgt|sge|slt|sle|ugt|uge|ult|ule]``
* ``lcmp [eq|ne]``
* ``ext [zero|sign]``
