# llhd

[![Build Status](https://travis-ci.org/fabianschuiki/llhd.svg?branch=master)](https://travis-ci.org/fabianschuiki/llhd)
[![Released API docs](https://docs.rs/llhd/badge.svg)](https://docs.rs/llhd)
[![Crates.io](https://img.shields.io/crates/v/llhd.svg)](https://crates.io/crates/llhd)
![Crates.io](https://img.shields.io/crates/l/llhd)

Welcome to the Low Level Hardware Description language. LLHD aims at introducing a simple and open interface between the compiler frontends of hardware description languages and backend design tools. This allows tools such as simulators and synthesizers to focus on their respective task, rather than implementing a compiler for each supported language. With the compiler detached from the tools, LLHD enables innovation to happen on the language front. Refer to the following documentation:

- [Language Reference](https://github.com/fabianschuiki/llhd/blob/master/LANGUAGE.md)
- [API Documentation](https://docs.rs/llhd/)

LLHD is written in [Rust](https://www.rust-lang.org/). Upon stabilization, a C interface will be added to the library, allowing it to be used from virtually every other programming language.
