Instructions
============

Control Flow
------------

Wait Instruction
~~~~~~~~~~~~~~~~

::

    wait <resume> (<signal>, ...) [until <timeout>]

The wait instruction has return type ``void``. The resume destination ``resume`` is of type ``label``. The ``timeout`` is of type ``time``. This instruction suspends the execution of the current process until activity occurs on one of the ``signals``, or the absolute time ``timeout`` has been reached. Execution then resumes at ``resume``.

.. code-block:: llhd

    Wait:
        %A = sig i3
        wait %Resume (%A)
    Resume:


Branch Instruction
~~~~~~~~~~~~~~~~~~

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
~~~~~~~~~~~~~~~~

::

    <result> = call <ty> <func> (<argty> <arg>, ...)

The call instruction represents a simple function call.

#. ``result``: The value returned by the function. Omitted if it returns ``void``.
#. ``ty``: The type of the call instruction itself, and also the type of the return value.
#. ``func``: The function to be called. Must be of type ``ty (argty, ...)``.
#. ``argty``: The type of the first argument.
#. ``arg``: The first argument.

The call instruction is used to transfer control flow to the specified function. The function's arguments are bound to the values provided in the call. A return instruction in the function causes control flow to resume after the call. The call yields the function's return value.

.. code-block:: llhd

    decl func i16 @MyFunc (i32, i8, n2)
    %return_value = call void @MyFunc (i32 42, i8 128, n2 0)


Arithmetic
----------

::

    <result> = add|sub|mul <ty> <op1> <op2>
    <result> = div [rem|mod] signed|unsigned <ty> <op1> <op2>

The arithmetic instructions perform addition, subtraction, multiplication, unsigned, and signed division of two numbers.

The two operands `op1` and `op2` must both be of type `ty`, which is also the return type of the instruction. `ty` can be an integer, enumerated, or time type. If the full precision of multiplication is required, extend the operands to the desired width beforehand. Division operates differently for unsigned and signed operands, whereas addition, subtraction, and multiplication are sign-agnostic. The division instruction can be configured to yield the remainder (``rem``) or modulus (``mod``) of the division instead. The two differ for signed operands as follows:

==  ==  =======  =======
A   B   A rem B  A mod B
==  ==  =======  =======
5   3   2        2
-5  3   -2       1
-5  -3  -2       -2
5   -3  2        -1
==  ==  =======  =======

.. code-block:: llhd

    %0 = add i8 1 2
    %1 = sub i8 10 3
    %2 = mul i8 5 5
    %3 = div unsigned i8 5 3
    %4 = div rem unsigned i8 5 3
    %5 = div mod unsigned i8 5 3
    %6 = div signed i8 -5 -3
    %7 = div rem signed i8 -5 -3
    %8 = div mod signed i8 -5 -3
