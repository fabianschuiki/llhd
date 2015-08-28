# LLHD Assembly

## Structure

### Instructions
At the most basic level, LLHD assembly consists of instructions that are executed in the order in which they appear. They may have none, one, or multiple return values that are stored in local variables.

    br %a
    %0 = add %a %b
    %1, %2 = call %func (%a, %b)

This makes it easy to identify dependencies since all written variables appear on the left of the "=" sign, and all read variables appear on the right.

All instructions start with the instruction name followed by the non-optional arguments. All subsequent optional arguments are prefixed with a "," to facilitate parsing.


### Modules
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


### Functions
Functions act in a similar way to modules, but act on variables rather than on signals. This allows common sequences of instructions to be factored into separate blocks.

    func %bar (l1 %a, l1 %b) (l1 %x, l1 %y) {
        %x = and %a %b
        %y = or %a %b
    }

Functions must be called with the `call` instruction for them to be evaluated.

    %u, %v = call %bar (%i, %j)


### Variables
Variables can only be assigned once, thus making the entire LLHD assembly of single-assignment form. This makes the code easier to reason about.

#### Signals
Signals are represented as a special kind of variable that allows the `drv` instruction to act upon it. Signals have to be declared before they can be driven:

    %a = sig l1
    drv %a %0

#### Memory
Memory is represented as a special kind of variable that allows reading through the `ld` and writing through the `st` instruction. This allows for loops and stateful variables even though the assembly at its core is in single-assignment form. Memory has to be allocated before it can be used:

    %a = alloc l1
    st %a %0
    %0 = ld %a


### Features
Instructions, modules, functions, and variables carry a set of feature flags that indicate what parts of the assembly language are used. This includes flags that indicate the use of branches, loops, memory, signals, function calls, implicit storage, and the like. A module's feature flags equals the feature flags of all its instructions ORed together. It should be possible to transform assembly into a representation that has certain feature flags cleared. This shifts part of the burden of making sense of a design assembly to the LLHD library.

For example, a combinatorial process in a piece VHDL code might be translated into an LLHD assembly module that contains a loop in order for the code to be as compact as possible. A synthesizer might not want to cope with this kind of optimization and thus requests a transformed version of the assembly that has no loops. This would transform the design into a representation without any loops, potentially at the expense of a lot of code.

Another example concerns implicit storage elements. When a module reacts to a change in its input values and does not issue a `drv` instruction in response, it effectively retains the previously driven state. This is an implicit storage device, since only certain combinations of input signals cause a change of the output signal. A synthesizer might not want to make the effort to analyze modules for their implicit storage devices in order to determine the registers a design requires. It thus requests a version of the assembly that contains explicit storage elements, where `drv` instructions are replaced with `reg` instructions that are always executed, thus making it easier to analyze the kind of storage element in use.



## HDL Examples
This section outlines the mapping of common HDL structures to LLHD.

### Structural SR Latch
**VHDL:**

    entity nor_gate is
        port (a,b: in bit;
              z: out bit);
    end nor_gate;

    architecture structural of nor_gate is
    begin
        z <= a nor b;
    end structural;

    entity latch is
        port (s,r: in bit;
              q,nq: out bit);
    end latch;

    architecture structural of latch is
        component nor_gate
            port (a,b: in bit;
                  z: out bit);
        end component;
    begin
        n1: nor_gate port map (r,nq,q);
        n2: nor_gate port map (s,q,nq);
    end structural;

**LLHD:**

    module %nor_gate (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        %n1 = inst %nor_gate (%r, %nq) (%q)
        %n2 = inst %nor_gate (%s, %q) (%nq)
    }


### Data Flow SR Latch
**VHDL:**

    entity latch is
        port (s,r: in bit,
              q,nq: out bit);
    end latch;

    architecture dataflow of latch is
    begin
        q <= r nor nq;
        nq <= s nor q;
    end dataflow;

**LLHD:**

    module %latch_p0 (u1 %r, u1 %nq) (u1 %q) {
        %0 = nor %r %nq
        drv %q %0
    }

    module %latch_p1 (u1 %s, u1 %q) (u1 %nq) {
        %0 = nor %s %q
        drv %nq %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        inst %latch_p0 (%r, %nq) (%q)
        inst %latch_p1 (%s, %q) (%nq)
    }

An optimization step can be performed that collapses structurally identical modules into one:

    module %latch_p0 (u1 %r, u1 %nq) (u1 %q) {
        %0 = nor %r %nq
        drv %q %0
    }

    module @latch (u1 %s, u1 %r) (u1 %q, u1 %nq) {
        inst %latch_p0 (%r, %nq) (%q)
        inst %latch_p0 (%s, %q) (%nq)
    }


### Inertial Delay vs. Transport Delay
**VHDL:**

    entity nor_gate is
        port (a,b: in bit,
              z: out bit);
    end nor_gate;

    architecture idm of nor_gate is
    begin
        z <= a nor b after 1ns;
    end idm;

    architecture tdm of nor_gate is
    begin
        z <= transport a nor b after 1ns;
    end tdm;

**LLHD:** The distinction between intertial and transport delay modle shall be made in the `drv` instruction. By appending the `clear` keyword, all pending events on `%z` are cleared. If `%0` assumes a transient value that lasts less than 1ns, the first change schedules an event to the new value after 1ns, and the second change schedules an event to the old value. If the `clear` keyword is set, the event scheduled by the first change shall be cancelled, thus effectively supressing the value change.

    module @nor_gate_idm (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z %0 1ns clear ; clears any pending events
    }

    module @nor_gate_tdm (u1 %a, u1 %b) (u1 %z) {
        %0 = nor %a %b
        drv %z %0 1ns
    }


### Counter
**VHDL:**

    entity counter is
        port (clk: in bit);
    end counter;

    -- Counter in Variable
    architecture var of counter is
    begin
        count: process (clk)
            variable cnt : integer := 0;
        begin
            if (clk = '1' and clk'last_value = '0') then
                cnt := cnt + 1;
            end if;
        end process;
    end var;

    -- Counter in Signal
    architecture sig of counter is
        signal cnt : integer := 0;
    begin
        count: process (clk)
        begin
            if (clk = '1' and clk'last_value = '0') then
                cnt <= cnt + 1;
            end if;
        end process;
    end sig;

**LLHD:**

    module @counter_var (u1 %clk) () {
        %clk_last_value = alloc u1
        %cnt = alloc i32 i32'0

        %0 = cmpeq %clk u1'1
        %1 = cmpeq %clk_last_value u1'0
        %2 = and %0 %1

        br %2, if_0, endif_0
    if_0:
        %3 = add %cnt i32'1
        st %cnt %3
    endif_0:

        st %clk_last_value %clk
    }

    module @counter_sig (u1 %clk) () {
        %clk_last_value = alloc u1
        %cnt = sig i32 i32'0

        %0 = cmpeq %clk u1'1
        %1 = cmpeq %clk_last_value u1'0
        %2 = and %0 %1

        br %2, if_0, endif_0
    if_0:
        %3 = add %cnt i32'1
        drv %cnt %3
    endif_0:

        st %clk_last_value %clk
    }

An optimization pass is possible that makes the clock edge detection explicit:

    module @counter_var (u1 %clk) () {
        %clk_edge = edge u1'0 u1'1 %clk
        %cnt = alloc i32 i32'0

        br %clk_edge, if_0, endif_0
    if_0:
        %0 = add %cnt i32'1
        st %cnt %0
    endif_0:
    }

    module @counter_sig (u1 %clk) () {
        %clk_edge = edge u1'0 u1'1 %clk
        %cnt = sig i32 i32'0

        br %clk_edge, if_0, endif_0
    if_0:
        %0 = add %cnt i32'1
        drv %cnt %0
    endif_0:
    }
