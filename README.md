# LLHD

[![Build Status](https://travis-ci.org/fabianschuiki/llhd.svg?branch=master)](https://travis-ci.org/fabianschuiki/llhd)
[![Released API docs](https://docs.rs/llhd/badge.svg)](https://docs.rs/llhd)
[![Crates.io](https://img.shields.io/crates/v/llhd.svg)](https://crates.io/crates/llhd)
![Crates.io](https://img.shields.io/crates/l/llhd)
[![dependency status](https://deps.rs/repo/github/fabianschuiki/llhd/status.svg)](https://deps.rs/repo/github/fabianschuiki/llhd)

The *Low Level Hardware Description language* is an intermediate representation for digital circuit descriptions, together with an accompanying simulator and SystemVerilog/VHDL compiler.

LLHD separates input languages from EDA tools such as simulators, synthesizers, and placers/routers. This makes writing such tools easier, allows for more rich and complex HDLs, and does not require vendors to agree upon the implementation of a language.

[Try it yourself!](http://llhd.io/)

LLHD is being developed as part of [CIRCT](https://github.com/circt), a larger community effort to establish an open hardware design stack.

## Scientific Work

The scientific paper on LLHD is available on arXiv:

- F. Schuiki, A. Kurth, T. Grosser, L. Benini (2020). *"LLHD: A Multi-level Intermediate Representation for Hardware Description Languages".* [arXiv:2004.03494](https://arxiv.org/abs/2004.03494) ([PDF](https://arxiv.org/pdf/2004.03494), [Recording at PLDI'20](https://www.youtube.com/watch?v=jOgbMVDf8Dc))

Are you interested in using open-source ideas to re-invent the hardware design software stack? Do you see LLHD as step one of a bigger picture and dream about extending it with formal verification, hardware synthesis, etc.? We are continuously looking for future PhD students and postdocs who are excited to work in this direction. For more details check out http://grosser.science or just write an informal email to tobias@grosser.es for us to discuss potential next steps.

## Documentation

- [Language Reference](http://llhd.io/spec.html) ([source](https://github.com/fabianschuiki/llhd/blob/master/doc/LANGUAGE.md))
- [API Documentation](https://docs.rs/llhd/)
