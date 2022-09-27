# Versioning
This project uses the SemVer versioning format.

Release version format: `[major].[minor].[patch]`

Prerelease version format: `[major].[minor].[patch]-[prerelease tag]`

## Prerelease stages
- Alpha: active development of new features (prerelease tag: `alpha`)
- Beta: feature set frozen, bug fixing and usability improvements (prerelease tag: `beta`)
- Release candidate: potentially stable, final bug ID and fixing (prerelease tag: `RC[release candidate number]`)

## Version incrementing
### Before 1.0
- Any API changes increment [minor] version

### After 1.0
- API-breaking changes increment [major] version
- Backwards-compatible API changes increment [minor] version
