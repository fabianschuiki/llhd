# LLHD Language Reference

This document outlines the architecture and instructions of the Low Level Hardware Description language.

# Instructions

## Call (*fpe*)

    call <target> (<args,...>)

The call instruction transfers control to a function declared in the module and yields the function's return value. If used in an entity, the function is reevaluated whenever its input arguments change.


## Instance (*e*)

    inst <target> (<inputs,...>) (<outputs,...>)

The instance instruction instantiates a process or entity declared in the module and connects its inputs and outputs.
