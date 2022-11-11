# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] 2022-11-09
No library changes, just bumping from release candidate to a full release.

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
