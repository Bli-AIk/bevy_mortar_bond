# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/Bli-AIk/bevy_mortar_bond/compare/v0.3.0...v0.4.0) - 2026-04-27

### Added

- Line grouping, condition caching, and function call conditions ([#14](https://github.com/Bli-AIk/bevy_mortar_bond/pull/14))
- *(ci)* add tokei lint checks to crate workflows
- multi-controller architecture for concurrent dialogues ([#12](https://github.com/Bli-AIk/bevy_mortar_bond/pull/12))
- *(system)* add auto-registration for unloaded mortar files

### Documentation

- *(readme)* rewrite plugin description for content-logic decoupling

### Miscellaneous Tasks

- *(lint)* improve #[expect] reason detection in tokei scripts
- add clippy configuration

### Other

- Based on the git status and diff, I can see there's only one staged change: updating the `mortar_compiler` dependency version from "0.5.2" to "0.5" in Cargo.toml.

### Refactor

- standardize terminology from "资产" to "资源"
- split runtime/dialogue modules and break up oversized tests ([#15](https://github.com/Bli-AIk/bevy_mortar_bond/pull/15))
- *(ui)* remove clippy expect attributes from example functions
- *(deps)* update bevy dependencies to disable default features
- *(examples)* replace clippy allow with expect attributes
- extract UI functions to reduce nesting depth
- fix all clippy excessive_nesting warnings

## [0.3.0](https://github.com/Bli-AIk/bevy_mortar_bond/compare/v0.2.1...v0.3.0) - 2026-02-11

### Added

- [**breaking**] upgrade to bevy 0.18

### Documentation

- *(readme)* update Bevy version support and dependencies

## [0.2.1](https://github.com/Bli-AIk/bevy_mortar_bond/compare/v0.2.0...v0.2.1) - 2026-02-06

### Miscellaneous Tasks

- *(ci)* update GitHub Actions to latest versions
- *(deps)* update mortar_compiler to published version
