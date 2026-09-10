---
id: "001-governed-harness"
title: "The governed harness (spec-spine self-governance and the session kit)"
status: approved
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
  - { kind: file, path: "spec-spine.toml" }
  - { kind: file, path: ".gitattributes" }
  - { kind: file, path: ".claude/settings.json" }
  - { kind: directory, path: ".claude/skills/" }
  - { kind: directory, path: ".claude/agents/" }
  - { kind: directory, path: ".claude/rules/" }
  - { kind: directory, path: ".githooks/" }
  - { kind: directory, path: "standards/spec/" }
  - { kind: file, path: ".github/workflows/govern.yml" }
  - { kind: file, path: ".github/workflows/ci.yml" }
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
spec-spine check --fail-on-unresolved --fail-on-warn
spec-spine lint --fail-on-warn
spec-spine index coverage --fail-on-untraced
spec-spine couple --base <base> --head HEAD
```

`check` (spec-spine 0.18.0, spec 075) is the composed freshness read: it asks
the question `compile --check` and `index check` asked separately, over both
committed trees, and returns the more severe of the two verdicts in the order
`3`, `1`, `2`, `0`. It is additive over the two primitives, which keep their
flags and their contracts, so the chain checks exactly what it checked before
and gains one thing: `--fail-on-warn` reaches the compile half, which is only
callable from the chain through this verb, and a warning-tier violation now
refuses rather than merely counting (spec 077). `spec-spine.toml` sets
`[meta] required_version = ">=0.18.0"`, so a binary that predates the verb is
refused at the call with a config error rather than answering with an exit code
this chain would misread.

`Makefile`'s `gate` target is the single definition of that chain, and
`.github/workflows/govern.yml` runs the same target rather than restating it, so
the local loop and the CI loop cannot drift. `make refresh` is the writing half,
for a session that has edited a spec and can commit the regenerated shards with
the change that made them stale.

`.claude/` carries the session harness: the skills that drive the governed loop,
the agents, and the rules that constrain what an agent may do. `AGENTS.md` is
the cross-agent protocol; `CLAUDE.md` is the repository guide. `.githooks/`
carries the opt-in merge driver for the committed shard trees, registered on the
shard globs by `.gitattributes`. `spec-spine.toml` configures the compiler this
chain runs, including the `[meta]` binary floor section 2 depends on, and
`.github/workflows/ci.yml` runs the stack half of the gate that `govern.yml`
deliberately does not duplicate.

`standards/spec/` is claimed too: the constitution and the contract are the
normative baseline every spec in this corpus defers to, and the templates are
what `/spec` copies from. They arrive from `spec-spine init` rather than being
authored here, which is the argument for leaving them alone, and it loses to
the simpler one: a document the whole corpus defers to should not be editable
without the gate noticing.

Every one of those paths is claimed, not merely referenced. The list in
`[index] extra_hashed_inputs` already says an edit to any of them should stale
the ledger; ownership is the other half of that statement, and without it the
coupling gate has nothing to refuse. Section 6 records what this cost.

## 3. Behavior

**The gate never writes.** `check` compiles in memory and compares against the
committed shards without writing, so a stale tree is reported rather than
repaired; it carries the same never-writes contract as the two primitives it
composes. A gate that writes repairs what it exists to judge, and if it
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
`check` on the merge commit, which runs whether or not the driver is
registered.

## 4. Out of scope

Cross-triple determinism of the shard trees, which spec-spine proves for its own
corpus over four target triples. The equivalent gate here would prove a
different and more valuable claim, the byte-stability of the crate's canonical
output across platforms, and it is a separate spec.

Release and publication, which `.github/workflows/release.yml` owns under spec
000. Ratification of spec 000 itself, which is a human act and not something
this harness performs.

## 5. Verification

The harness's acceptance is that the gate it installs actually runs and passes,
and that the three surfaces which must state the same chain still do. The first
command is the chain itself; the rest are the structural claims sections 2 and 3
make, each one a thing that would silently rot if the kit were updated by hand.

```verify:cli
make gate
test "$(ls .claude/skills | wc -l | tr -d ' ')" = 10
test -f .claude/settings.json
test -f .mcp.json
test -f .githooks/merge-derived-index.sh
grep -q 'check --fail-on-unresolved --fail-on-warn' Makefile
grep -q 'index coverage --fail-on-untraced' Makefile
grep -q 'couple --base' Makefile
grep -q 'spec-spine check --fail-on-unresolved --fail-on-warn' .github/workflows/govern.yml
spec-spine registry show 001-governed-harness --json | jq -e '[.establishes[].path] | index("spec-spine.toml")'
spec-spine registry show 001-governed-harness --json | jq -e '[.establishes[].path] | index(".claude/rules/")'
spec-spine registry show 001-governed-harness --json | jq -e '[.establishes[].path] | index(".github/workflows/ci.yml")'
spec-spine registry show 000-canonical-keysort-json-bootstrap --json | jq -e '[.establishes[].path] | index(".github/workflows/release.yml")'
spec-spine registry show 001-governed-harness --json | jq -e '[.establishes[].path] | index("standards/spec/")'
```

`make gate` covers freshness of both committed trees, the conformance lint,
ownership coverage and the coupling gate, so a failure in any of them fails
this spec's acceptance. The `grep` assertions are deliberately narrow: they
pin the verb, not the whole line, because `Makefile` reaches it through
`$(SPEC_SPINE)` while `govern.yml` names the binary outright.

The four `registry show` assertions hold the ownership this spec claims. They
read through the CLI and pipe its `--json` answer to `jq`, which
`.claude/rules/governed-artifact-reads.md` allows explicitly: the shard files
are never parsed, only the tool's typed reply. Without them, a future edit could
drop a unit from `establishes` and every other check here would still pass.

## 6. Resolved decisions

**2026-09-09: `standards/spec/` joins the claim.** The sweep that closed the
ownership gaps left one file class hashed but unowned, and an edit to
`standards/spec/contract.md` still passed `couple` with exit 0. These files come
from `spec-spine init` rather than being authored here, and a future refresh of
the constitution or the contract now needs a spec 001 edit in the same change.
That cost is accepted: the constitution and the contract are the baseline every
spec in this corpus defers to, and `/spec` reads the template on every new spec,
so a silent edit to any of them changes what the corpus means. A directory unit
covers the templates too, which is deliberate rather than incidental.

With this, every path in `[index] extra_hashed_inputs` is owned by exactly one
spec, which is the property that makes the coupling gate total rather than
partial.

**2026-09-09: the harness claims every file it is judged by.** An audit on the
merge of the 0.18.0 upgrade found that `couple` passed a commit editing
`.github/workflows/ci.yml` and `.claude/rules/orchestrator-rules.md` with no
spec touched at all: it reported "1 path(s) checked" and exited 0. Both files
sit in `[index] extra_hashed_inputs`, so an edit staled the ledger, but nothing
owned them and the coupling gate therefore had nothing to refuse. The rules file
is the one that constrains what an agent may do here, which makes it the worst
possible thing to leave ungoverned.

Four units joined `establishes` (`spec-spine.toml`, `.gitattributes`,
`.claude/rules/`, `.github/workflows/ci.yml`), and `spec-spine.toml` and
`.gitattributes` were promoted out of `references`, where `role: context` had
recorded that the spec knew about them without holding them. Spec 000 took
`.github/workflows/release.yml`, which section 4 of this spec had already
assigned to it in prose.

Two costs are accepted deliberately:

- **`.gitattributes` is over-claimed.** Most of that file is line-ending
  normalization that has nothing to do with this spec, so a future `*.rs text
  eol=lf` line will now require editing spec 001. The alternative was a
  `{ kind: section, ... }` unit, which names a Makefile target, a Markdown
  heading or a mapping keypath; a flat glob list is none of those, so it would
  not resolve. A rarely-edited file held whole beats a merge-driver stanza that
  can be deleted silently. Claiming it also forced a second edit: `lint` refused
  the bare claim as `L-008`, because a `file` unit carries no span and the
  contents therefore entered no content hash, so `.gitattributes` joined
  `[index] extra_hashed_inputs`. That is section 3's `src/lib.rs` argument
  arriving a second time, on a file section 3 did not anticipate.
- **The upstream corpus does not do this.** spec-spine references its own
  `spec-spine.toml` and `ci.yml` as `exemplar` and its rules as `context`
  without establishing any of them, so this repository is deliberately stricter
  than the tool it adopts. With two specs and one crate there is no cost to
  strictness, and the whole reason spec 001 exists is that a contract nothing
  checks is prose.

**2026-09-09: the harness tracks the spec-spine kit at 0.18.0.** The session
kit under `.claude/` was refreshed from the kit the 0.18.0 release ships, and
the gate chain in section 2 was restated onto `spec-spine check` with it. Three
consequences are worth recording, because none is recoverable from the diff
alone:

- **The chain is not weaker.** `check` composes the two reads it replaces; the
  gain is `--fail-on-warn` on the compile half, which no chain could reach
  before. Both flags the kit ships commented out are enabled here, because this
  corpus passes them today: `index coverage` reports 3/3 source files claimed
  and the compile emits no warnings.
- **The skill set shrank from fifteen to ten.** `init` became `/prime`, and
  `cleanup`, `implement-plan`, `refactor-claude-md`, `research` and
  `validate-and-fix` were dropped upstream (spec 081) because nothing in the
  loop referenced them. The ten that remain are byte-identical to the kit, so a
  future kit update stays a copy rather than a merge.
- **`govern.yml` keeps one deliberate deviation from the kit.** The kit's
  `probe` and `build` jobs are omitted, because `.github/workflows/ci.yml`
  already runs the cargo stack gate with `--locked` on the same triggers and
  keeping both would run every cargo command twice per pull request. The
  workflow comment carries the reason and the `hashFiles` finding the probe job
  exists to hold, so neither is rediscovered.
