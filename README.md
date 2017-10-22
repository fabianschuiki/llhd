# llhd

[![Build Status](https://travis-ci.org/fabianschuiki/llhd.svg?branch=rust)](https://travis-ci.org/fabianschuiki/llhd)
[![Documentation Status](https://readthedocs.org/projects/llhd/badge/?version=latest)](https://llhd.readthedocs.io/en/latest/?badge=latest)

Welcome to the Low Level Hardware Description language. LLHD aims at introducing a simple and open interface between the compiler frontends of hardware description languages and backend design tools. This allows tools such as simulators and synthesizers to focus on their respective task, rather than implementing a compiler for each supported language. With the compiler detached from the tools, LLHD enables innovation to happen on the language front. Refer to the following documentation:

- [Language Reference](https://llhd.readthedocs.io/)
- [API Documentation](https://fabianschuiki.github.io/llhd/)

LLHD is written in [Rust](https://www.rust-lang.org/). Upon stabilization, a C interface will be added to the library, allowing it to be used from virtually every other programming language.


## Roadmap and Milestones

- [x] emit in-memory representation as assembly
- [x] parse assembly into in-memory representation
- [x] create processes, entities, functions
- [x] integer arithmetic instructions
- [x] call, inst instruction
- [x] wait, return, branch
- [x] signal, probe, drive instructions
- [x] time literals
- [ ] global values, convention to declare "main"
- [x] halt instruction
- [ ] validate types and terminators in module
- **Milestone:** can write simulator
- [ ] attach metadata to arbitrary nodes in the graph
- **Milestone:** can extend simulator with debugging mode
- [ ] declare prototype processes, entities, functions
- [ ] link modules
- **Milestone:** can write linker
- [ ] logic type
- [ ] logic arithmetic instructions
- [ ] conversion between 9-value logic, 4-value logic, and integer
- **Milestone:** can efficiently model SystemVerilog/VHDL hardware
- [x] pointer type
- [ ] alloc, free, stack, load, store instructions
- **Milestone:** can represent arbitrary programs
- [ ] struct, array, vector type
- [ ] element extraction, insertion instructions
- **Milestone:** can implement polymorphism efficiently
- [ ] phi instruction
- [ ] add unsigned/signed mul, div, mod, rem
- [ ] emit in-memory representation as bitcode
- [ ] parse bitcode into in-memory representation
