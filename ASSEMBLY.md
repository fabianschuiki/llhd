# LLHD Assembly

## Instructions
At the most basic level, LLHD assembly consists of instructions that are executed in the order in which they appear. They may have none, one, or multiple return values that are stored in local variables.

    br %a
    %0 = add %a %b
    %1, %2 = call %func (%a, %b)

This makes it easy to identify dependencies since all written variables appear on the left of the "=" sign, and all read variables appear on the right.

All instructions start with the instruction name followed by the non-optional arguments. All subsequent optional arguments are prefixed with a "," to facilitate parsing.


## Modules
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


## Functions
Functions act in a similar way to modules, but act on variables rather than on signals. This allows common sequences of instructions to be factored into separate blocks.

    func %bar (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %x = and %a %b
        %y = or %a %b
    }

Functions must be called with the `call` instruction for them to be evaluated.

    %u, %v = call %bar (%i, %j)


## Variables
Variables can only be assigned once, thus making the entire LLHD assembly of single-assignment form. This makes the code easier to reason about.

### Signals
Signals are represented as a special kind of variable that allows the `drv` instruction to act upon it. Signals have to be declared before they can be driven:

    %a = sig l1
    drv %a %0

### Memory
Memory is represented as a special kind of variable that allows reading through the `ld` and writing through the `st` instruction. This allows for loops and stateful variables even though the assembly at its core is in single-assignment form. Memory has to be allocated before it can be used:

    %a = alloc l1
    st %a %0
    %0 = ld %a


## Features
Instructions, modules, functions, and variables carry a set of feature flags that indicate what parts of the assembly language are used. This includes flags that indicate the use of branches, loops, memory, signals, function calls, implicit storage, and the like. A module's feature flags equals the feature flags of all its instructions ORed together. It should be possible to transform assembly into a representation that has certain feature flags cleared. This shifts part of the burden of making sense of a design assembly to the LLHD library.

For example, a combinatorial process in a piece VHDL code might be translated into an LLHD assembly module that contains a loop in order for the code to be as compact as possible. A synthesizer might not want to cope with this kind of optimization and thus requests a transformed version of the assembly that has no loops. This would transform the design into a representation without any loops, potentially at the expense of a lot of code.

Another example concerns implicit storage elements. When a module reacts to a change in its input values and does not issue a `drv` instruction in response, it effectively retains the previously driven state. This is an implicit storage device, since only certain combinations of input signals cause a change of the output signal. A synthesizer might not want to make the effort to analyze modules for their implicit storage devices in order to determine the registers a design requires. It thus requests a version of the assembly that contains explicit storage elements, where `drv` instructions are replaced with `reg` instructions that are always executed, thus making it easier to analyze the kind of storage element in use.
