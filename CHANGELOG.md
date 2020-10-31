# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- Update dependencies `indoc`, `num`, `regex`, `lalrpop-util`, `time`
- Increase minimum rustc version to 1.45

## 0.14.2 - 2020-09-17
### Fixed
- Fix entity simulation panic caused by `halt` instruction (#143)

## 0.14.1 - 2020-09-05
### Fixed
- Fix faulty constant folding on signal/pointer shifts (#138)

## 0.14.0 - 2020-09-05
### Added
- Add `llhd-sim` tool (merged in from https://github.com/fabianschuiki/llhd-sim)
- Add MLIR output to `llhd-conv`
- Add `pretty_env_logger` dependency

### Fixed
- Fix implementation of `IntValue::smod`.

### Removed
- Remove `stderrlog` and `env_logger` dependencies

## 0.13.0 - 2020-04-13
### Added
- Add `UnitData` struct.
- Add `Unit` and `UnitBuilder` structs.
- Add `analysis` module.
- Add `trg()`, `predtbl()`, `domtree()`, and `domtree_with_predtbl()` to `Unit`.
- Extend dominator tree queries to cover all block, instruction, and value combinations.

### Changed
- Fold `ControlFlowGraph`, `DataFlowGraph`, and `FunctionLayout` functions into `Unit` and `UnitBuilder`.
- Change `dump()` functions to take a `&Unit` instead of DFG/CFG.
- Make `sig`, `dfg`, `cfg`, and `layout` fields of `UnitData` private.
- Make `FunctionLayout` and `InstLayout` structs private.
- Make `ControlFlowGraph` and `DataFlowGraph` structs private.
- Factor out `TemporalRegionGraph`, `PredecessorTable`, and `DominatorTree` into `analysis` module.
- Rename `licm` pass to `ecm`.

### Deprecated
- Deprecate calling `new` on `TemporalRegionGraph`, `PredecessorTable`, and `DominatorTree`, in favor of accessors on `Unit`.

### Removed
- Remove `Function`, `Process`, and `Entity` structs.
- Remove `Unit` and `UnitBuilder` traits.
- Remove `Layout` trait.
- Remove `func_layout()` and `func_layout_mut()` functions.
- Remove `cfg()`, `cfg_mut()`, `try_cfg()`, and `try_cfg_mut()` functions.
- Remove `dfg()` and `dfg_mut()` functions.
- Remove `sig()` and `name()` from `UnitBuilder`.

## 0.12.0 - 2020-04-09
### Added
- llhd-opt: Add `-p` option to specify exact passes to be executed.
- Add instruction simplification pass. ([#92](https://github.com/fabianschuiki/llhd/issues/92))
- Add optional condition to drive instruction. ([#97](https://github.com/fabianschuiki/llhd/issues/97))
- Add process lowering pass. ([#93](https://github.com/fabianschuiki/llhd/issues/93))
- Add desequentialization pass. ([#103](https://github.com/fabianschuiki/llhd/issues/103))
- Add `reg` gating conditions. ([#105](https://github.com/fabianschuiki/llhd/issues/105))
- llhd-opt: Add `-l` option to lower from behavioural to structural LLHD.
- llhd-conv: Add `-i`, `-o`, `--input-format`, `--output-format` options.

### Changed
- Use dense vector table instead of hash map for blocks, instructions, values, and external unit data.
- Improve dominator tree computation performance.
- Add auxiliary temporal region entry blocks during TCM.
- Improve value/block use lookup performance. ([#91](https://github.com/fabianschuiki/llhd/issues/91))
- Fold `mux` instructions with constant selector.
- Fold `extf` instructions on constant arrays and structs.
- Wrap register triggers in `RegTrigger`.
- Changed `reg` and `del` to return `void`, and take target signal as operand.

### Fixed
- Fix instructions in entry block being reordered during LICM.
- Fix `drv` instructions being removed during TCM. ([#100](https://github.com/fabianschuiki/llhd/issues/100))
- Fix value names not being uniquified properly in assembly writer.

## 0.11.0 - 2020-02-08
### Added
- ir: Add various `unit()`, `unit_mut()`, `get_unit()`, and `get_unit_mut()` functions.
- ir: Add location hint tracking for units and values.
- llhd-check: Add `--emit-trg` option.
- llhd-check: Add verbosity options.
- Add `Layout` trait with functionality shared between `InstLayout` and `FunctionLayout`.

### Changed
- Extend verifier to check if used values and blocks have a definition.

### Fixed
- llhd-check: Honor dump flag `-d`.
- Fix Temporal Region Graph computation not producing distinct regions for blocks that may execute at different points in time.
- Fix broken phi nodes after block removal. ([#87](https://github.com/fabianschuiki/llhd/issues/87))

## 0.10.0 - 2019-11-24
### Added
- Add Sublime Text syntax highlighting.
- Add function/process/entity and name getters to `Unit`.
- Add `--dump` option to `llhd-check`.
- Add `name()`, `const_zero()`, and `suffix()` to instruction builder.
- Add `named_block()` to unit builder.
- Add `log` dependency.
- Add `value` module with utilities to deal with values.
- Add `get_const*` functions to the DFG.
- Add fallible `try_cfg*` functions to units.
- Add `len()` for types.
- Add `stderrlog` dependency.
- Add `rayon` dependency.
- Add `time` dependency.
- Add `-v`, `-t`, and `-s` options to `llhd-opt`.
- Add `opt` module for optimization infrastructure.
- Add Global Common Subexpression Elimination pass.
- Add Temporal Code Motion pass.
- Add `phi` instruction.
- Add Loop Independent Code Motion Pass.
- Add Control Flow Simplification Pass.
- Add Variable to Phi Promotion Pass.
- Add `serde` dependency.
- Add serialization/deserialization for the IR.

### Changed
- Add missing `dyn` keywords.
- Preserve anonymous names as hints in the DFG.
- Use names when dumping values.
- Make `Unit` object-safe.
- Allow comments in certain locations in the assembly.
- Make `ty` module visible.
- Change `const` instructions to use `IntValue` and `TimeValue`.
- Improve constant folding.

### Removed
- Remove `konst` module.

### Fixed
- Fix issue in `insert_inst_before` of block layouts.
- Fix instruction insertion position being invalidated on instruction removal.

## 0.9.0 - 2019-10-24
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
