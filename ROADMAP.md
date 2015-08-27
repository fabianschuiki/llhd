# Roadmap for the LLHD Project

## Programs
The user of LLHD shall interact with the system through the use of several distinct programs, much like GCC and Clang. The programs expose the functionality of the LLHD libraries. The programs are as follows:

-   **llhd-vhdl** acts as the VHDL compiler whose main task it is to convert VHDL source code to LLHD. May also be used to syntax check, parse, and rewrite VHDL code. Allows LLHD code to be synthesized back into VHDL.

-   **llhd-verilog** acts as the Verilog compiler whose main task it is to convert Verilog source code to LLHD. May also be used to syntax check, parse, and rewrite Verilog code. Allows LLHD code to be synthesized back into Verilog.

-   **llhd-systemverilog** acts as the SystemVerilog compiler whose main task it is to convert SystemVerilog source code to LLHD. May also be used to syntax check, parse, and rewrite SystemVerilog code. Allows LLHD code to be synthesized back into SystemVerilog.

-   **llhd-sim** is a reference implementation of a hardware simulator. It shall be a proof-of-concept that VHDL, Verilog, and SystemVerilog code can be compiled into LLHD and simulated, and that the results line up properly.


## Libraries
The source code is separated into several distinct sets of functionality which are compiled into separate libraries to ensure modularity. The libraries are as follows:

-   **libllhd-common** contains the common code that is shared among the other libraries such as memory management, utilities, diagnostics, and the like. In general, if some source code cannot be associated with any other library, it goes in here. Connected parts of the common library that appear to form a specific subsystem can later be extracted into a separate library.

-   **libllhd-vhdl** contains VHDL-specific code such as an AST, lexer, parser, writer, and compiler.

-   **libllhd-verilog** contains Verilog-specific code such as an AST, lexer, parser, writer, and compiler.

-   **libllhd-systemverilog** contains SystemVerilog-specific code such as an AST, lexer, parser, writer, and compiler. Might also pull in dependencies from the *libllhd-verilog* library if need be, since the two languages share a common foundation, or rather SystemVerilog is a superset of Verilog.
