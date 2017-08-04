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
``nN``           Enumerated type of N values.
``lN``           Logic type with 9 possible values ``UX01ZWLH-`` (see IEEE 1164)
``sN``           Strong logic type with 4 possible values ``01ZX`` (see IEEE 1800)
``T*``           Pointer to value of type T.
``T$``           Signal carrying a value of type T.
``[N x T]``      Array containing N elements of type T.
``{T0,T1,…}``    Struct containing fields of types T0, T1, etc.
``T (T0,T1,…)``  Function returning value T, taking arguments of type T0, T1, etc.
===============  ====


Names
-----

===========  ====
**Global Visibility**
-----------------
``@<name>``  Global name.  Visible in the symbol table of the module.
``!<name>``  Metadata name. Visible in the symbol table of the module.
**Local Visibility**
-----------------
``%<name>``  Local name. Not visible beyond the scope of the module or section it is declared in.
``!<int>``   Temporary local metadata name.
``%<int>``   Temporary local name.
===========  ====


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


Instructions
------------

Control Flow
~~~~~~~~~~~~

Wait Instruction
^^^^^^^^^^^^^^^^

::

    wait <resume> (<signal>, ...) [until <timeout>]

The wait instruction has return type ``void``. The resume destination ``resume`` is of type ``label``. The ``timeout`` is of type ``time``. This instruction suspends the execution of the current process until activity occurs on one of the ``signals``, or the absolute time ``timeout`` has been reached. Execution then resumes at ``resume``.

.. code-block:: llhd

    Wait:
        %A = sig i3
        wait %Resume (%A)
    Resume:


Branch Instruction
^^^^^^^^^^^^^^^^^^

::

    br <cond> label <iftrue> <iffalse>  ; conditional form
    br label <target>                   ; unconditional form

The branch instruction has return type ``void``. The condition ``cond`` is of type ``i1``, the branch destinations ``iftrue``, ``iffalse``, and ``target`` are of type ``label``.

.. code-block:: llhd

    Test:
        %cmp = cmp eq i32 %a %b
        br %cmp label %IfEqual %IfUnequal
    IfEqual:
        ret i32 1
    IfUnequal:
        ret i32 0


Call Instruction
^^^^^^^^^^^^^^^^

::

    <retval> = call <retty> <func> (<argty> <arg>, ...)

The **call** instruction represents a simple function call.

#. ``retval``: The value returned by the function. Omitted if it returns ``void``.
#. ``retty``: The type of the call instruction itself, and also the type of the return value.
#. ``func``: The function to be called. Must be of type ``retty (argty, ...)``.
#. ``argty``: The type of the first argument.
#. ``arg``: The first argument.

The call instruction is used to transfer control flow to the specified function. The function's arguments are bound to the values provided in the call. A return instruction in the function causes control flow to resume after the call. The call yields the function's return value.

.. code-block:: llhd

    decl func i16 @MyFunc (i32, i8, n2)
    %return_value = call void @MyFunc (i32 42, i8 128, n2 0)
