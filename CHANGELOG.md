# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- Add function to lookup external units in a module.

### Changed
- Data flow instructions in entities are no longer automatically of signal type.
- Leading `%` in block names may now be omitted.
- Make types in syntax of `extf`, `exts`, `insf`, and `inss` more explicit.
- Increase minimum rustc version to 1.36.

### Fixed
- Limit `wait` to processes.
- Limit `inst` to entities.
- Fix reader not always accepting non-uniform arrays like `[i32 %0, %1]`.
- Fix reader not accepting `shl`, `shr`, and `mux` instructions.
- Fix `extf` and `exts` sometimes having wrong return type.

## 0.8.0 - 2019-08-07
### Added
- Add `llhd-check` tool to verify consistency of assembly files.
- Add `const iN$` flavor to generate a constant integer signal.
- Add `llhd-opt` tool to perform assembly optimization.
- Add constant folding pass.
- Add dead code elimination pass.

### Changed
- Triggers for `reg` instruction now require type annotation.

### Fixed
- Fix `reg` instruction writer output having data and triggers mangled.
- Fix `remove_inst` for instructions not yielding a result.
- Fix `prune_if_unused` for instructions not yielding a result.

## 0.7.1 - 2019-05-04
### Fixed
- Make llhd-conv more robust in presence of unknown LIB pin functions.

## 0.7.0 - 2019-05-02
### Added
- Add `llhd-conv` tool to convert between intermediate representations.

## 0.6.0 - 2019-05-02
### Added
- Add `ir` module.
- Add `lalrpop` dependency.
- Add `write_module`, `write_module_string`, `parse_type`, `parse_time`, and `
parse_module` to the `assembly` module.

### Changed
- Update assembly reader and writer to new IR module.
- Update `num` to version 0.2.

### Removed
- Remove `write` and `write_string` from the `assembly` module.
- Remove `visit` module.
- Remove `Module` in favor of `ir::Module`.
- Remove `Entity` in favor of `ir::Entity`.
- Remove `Function` in favor of `ir::Function`.
- Remove `Process` in favor of `ir::Process`.
- Remove `seq_body` module.
- Remove `block` module.
- Remove `argument` module.
- Remove `inst` module.
- Remove `unit` module.
- Remove `Const` and `ConstInt` in favor of the `const` instruction.
- Remove `aggregate` module in favor of `array` and `struct` instructions.
- Remove `value` module.
- Remove `combine` dependency.

### Fixed
- Fix emission of time constants.

## 0.5.0 - 2019-02-19
### Added
- Signal and pointer semantics for `extract`. ([#41](https://github.com/fabianschuiki/llhd/issues/41))
- Constructor for constant aggregate values.

### Changed
- Make `shl` and `shr` separate instructions.

### Fixed
- Fix parsing of spaces in time constants.
- Fix parsing of dots in names. ([#44](https://github.com/fabianschuiki/llhd/issues/44))

## 0.4.0 - 2019-02-05
### Added
- Add `insert` and `extract` instructions.
- Add missing `unwrap_*` and `is_*` functions to types. ([#35](https://github.com/fabianschuiki/llhd/issues/35))
- Add aggregate values. ([#39](https://github.com/fabianschuiki/llhd/issues/39))
- Add struct and array type parsing.

### Changed
- Rename `as_*` functions on types to `unwrap_*`.
- Rename `VectorType` to `ArrayType`. ([#24](https://github.com/fabianschuiki/llhd/issues/24))
- Change vector syntax from `<N x T>` to `[N x T]`.
- Swap order of `store` instruction parameters such that the pointer comes first.
- Change type of delta and epsilon time steps to `usize`.

### Fixed
- Fix representation of times in assembly; always print seconds. ([#36](https://github.com/fabianschuiki/llhd/issues/36))

## 0.3.1 - 2019-01-17
### Fixed
- Fix type lookup of constants via `Context::ty`.

## 0.3.0 - 2019-01-15
### Added
- Make `ValueRef`, `InstKind`, `ReturnKind`, and `BranchKind` comparable.
- Add `var`, `load`, and `store` instructions.

## 0.2.1 - 2019-01-15
### Fixed
- Fix blocks with temporary names not being parsed properly.
- Fix parsing of comparison operations.
- Fix time constant printing. ([#28](https://github.com/fabianschuiki/llhd/issues/28))

## 0.2.0 - 2019-01-14
### Added
- Add `unwrap_*` functions on `ValueRef`.
- Add `write` and `write_string` convenience functions. ([#26](https://github.com/fabianschuiki/llhd/issues/26))

### Fixed
- Emit instance names. ([#21](https://github.com/fabianschuiki/llhd/issues/21))
- Parse temporary names as `None`. ([#30](https://github.com/fabianschuiki/llhd/issues/30))
- Fix parsing of binary operations. ([#31](https://github.com/fabianschuiki/llhd/issues/31))

### Changed
- Make most modules private.
- Re-export contents of modules directly. ([#22](https://github.com/fabianschuiki/llhd/issues/22), [#23](https://github.com/fabianschuiki/llhd/issues/23))
- Make `util::write_implode` and `util::write_implode_with` private.
- Accept anything that converts to a string as name of entities, functions, and processes.

## 0.1.0 - 2018-02-27
### Added
- Initial release
