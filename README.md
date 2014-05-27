An attempt at writing a low-level hardware description toolkit. First up is a
parser and code generator for VHDL.

## Design Guidelines

- source files have suffix `.cpp`
- header files have suffix `.hpp`
- sources and headers both live in the `llhd` directory
- everything lives in the `llhd` namespace
- files may be grouped into directories for better readability and structure
- files containing sub-namespaces of `llhd` must be placed in a directory with
  the same name as the namespace
- directory names are lowercase
