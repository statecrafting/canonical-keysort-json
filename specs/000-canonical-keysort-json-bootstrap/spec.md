---
id: "000-canonical-keysort-json-bootstrap"
title: "canonical-keysort-json bootstrap (deterministic JSON key-ordering library)"
status: approved
created: "2026-07-12"
authors: ["canonical-keysort-json"]
kind: tooling
implementation: complete
risk: low
summary: >
  Bootstrap spec for the canonical-keysort-json repository: a single-crate library
  that recursively lexicographically sorts object keys at the JSON
  serialization boundary, so values serialize byte-identically regardless of
  serde_json's preserve_order feature. Extracted (relicensed Apache-2.0 by the
  sole copyright holder) from the Open Agentic Platform's crates/canonical-json.
  It is the leaf dependency of attest-ledger and action-gate. This spec
  establishes the crate skeleton and seeds this repo's own spec corpus, which
  is governed by the pinned spec-spine library.
depends_on: []
establishes:
  - { kind: file, path: "Cargo.toml" }
  - { kind: file, path: "src/lib.rs" }
  - { kind: file, path: ".github/workflows/release.yml" }
references:
  - { unit: { kind: file, path: "README.md" }, role: context }
---

# 000: canonical-keysort-json bootstrap

## 1. Purpose

canonical-keysort-json is the leaf of the `statecrafting` reusable-primitive family.
It guarantees one thing: a `serde_json::Value` canonicalizes to a
byte-identical string regardless of `serde_json`'s `preserve_order` feature
state anywhere in the dependency graph. That byte string is the substrate on
which `attest-ledger` computes record hashes and `action-gate` computes a
stable decision hash, so this crate underwrites the tamper-evidence of every
downstream ledger.

This bootstrap spec exists so the repository has a governed seed: the crate
compiles, this corpus is non-empty, and spec-spine can dogfood it. The public
surface is two functions, `canonicalize_value` and `to_canonical_string`,
established here.

## 2. Scope

In scope: recursive lexicographic sort of object keys; array and scalar
pass-through; a compact canonical-string convenience; byte-stability tests
(the determinism gate).

Out of scope: hashing (owned by `attest-ledger`); signing; any domain logic.
The crate holds no Open Agentic Platform semantics, which is what makes the
Apache-2.0 relicense of the extracted AGPL source a domain-neutral utility
vend (see `NOTICE`).

## 3. Provenance

Extracted from OAP `crates/canonical-json` (AGPL-3.0-or-later there),
relicensed Apache-2.0 by the sole copyright holder. The interim extraction
record is `chancery/docs/preliminary/00-extraction-overview.md` and
`05-canonical-json.md`; a forthcoming OAP extraction spec formalizes the vend.

## 4. The canonical form (normative)

Section 1 says this crate guarantees a byte-identical canonicalization. It did
not say over what, and `src/lib.rs` said only "lexicographically-sorted". That
gap was survivable while every consumer was Rust calling this crate. It stopped
being survivable when a second implementation appeared: `claude-observatory`'s
`journal.ts`, which hashes its journal records against this same contract and
defines a different canonical form for two classes of input.

So the contract is stated here, in the corpus, rather than only in a Rust doc
comment, which is precisely the surface a port in another language does not
read. `src/lib.rs` is the reference implementation of the rules below, not
their definition.

1. **Object keys order by Unicode code point, ascending**, at every nesting
   level. Equivalently, by UTF-8 byte order: for well-formed UTF-8 the two are
   the same total order. `String::cmp` gives this for free in Rust, which is
   why the reference implementation does not spell it out.
2. **Array element order is preserved and never sorted.** Any ordering
   invariant within an array belongs to the caller.
3. **The serialization is compact**: no insignificant whitespace, `:` and `,`
   unpadded.
4. **String escaping is the minimal JSON set**: `"` and `\`, plus C0 controls,
   using the short forms `\b \t \n \f \r` where they exist and `\u00XX`
   otherwise. Nothing else is escaped, U+2028 and U+2029 included. This has
   been verified byte-identical between `serde_json::to_string` and
   JavaScript's `JSON.stringify` across the full C0 range, DEL, quote,
   backslash, non-ASCII, U+2028, U+2029, and astral characters, so a port does
   not need its own escaper.
5. **Unpaired surrogates are not representable.** A Rust `String` cannot hold
   one, and `serde_json` refuses to parse `\uD800` in either key or value
   position. A producer in a language whose strings can hold one must reject it
   at the boundary rather than emit it: such a record is not merely ordered
   differently here, it is unparseable, and it breaks every subsequent link of
   a hash chain for this implementation.

Two consequences follow that a port gets wrong by default, and both are pinned
by tests in `src/lib.rs`:

- Rule 1 is **not** UTF-16 code unit order. A surrogate pair (0xD800..=0xDBFF)
  sorts below U+E000..=U+FFFF as code units, so JavaScript's default
  `Array.prototype.sort` places `"\u{1F600}"` before U+FFFD where this contract
  places it after.
- Rule 1 has **no carve-out for integer-like keys**: `"10"` precedes `"2"`. A
  port that sorts keys and then inserts them into a JavaScript object loses
  this unconditionally, because that language enumerates array-index-like keys
  first in ascending numeric order regardless of insertion order. Such a port
  must emit the serialized text directly instead of round-tripping through an
  object.

### Non-guarantees

This crate does not normalize number representation. It emits whatever
`serde_json` parsed, so `100.0`, `-0.0`, exponent forms, and integers beyond
2^53 are outside the set for which cross-language byte equality is guaranteed.
A consumer that needs that guarantee restricts its own value set; `journal.ts`
does exactly this, admitting integers only. This is also the reason RFC 8785
and Mozilla's `canonicaljson-rs` were not adopted (section 1 of the extraction
record): both normalize numbers and escape non-ASCII, which defines different
bytes than the chains that already exist.

## 5. Established units

- `Cargo.toml`: the single-crate manifest, Apache-2.0, edition 2024.
- `src/lib.rs`: `canonicalize_value` and `to_canonical_string` plus their
  tests, including the byte-stability determinism gate.
- `.github/workflows/release.yml`: the tag-gated publish. Spec 001 section 4
  puts release and publication out of the harness's scope and says this file
  belongs to spec 000; until 2026-09-09 that sentence was the only thing
  claiming it, and a claim only prose makes is one the coupling gate cannot
  enforce.

## 6. Verification

Section 4 is the normative canonical form, and the tests in `src/lib.rs` are
what hold the implementation to it. Byte stability across key insertion order
is the one that matters most to downstream ledgers, and it is asserted by
`to_canonical_string_is_byte_stable_across_key_insertion_order`.

```verify:cli
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo build --locked
cargo test --locked
```

Every command carries `--locked`, so a `Cargo.lock` that drifted from
`Cargo.toml` fails acceptance rather than being silently resolved.
