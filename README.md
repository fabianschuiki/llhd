# llhd

[![Build Status](https://travis-ci.org/fabianschuiki/llhd.svg?branch=rust)](https://travis-ci.org/fabianschuiki/llhd)

Welcome to the Low Level Hardware Description language. LLHD aims at introducing a simple and open interface between the compiler frontends of hardware description languages and backend design tools. This allows tools such as simulators and synthesizers to focus on their respective task, rather than implementing a compiler for each supported language. With the compiler detached from the tools, LLHD enables innovation to happen on the language front. New approaches can be implemented without the need for support by established tools.

LLHD is written in Rust, but is available as a fully-featured C library for use in virtually any environment.


## Roadmap and Milestones

- [ ] emit in-memory representation as assembly
- [ ] parse assembly into in-memory representation
- [x] create processes, entities, functions
- [x] integer arithmetic instructions
- [x] call, inst instruction
- [ ] wait, return, branch, phi instructions
- [ ] signal, probe, drive instructions
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
- [ ] pointer type
- [ ] alloc, free, stack, load, store instructions
- **Milestone:** can represent arbitrary programs
- [ ] struct, array, vector type
- [ ] element extraction, insertion instructions
- **Milestone:** can implement polymorphism efficiently
- [ ] emit in-memory representation as bitcode
- [ ] parse bitcode into in-memory representation
