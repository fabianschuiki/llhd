# Roadmap for the LLHD Project

## Programs
The user of LLHD shall interact with the system through the use of several distinct programs, much like GCC and Clang. The programs expose the functionality of the LLHD libraries. The programs are as follows:

-   **llhd-tool** exposes some of the functionality of the LLHD libraries on the command line. This includes syntax checking for LLHD source files, converting between assembly and bitcode, applying various transformations, and extracting information on the assembly at hand.

-   **llhd-vhdlc** acts as the VHDL compiler whose main task it is to convert VHDL source code to LLHD. May also be used to syntax check, parse, and rewrite VHDL code. Allows LLHD code to be synthesized back into VHDL.

-   **llhd-vc** acts as the Verilog compiler whose main task it is to convert Verilog source code to LLHD. May also be used to syntax check, parse, and rewrite Verilog code. Allows LLHD code to be synthesized back into Verilog.

-   **llhd-svc** acts as the SystemVerilog compiler whose main task it is to convert SystemVerilog source code to LLHD. May also be used to syntax check, parse, and rewrite SystemVerilog code. Allows LLHD code to be synthesized back into SystemVerilog.

-   **llhd-sim** is a reference implementation of a hardware simulator. It shall be a proof-of-concept that VHDL, Verilog, and SystemVerilog code can be compiled into LLHD and simulated, and that the results line up properly.


## Libraries
The source code is separated into several distinct sets of functionality which are compiled into separate libraries to ensure modularity. The libraries are as follows:

-   **libllhd-common** contains the common code that is shared among the other libraries such as memory management, utilities, diagnostics, and the like. In general, if some source code cannot be associated with any other library, it goes in here. Connected parts of the common library that appear to form a specific subsystem can later be extracted into a separate library.

-   **libllhd-vhdl** contains VHDL-specific code such as an AST, lexer, parser, writer, and compiler.

-   **libllhd-verilog** contains Verilog-specific code such as an AST, lexer, parser, writer, and compiler.

-   **libllhd-systemverilog** contains SystemVerilog-specific code such as an AST, lexer, parser, writer, and compiler. Might also pull in dependencies from the *libllhd-verilog* library if need be, since the two languages share a common foundation, or rather SystemVerilog is a superset of Verilog.


## Steps
The following is a very coarse estimate of the work required to get LLHD to a point where it is actually useful.

- specify components and final programs that the user interacts with *(1d)*
- specify assembly *(3d)*
  - structure and event model
  - drive conflict resolution
  - instructions
  - data types
  - metadata
- overhaul diagnostic subsystem *(0.5d)*
- restructure source code to match atlas *(0.5d)*
- implement assembly representations (in-memory, bitcode, human-readable) *(4d)*
  - assembly reader / writer *(2d)*
  - bitcode reader / writer *(2d)*
- implement reference simulator *(5d)*
  - event queue and value representation *(1d)*
  - simulation stepping and value change dump *(2d)*
  - frontend program and its arguments *(1d)*
- implement VHDL compiler *(9d)*
  - lexer *(1d)*
  - parser (needs some template magic to get an LR(1) parser) *(3d)*
  - code generation *(5d)*
- implement Verilog compiler *(6.5d)*
  - lexer (reuse VHDL lexer where possible) *(0.5d)*
  - parser *(2d)*
  - code generation *(4d)*
- implement SystemVerilog compiler *(8.5d)*
  - lexer (reuse Verilog lexer where possible) *(0.5d)*
  - parser *(3d)*
  - code generation *(5d)*
- implement assembly synthesizer *(2d)*
  - to VHDL
  - to Verilog
  - to SystemVerilog
