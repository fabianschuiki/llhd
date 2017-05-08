# llhd

Welcome to the Low Level Hardware Description language. LLHD aims at introducing a simple and open interface between the compiler frontends of hardware description languages and backend design tools. This allows tools such as simulators and synthesizers to focus on their respective task, rather than implementing a compiler for each supported language. With the compiler detached from the tools, LLHD enables innovation to happen on the language front. New approaches can be implemented without the need for support by established tools.

LLHD is written in Rust, but is available as a fully-featured C library for use in virtually any environment.
