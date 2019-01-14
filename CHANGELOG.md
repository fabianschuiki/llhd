# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- Add `unwrap_*` functions on `ValueRef`.

### Fixed
- Emit instance names. ([#21](https://github.com/fabianschuiki/llhd/issues/21))
- Parse temporary names as `None`. ([#30](https://github.com/fabianschuiki/llhd/issues/30))

### Changed
- Make most modules private.
- Re-export contents of modules directly. ([#22](https://github.com/fabianschuiki/llhd/issues/22), [#23](https://github.com/fabianschuiki/llhd/issues/23))
- Make `util::write_implode` and `util::write_implode_with` private.
- Accept anything that converts to a string as name of entities, functions, and processes.

## 0.1.0 - 2018-02-27
### Added
- Initial release
