# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] 2022-11-22
Readme fix, no library changes

## [0.2.0] 2022-11-22
### Added
- Added the ability to set proxies for the client to use. See the docs for
more info.

### Changed
- Replaced thiserror with snafu for error implementions. There are some changes
to the error enum variants, check the docs if you depend on a specific variant.
- Depreciated RegisteredClient::get_interaction_url() in favor of
RegisteredClient::get_interaction_fqdn(). This is purely for naming accuracy, it returns the
same string as RegisteredClient::get_interaction_url() did.

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
