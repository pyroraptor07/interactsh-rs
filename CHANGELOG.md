# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Removed the explicit backtrace field from the errors when run under the
nightly toolchain. This appears to not be needed as eyre is still able to
get the error backtrace without it on both nightly and stable.

## [0.1.0-RC2] 2022-10-29
### Changed
- Removed the "reqwest-" prefix from the TLS feature flags. The old TLS
feature flag names have been left in for now and just activate the new feature flags.
The changes are as follows:
    - `reqwest-rustls-tls` changed to `rustls-tls`
    - `reqwest-native-tls` changed to `native-tls`
    - `reqwest-native-tls-vendored` changed to `native-tls-vendored`

## [0.1.0-RC1] 2022-10-15
Release candidate for the initial published crate version.

### Added
- Initial client implementation
- Examples of use
