# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-10-04

No change to emitted bytes: every input serializes exactly as under 0.1.0.

### Changed

- The crate description, README and module docs no longer call this crate
  "Canonical JSON". It sorts object keys only and is byte-incompatible with
  gibson042 Canonical JSON and RFC 8785.

### Added

- The cross-language canonical-form documentation on `canonicalize_value`
  (code point order rather than UTF-16 code unit order, no numeric carve-out
  for integer-like keys, unpaired surrogates), which landed after 0.1.0 was
  published.
- Property tests asserting byte stability across key-insertion order,
  strictly sorted output, lossless round-trip and idempotence over generated
  values with non-ASCII, astral and integer-like keys.

### Fixed

- The test suite now runs with `serde_json/preserve_order` enabled (a
  dev-dependency feature, so consumers are unaffected). Previously every test,
  including the byte-stability gate, passed even with `canonicalize_value`
  replaced by the identity, because a `BTreeMap`-backed `Map` already iterates
  in sorted order.

## [0.1.0] - 2026-07-13

### Added

- Initial release. `canonicalize_value(Value) -> Value` and
  `to_canonical_string(&Value) -> String`: a recursive lexicographic sort of
  JSON object keys at the serialization boundary, so a `serde_json::Value`
  serializes byte-identically regardless of `serde_json`'s `preserve_order`
  feature state anywhere in the dependency graph.
- Byte-stability determinism test asserting identical canonical output across
  key-insertion-order permutations.
- Extracted and relicensed Apache-2.0 (by the sole copyright holder) from the
  Open Agentic Platform's `crates/canonical-json`. See `NOTICE`.

[0.1.1]: https://github.com/statecrafting/canonical-keysort-json/releases/tag/v0.1.1
[0.1.0]: https://github.com/statecrafting/canonical-keysort-json/releases/tag/v0.1.0
