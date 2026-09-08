---
id: "001-governed-harness"
title: "The governed harness (spec-spine self-governance and the session kit)"
status: draft
created: "2026-09-08"
authors: ["canonical-keysort-json"]
kind: tooling
implementation: complete
risk: medium
summary: >
  This repository's own governance surface: the spec-spine gate chain, the
  committed .derived shard trees it compares against, the merge driver that
  keeps those shards mergeable, and the Claude Code session kit that constrains
  what an agent may do here. Spec 000 governs what the crate emits; this spec
  governs what the repository does to keep 000 honest. It exists because 000
  section 4 became a normative cross-language contract while nothing
  mechanically checked that the corpus stating it stayed coherent with the code
  implementing it.
depends_on:
  - "000-canonical-keysort-json-bootstrap"
establishes:
  - { kind: file, path: "Makefile" }
  - { kind: file, path: "AGENTS.md" }
  - { kind: file, path: "CLAUDE.md" }
  - { kind: file, path: ".mcp.json" }
  - { kind: file, path: ".claude/settings.json" }
  - { kind: directory, path: ".claude/skills/" }
  - { kind: directory, path: ".claude/agents/" }
  - { kind: directory, path: ".githooks/" }
  - { kind: file, path: ".github/workflows/govern.yml" }
references:
  - { unit: { kind: file, path: "spec-spine.toml" }, role: context }
  - { unit: { kind: file, path: ".gitattributes" }, role: context }
---

# 001: The governed harness

## 1. Purpose

Spec 000 section 4 states this family's canonical form normatively, for ports in
other languages as much as for the crate here. A contract stated in a corpus
that nothing checks against its implementation is prose. This spec installs the
checking.

The trigger was concrete. A second implementation of 000 section 4 exists in
`claude-observatory`'s `journal.ts`, and it defined a different canonical form
for two classes of input for months without anything noticing, because the only
fixture both sides tested was one where they could not disagree. That defect was
invisible to everything except a second implementation. The corresponding risk
on this side is an edit to `src/lib.rs` that drifts from what spec 000 says,
with nothing to catch it.

## 2. Territory

The gate chain, in the order the chain requires, read-only throughout:

```
spec-spine compile --check
spec-spine lint --fail-on-warn
spec-spine index check --fail-on-unresolved
spec-spine index coverage --fail-on-untraced
spec-spine couple --base <base> --head HEAD
```

`Makefile`'s `gate` target is the single definition of that chain, and
`.github/workflows/govern.yml` runs the same target rather than restating it, so
the local loop and the CI loop cannot drift. `make refresh` is the writing half,
for a session that has edited a spec and can commit the regenerated shards with
the change that made them stale.

`.claude/` carries the session harness: the skills that drive the governed loop,
the agents, and the rules that constrain what an agent may do. `AGENTS.md` is
the cross-agent protocol; `CLAUDE.md` is the repository guide. `.githooks/`
carries the opt-in merge driver for the committed shard trees, registered on the
shard globs by `.gitattributes`.

## 3. Behavior

**The gate never writes.** `compile --check` compiles in memory and compares
against the committed shards without writing, so a stale tree is reported rather
than repaired. A gate that writes repairs what it exists to judge, and if it
were sequenced after a writing `compile` it would compare the committed shards
against files the same run had just overwritten and pass forever.

**`.derived/` is committed.** The shard trees under `.derived/spec-registry/`
and `.derived/codebase-index/` are the artifacts the staleness gate compares
against, which is only sane because compilation is deterministic.
`build-meta.json` is the one exception, carrying a wall-clock `builtAt`, and is
gitignored.

**The crate source is in the content hash.** `src/**/*.rs` is listed in
`[index] extra_hashed_inputs`, so an edit to `src/lib.rs` stales the index until
the corpus is recompiled and committed. Without it the claim spec 000 makes on
`src/lib.rs` is a bare file unit, which carries no span and therefore enters no
content hash, and the implementation of the normative section could change
freely while the ledger reported everything fresh. That is `L-008`, and this
repository does not take the allowlist exemption for it.

**The merge driver is opt-in per clone** and never replaces the staleness gate.
It resolves a textual conflict between two branches that both regenerated shards
by regenerating from the merged tree; what proves the result correct is
`index check` on the merge commit, which runs whether or not the driver is
registered.

## 4. Out of scope

Cross-triple determinism of the shard trees, which spec-spine proves for its own
corpus over four target triples. The equivalent gate here would prove a
different and more valuable claim, the byte-stability of the crate's canonical
output across platforms, and it is a separate spec.

Release and publication, which `.github/workflows/release.yml` owns under spec
000. Ratification of spec 000 itself, which is a human act and not something
this harness performs.
