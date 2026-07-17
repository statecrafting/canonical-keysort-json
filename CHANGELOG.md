# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/statecrafting/canonical-keysort-json/releases/tag/v0.1.0
