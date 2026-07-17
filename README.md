# canonical-keysort-json

Deterministic Canonical JSON: a recursive lexicographic sort of object keys at
the serialization boundary, so a value serializes byte-identically regardless
of `serde_json`'s `preserve_order` feature state anywhere in the dependency
graph.

It exists because `preserve_order` unifies monotonically under Cargo's
resolver 2: one crate enabling it silently flips key order for every dependent
that emits JSON. The robust fix is explicit canonicalization at the emission
boundary, in one shared crate every hashing consumer routes through, so
everyone agrees on the canonical byte string that record hashes are computed
over.

```rust
use canonical_keysort_json::{canonicalize_value, to_canonical_string};
use serde_json::json;

let v = json!({ "z": 1, "a": { "n": 2, "b": 3 } });
assert_eq!(to_canonical_string(&v), r#"{"a":{"b":3,"n":2},"z":1}"#);
```

- `canonicalize_value(Value) -> Value` recursively sorts object keys; arrays
  and scalars pass through unchanged (array ordering is the caller's
  responsibility).
- `to_canonical_string(&Value) -> String` canonicalizes then serializes: the
  byte string a consumer feeds to its hash function.

Hashing itself is deliberately **not** in this crate; it lives in
`attest-ledger`, which owns record-hash construction. This crate guarantees
only the canonical bytes.

## Name

Published on crates.io as **`canonical-keysort-json`** (import
`canonical_keysort_json`). The plain `canonical-json` / `canonical_json` name
is taken by Mozilla's `canonicaljson-rs`, which implements gibson042's stricter
Canonical JSON spec (non-ASCII escaping, float normalization) and is
byte-incompatible with this minimal key-sort. `keysort` is the honest signal:
it sorts keys, it does not do full canonical JSON.

## Ecosystem

Part of the `statecrafting` reusable-primitive family. It is the leaf
dependency of `attest-ledger` (record hashing) and `action-gate` (decision
serialization), extracted from the Open Agentic Platform and relicensed
Apache-2.0 by the sole copyright holder (see `NOTICE`). This repo is
self-governed by its own `specs/` corpus, compiled by the pinned `spec-spine`
library.

Licensed under Apache-2.0.
