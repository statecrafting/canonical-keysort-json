---
id: "000-canonical-keysort-json-bootstrap"
title: "canonical-keysort-json bootstrap (deterministic JSON key-ordering library)"
status: draft
created: "2026-07-12"
authors: ["canonical-keysort-json"]
kind: tooling
implementation: pending
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
references:
  - { unit: { kind: file, path: "README.md" }, role: context }
---

# 000: canonical-keysort-json bootstrap

## 1. Purpose

canonical-keysort-json is the leaf of the `stagecraft-ing` reusable-primitive family.
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

## 4. Established units

- `Cargo.toml`: the single-crate manifest, Apache-2.0, edition 2024.
- `src/lib.rs`: `canonicalize_value` and `to_canonical_string` plus their
  tests, including the byte-stability determinism gate.
