# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A single-crate Rust library with a two-function public surface
(`canonicalize_value`, `to_canonical_string`) that recursively lex-sorts JSON
object keys at the serialization boundary. It is the leaf dependency of
`attest-ledger` (record hashing) and `action-gate` (decision serialization),
so its output bytes are the substrate every downstream record hash is computed
over.

Crate name is `canonical-keysort-json`, import path is
`canonical_keysort_json`. The plain `canonical-json` name is taken on crates.io
by Mozilla's `canonicaljson-rs`, which implements the stricter gibson042
Canonical JSON spec and is byte-incompatible with this key-sort. Do not
describe this crate as implementing "Canonical JSON" in the gibson042 sense: it
sorts keys and nothing else.

## Commands

```sh
cargo test                      # all tests (they live inline in src/lib.rs)
cargo test byte_stable          # single test by substring, e.g. the determinism gate
cargo test -- --nocapture       # with stdout

cargo fmt --all --check         # CI gate
cargo clippy --all-targets --locked -- -D warnings   # CI gate, warnings are errors
cargo build --locked
```

The governance gate runs through `Makefile`, which is the single definition
both a local session and `.github/workflows/govern.yml` use:

```sh
make gate                       # read-only: the whole governed loop, in order
make refresh                    # writing: recompute the committed shard trees
make verify SPEC=001            # one spec's declared acceptance
make fmt clippy build test      # the cargo stack gate through the same file
```

The toolchain is pinned in `rust-toolchain.toml` (1.92.0 with rustfmt and
clippy). CI runs every command with `--locked`, so `Cargo.lock` must be
committed alongside any dependency change.

## Invariants that constrain changes

- **Byte stability is the contract.** Two logically-equal `Value`s built with
  different key insertion order must serialize to identical bytes. The guard is
  `to_canonical_string_is_byte_stable_across_key_insertion_order` in
  `src/lib.rs`. Any change that alters output bytes for existing inputs is a
  breaking change for every downstream hash already written to a ledger, even
  if it is "more canonical".
- **Arrays are deliberately not sorted.** `canonicalize_value` recurses into
  array elements but preserves element order; array-ordering invariants belong
  to the caller. This is asserted by
  `handles_deeply_nested_arrays_and_objects`.
- **Hashing stays out.** No hash function, no signing, no domain logic. Hashing
  is owned by `attest-ledger`. Keeping this crate domain-neutral is also what
  justifies the Apache-2.0 relicense of source extracted from the AGPL Open
  Agentic Platform (see `NOTICE`).
- **No unsafe.** The root package is a workspace-of-one purely so
  `[workspace.lints.rust] unsafe_code = "forbid"` applies (and so spec-spine
  indexes the repo as a Cargo workspace). Do not collapse that workspace table.

## Release

Tag-gated. `.github/workflows/release.yml` fires on `v*` tags, refuses to
publish if the tag does not match `version` in `Cargo.toml`, then runs an
idempotent `cargo publish --locked` (an "already uploaded" error is treated as
success). A release therefore needs, in one commit: the bumped `Cargo.toml`
version, a matching `CHANGELOG.md` entry with its link reference, then the tag.

`Cargo.toml` carries an explicit `include` list so repo governance
(`specs/`, `.github/`, `spec-spine.toml`) stays out of the published tarball.
A new file that must ship to crates.io has to be added there.

npm and PyPI channels are intentionally unused: this is a pure library with no
binary. `.env` holds local publishing tokens and is gitignored; CI reads its
own secrets.

## Spec governance

`specs/` holds this repo's own spec corpus, and `Cargo.toml` points at the
active spec via `[package.metadata.canonical-keysort-json].spec`. Specs use
frontmatter with `establishes` / `references` unit lists; see
`specs/000-canonical-keysort-json-bootstrap/spec.md` for the shape.

Self-governance is live. Spec 000 governs what the crate emits; spec 001
governs the harness that keeps 000 honest. The compiled shard trees under
`.derived/` are committed, and the gate compares them against the corpus:
`make gate` is the read-only chain, `make refresh` the writing half. Both the
`.gitattributes` merge driver and the `.derived/**/build-meta.json` gitignore
entry are active, not forward-looking.

Governance runs on the published `spec-spine` binary installed on `PATH`.
There is no `package.json` here and no `npx` invocation. `spec-spine.toml`
sets `[meta] required_version = ">=0.18.0"`, so a binary too old for the verbs
the harness calls is refused at the call with a config error rather than
answering with a misleading exit code. Install or upgrade it with `/setup`.

Read compiled artifacts only through `spec-spine` subcommands, never with
`jq`, `grep`, `python`, `awk` or `sed` over the shard JSON
(`.claude/rules/governed-artifact-reads.md`).

`AGENTS.md` is the cross-agent protocol and the authority for the gate command
list; `.claude/` carries the session harness (ten skills, four agents, four
rules) and is a byte-identical copy of the spec-spine kit, so a kit update is
a copy rather than a merge. All of it is claimed by spec 001, which means
editing `Makefile`, `AGENTS.md`, `CLAUDE.md`, `.claude/**`, `.githooks/**` or
`.github/workflows/govern.yml` requires editing spec 001 in the same change or
`spec-spine couple` refuses the pull request.

Never resolve a coupling failure by rewriting a spec to match code already
written. Surface the contradiction instead; a `Spec-Drift-Waiver:` is a human
instrument and an agent never writes one on its own authority
(`.claude/rules/adversarial-prompt-refusal.md`).
