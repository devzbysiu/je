# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2021-11-07
### Added
- Mechanism for adjusting the old configuration file to the new version via `je reinit`
- Support for bundles (packs of files)
- Regex type mechanism for ignoring properties

### Changed
- [Fix](https://github.com/devzbysiu/je/commit/a405d0240562a1766c3aa60c7e541decfbb66af7) issue with cross-partition file move on windows: [#3](https://github.com/devzbysiu/je/issues/3)
- [Fix](https://github.com/devzbysiu/je/commit/ccb132618ea5047bfbbdd7c4eb26972bfe9aad64) issue with incorrect paths in zip file

## [0.2.0] - 2020-25-10
### Added

- Support for profiles via `--profile`.
