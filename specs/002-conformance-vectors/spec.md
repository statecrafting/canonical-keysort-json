---
id: "002-conformance-vectors"
title: "Language-neutral conformance vectors for the canonical form"
status: draft
implementation: pending
created: "2026-10-04"
kind: tooling
risk: medium
summary: >
  A committed, versioned, language-neutral vector file that states spec 000
  section 4 as data: inputs as JSON text, expected canonical output as both a
  string and its UTF-8 bytes in hex, and inputs every implementation must
  reject. The Rust crate runs every vector as a test, so the file cannot drift
  from the reference implementation, and a port in another language
  (claude-observatory's journal.ts is the first) checks itself against the
  same file instead of against prose. It exists because the one divergence
  this family has had was invisible to every test either side owned, and would
  have failed the first vector here that exercised it.
depends_on:
  - "000-canonical-keysort-json-bootstrap"
extends:
  - { spec: "000-canonical-keysort-json-bootstrap", unit: "src/lib.rs", nature: additive }
  - { spec: "000-canonical-keysort-json-bootstrap", unit: "Cargo.toml", nature: additive }
references:
  - { unit: { kind: file, path: "README.md" }, role: context }
---

# 002: Language-neutral conformance vectors

## 1. Purpose

Spec 000 section 4 is normative for every implementation of this canonical
form, and today it is held to the Rust implementation by Rust tests in
`src/lib.rs`. Those tests are the wrong shape for anyone else: a port cannot
run them, so it reimplements its understanding of the prose and tests that.
That is exactly how `journal.ts` came to define a different canonical form for
integer-like keys and astral characters while both sides were green (spec 001
section 1).

This spec moves the contract's examples out of Rust and into data. One file,
read by the Rust test suite and by any port, so that agreement between two
implementations is a fact both can check rather than an assumption.

## 2. Territory

While this spec is a draft the territory is stated here in prose only: a
`vectors/` claim in `establishes` before the directory exists is an unresolved
unit, and the gate refuses it. The build adds
`{ kind: directory, path: "vectors/" }` to `establishes` in the same change
that creates the directory.

- `vectors/canonical-form.json`: the vector file (established).
- `vectors/README.md`: the file format and the rules for changing it, for a
  reader who arrives from a port and never opens `specs/` (established).
- `src/lib.rs`: a test that loads the vector file and runs every entry
  against `to_canonical_string` (extends spec 000; additive, test code only).
- `Cargo.toml`: `vectors/**` joins the `include` list so the packaged crate
  carries the file its own tests read (extends spec 000; additive).

## 3. Behavior

### 3.1 File format

The file is a single JSON object, UTF-8, no BOM:

```json
{
  "format": "canonical-keysort-json/vectors/v1",
  "contract": "specs/000-canonical-keysort-json-bootstrap/spec.md section 4",
  "vectors": [
    {
      "id": "astral-after-bmp",
      "rule": "4.1",
      "normative": true,
      "description": "U+1F600 sorts after U+FFFD by code point; UTF-16 order reverses them.",
      "input": "{\"\\ud83d\\ude00\":1,\"\\ufffd\":2}",
      "expected": "{\"\ufffd\":2,\"\ud83d\ude00\":1}",
      "expected_utf8_hex": "7b22efbfbd223a322c22f09f9880223a317d"
    },
    {
      "id": "lone-high-surrogate-key",
      "rule": "4.5",
      "normative": true,
      "description": "An unpaired surrogate is not representable and must be rejected.",
      "input": "{\"\\ud800\":1}",
      "reject": true
    }
  ]
}
```

- `input` MUST be JSON text carried as a string, not an embedded JSON value.
  An embedded value would be parsed by the port's own reader before the test
  sees it, which is the step that destroys integer-like key order in
  JavaScript; a string lets the port feed its real ingestion path.
- `expected` and `expected_utf8_hex` MUST both be present on an accepting
  vector and MUST agree. The hex is the authority: it is what a hash function
  sees, and it is immune to how a reader decodes the vector file's own
  escapes.
- A vector with `"reject": true` carries no `expected`. A conforming
  implementation MUST refuse the input at its boundary rather than emit any
  bytes for it.
- `normative: false` marks a vector that pins what the Rust reference emits
  for an input outside the section 4 guarantee (the non-guarantees on number
  representation). A port MAY skip these; the Rust suite MUST still pass them,
  so a change in `serde_json`'s number formatting is noticed.
- `id` values are unique, kebab-case, and stable.

### 3.2 Required coverage

The file MUST contain at least one normative vector for each of:

1. Top-level key sort, and recursive sort at a depth of three or more.
2. Objects inside arrays, and array element order preserved (never sorted).
3. Code point order against UTF-16 code unit order (an astral key against a
   key in U+E000..=U+FFFF).
4. Integer-like keys sorting lexicographically (`"10"` before `"2"`), at the
   top level and nested.
5. The empty-string key, and a key that is a prefix of another.
6. Compact output: an input with insignificant whitespace.
7. The minimal escape set: every C0 control, `"` and `\`, DEL, and U+2028 and
   U+2029 left unescaped.
8. Non-ASCII emitted raw, not `\u`-escaped.
9. Rejection of an unpaired surrogate in key position and in value position.

And informative (`normative: false`) vectors for: `1.0`, `-0.0`, an exponent
form, and an integer beyond 2^53.

### 3.3 The Rust suite

A test in `src/lib.rs` MUST read the file with `include_str!`, and for every
vector:

- accepting: parse `input` with `serde_json`, assert `to_canonical_string`
  equals `expected`, and assert its UTF-8 bytes equal `expected_utf8_hex`;
- rejecting: assert `serde_json::from_str` refuses `input`.

It MUST also assert the coverage of 3.2 by `rule` tags, so a vector cannot be
deleted without a test failing, and that `id`s are unique.

The test runs under the `preserve_order` test build spec 000 section 7
established; without it, the sort vectors would pass against an identity
canonicalizer.

### 3.4 Change discipline

- The file is append-only. Adding a vector is a change to this spec.
- Changing or removing the `expected` of a normative vector changes the
  canonical form, which spec 000 says breaks every downstream hash. It
  requires a spec 000 change in the same pull request and a major version.
- `format` changes only when the schema changes incompatibly; a port MUST
  refuse a `format` it does not know.

## 4. Out of scope

- Any port, including fixing `journal.ts`; that lives in its own repository
  and consumes this file.
- Publishing the vectors outside crates.io (npm, a standalone release asset).
  A port pins a git tag of this repository until there is a reason to do more.
- Generating vectors. They are authored by hand and the Rust suite is the
  oracle; a generator would make the reference implementation the source of
  its own test data.
- Hashes of the expected bytes. Hashing belongs to `attest-ledger`, and the
  hex is already exact.

## 5. Open questions for approval

- **Location.** `vectors/` at the root, so a port can fetch
  `https://raw.githubusercontent.com/statecrafting/canonical-keysort-json/vX.Y.Z/vectors/canonical-form.json`
  at a tag. The alternative, `tests/vectors/`, reads more conventionally for
  Rust but hides a cross-language artifact behind a Rust convention.
- **Shipping in the crate.** Proposed yes, because `include_str!` in a test
  would otherwise break `cargo test` on the unpacked crate. The cost is a few
  kilobytes in the tarball.

## Verification

```verify:cli
cargo test --locked conformance_vectors
cargo package --locked --list | grep -qx vectors/canonical-form.json
```
