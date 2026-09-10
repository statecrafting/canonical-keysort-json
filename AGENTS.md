# AGENTS.md: canonical-keysort-json

This file is the cross-agent session-init protocol authority, read by Claude
Code, Codex CLI, Cursor, and GitHub Copilot via the AAIF/Linux Foundation
AGENTS.md standard. It is the single source for the init protocol: tooling that
runs `/prime` reads the `## New Sessions` section to derive its plan.

Governance is provided by `spec-spine`, installed on your `PATH`. There is no
`package.json` here and no `npx` invocation. `spec-spine.toml` sets
`[meta] required_version = ">=0.18.0"`, so the CLI refuses a binary too old for
the verbs below rather than answering them with a misleading exit code.
Bootstrap spec: `specs/000-canonical-keysort-json-bootstrap/spec.md`.

## New Sessions

Run `/prime` as the first action of every new session. It reads this section to
derive its execution plan dynamically: any item added here is automatically
picked up on the next init.

> AGENTS.md is loaded implicitly as the protocol source; its contents are the
> protocol, so `/prime` does not list AGENTS.md as a parallel identity read in
> Step 1 (avoiding the self-reference loop).

**Init protocol:**

0. **Load rules** (read first): `.claude/rules/orchestrator-rules.md`,
   `.claude/rules/governed-artifact-reads.md`, and
   `.claude/rules/adversarial-prompt-refusal.md`.

1. **Parallel reads.** Dispatch the following simultaneously (nothing here
   mutates the working tree, so there is no required ordering):
   - `CLAUDE.md`: project overview, governance model, conventions
   - `README.md`: full project description
   - `standards/spec/contract.md`: the short normative spec-spine contract
   - `standards/spec/constitution.md`: durable constitutional baseline
   - `spec-spine --version`: the binary's version. The `[meta]` pin makes the
     CLI check this itself on every run, so a version too old fails loudly at
     the call rather than silently mis-answering it.
   - `spec-spine check`: the freshness read for **both** committed trees, the
     spec registry and the codebase index (spec 075; non-fatal, see
     **Freshness** below)
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts
   - `spec-spine registry plan`: the ready set (spec 038): which specs can be
     worked on now and what blocks the rest
   - `spec-spine index coverage`: which source files no spec specifically
     claims (spec 032; non-fatal, exit 2 if the index is stale)
   - `spec-spine registry list --ids-only`: spec inventory (for latest-spec
     detection)
   - `ls src/`: the crate surface (one file, `src/lib.rs`)
   - `git log --oneline -10`: recent history
   - `git diff --stat HEAD~1`: last change summary

   There is no `docs/` directory in this repository; the prose that would live
   there is in `README.md`, `CLAUDE.md`, and the corpus itself.

2. **Emit** an `## primed: canonical-keysort-json` summary block (layer
   overview, recent activity, ready-to-help line), with a `## lifecycle:`
   sub-section populated from the `status-report` output.

**Read discipline:** the init protocol MUST NOT parse `.derived/**/*.json`
directly (no `python`, `jq`, `awk`, `sed` against compiled artifacts). All
structural and lifecycle data comes from `spec-spine` subcommands.

**Freshness:** this repository commits its derived artifacts, so
`spec-spine check` (spec 075) asks about both committed trees in one call. It
compiles in memory and compares against the committed shards **without
writing**, reports each tree separately, and returns the more severe of the two
verdicts in this order: **`3` then `1` then `2` then `0`**. It is non-fatal to
`/prime`: report it in the summary and continue.

- **`0` (both fresh):** the committed shards are exactly what the corpus
  compiles to, so the lifecycle counts reflect the current `specs/*/spec.md`
  frontmatter. Report nothing.
- **`2` (stale):** say which tree the output named, name the drifted shards
  from stderr, report "run `spec-spine compile` and commit" or "run
  `spec-spine index`" accordingly, and continue. The lifecycle counts come from
  the committed ledger and are therefore the stale ones; say so rather than
  presenting them as current.
- **`1` (validation failed, or unresolved units refused):** with
  `--fail-on-unresolved` this code also covers a refused unresolved-unit
  diagnostic, so read the report lines to tell the two apart. If the corpus
  fails validation, surface the violations and report the counts as unverified.
  This outranks `2`: staleness is not meaningful against a corpus that does not
  validate.
- **`3` (I/O, parse, schema, or config):** a read that could not be performed
  has not answered. Treat freshness as unknown for both trees, report stderr
  verbatim, and continue. Never report "fresh" for a code you did not
  recognize. Against the `[meta]` pin this is also the code a too-old binary
  produces, and the message says so.

The counts are formatted in step 2, after every parallel read has returned, so
the freshness verdict is always in hand before the lifecycle numbers are
written down. Do not emit counts earlier.

Do **not** substitute a plain `spec-spine compile` or `spec-spine index` here.
Writing repairs the tree as a side effect of reading it, which hides that the
*committed* copy was stale: the drift then reads as an uncommitted local edit
rather than as a defect already on the branch. `/prime` reports; it does not
silently mutate, and `spec-spine check` carries the same never-writes contract.

**CLI missing:** if `spec-spine --version` fails, run `/setup`. Do NOT fall back
to ad-hoc parsing of `.derived/**/*.json`.

If any file is missing: log "not found" and continue.

## Working the backlog

The governed loop is one spec per session, start to finish, then stop. It is
what `spec-spine registry plan`, the in-flight leniency (specs 025, 041, 044)
and the ownership ratchet (spec 032) exist to serve. Record specs (the
bootstrap spec, a thesis, a harness spec at `n-a` or `complete`) are never
work orders.

1. **Pick the spec.** `spec-spine registry plan` prints the ready set in
   dependency order; `/next` applies the two rules on top of it and names the
   pick. Take the first entry unless a human named another. Never guess and
   never pick a `draft`: approval is a human act (`plan` will offer a draft
   whose dependencies are met; `/next` will not). If the spec's Territory names
   an operator prerequisite (a credential, a bucket, a cluster) that is
   missing, stop and report exactly what is needed instead of mocking around
   it.
2. **Branch and flip.** `/build <id>` sequences steps 2 to 6 with the exact
   commands. Work on a feature branch named after the spec id. Flip the spec to
   `implementation: in-progress`, run `spec-spine compile` and
   `spec-spine index`, and commit the flip with the regenerated derived shards
   before writing code. Never commit to `main`.
3. **Re-read the spec in full before coding.** The design precedes the code.
   If the design is imprecise, record the choice you make as a dated decision
   entry in the spec. If the design is *wrong*, stop and report the
   contradiction: never edit a spec afterwards to ratify what the code
   happened to do (`.claude/rules/adversarial-prompt-refusal.md`).
4. **Implement within the territory.** Every file you add must be claimed by
   the spec you are implementing, in the same change (`C-002` refuses an
   unclaimed source file). Touching a unit another spec owns requires an
   `extends` edge on that spec's unit, declared in your spec's frontmatter;
   that amends nobody. Never edit the derived directory by hand.
5. **Run the gate before every commit.** The governance floor, in this order
   (`compile` and `index` write; the checks follow):

   ```sh
   spec-spine compile
   spec-spine index
   spec-spine lint --fail-on-warn
   spec-spine check --fail-on-unresolved --fail-on-warn
   spec-spine index coverage --fail-on-untraced
   spec-spine couple --base "$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || echo origin/main)" --head HEAD
   ```

   `make refresh` is the writing pair, `make gate` is the four reads that
   follow it, and `.github/workflows/govern.yml` runs the same `make gate`
   target rather than restating it, so the local loop and the CI loop cannot
   drift. Both optional lines the kit ships commented out are enabled here and
   enforced by CI: this corpus claims every source file it builds
   (`index coverage` reports 3/3) and compiles without warnings.

   The base ref is resolved from the repository rather than assumed to be
   `origin/main` (spec 072). Set `$SPEC_SPINE_DEFAULT_BRANCH` to override the
   branch the push gate protects and `Makefile` compares against.

   Then the stack gate, which CI runs in `.github/workflows/ci.yml`:

   ```sh
   cargo fmt --all --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo build --locked
   cargo test --locked
   ```

   `make fmt clippy build test` runs that stack gate through the same file.
   All of it must exit 0. Commit the regenerated shards with the code they
   describe.
6. **Satisfy the spec's acceptance criteria verbatim.** `/verify <id>` runs the
   spec's `## Verification` block the way the post-merge verify stage will. If
   a criterion cannot be satisfied (external state, a missing sibling), keep
   `implementation: in-progress`, add a dated note to the spec saying exactly
   what remains, and report it. Flip to `implementation: complete` only when
   acceptance holds; recompile and commit. The gate then holds the spec to
   every unit it claims (spec 041).
7. **Ship.** `/ship`: gate, review, a conventional commit naming the spec id
   (`feat(001): ...`), push the feature branch, open the PR. A
   `Spec-Drift-Waiver:` line needs explicit human approval; a driven session
   never self-approves one. `/shepherd` then watches the checks, remediates
   through the gate, merges, and confirms the merge on disk. Then stop: the
   next session takes the next spec.

## Available Agents

Agents live in `.claude/agents/`. Four pipeline agents handle the
plan/explore/implement/review cycle:

- `architect`: plans and decomposes tasks, validates approaches against specs. Read-only.
- `explorer`: searches the codebase, traces dependencies, gathers context. Read-only.
- `implementer`: executes focused changes from an existing plan. Minimal diffs.
- `reviewer`: post-change review for bugs, correctness, performance, spec compliance. Read-only.

## Available Commands

Skills live in `.claude/skills/`:

The governed loop, in the order "Working the backlog" runs it:

- `/prime`: prime a session (this protocol).
- `/setup`: one-time contributor setup; installs the pinned spec-spine and verifies the governed loop.
- `/next`: name the next work order from `registry plan`, minus drafts, with in-flight specs and blockers reported. Read-only.
- `/build <id>`: implement one spec start to finish: preflight, branch, flip, implement, gate, verify, flip complete.
- `/verify <id>`: run the spec's `## Verification` block locally through `spec-spine verify <id>`.
- `/ship`: run the gate, review, commit on the feature branch, open the PR.
- `/shepherd`: watch the PR's checks by head sha, remediate through the gate, merge, confirm on disk.
- `/spec`: author a new spec at the next free ordinal, born `draft`; approval stays a human flip.

The skills the loop calls:

- `/commit`: create a git commit with an impact-focused conventional message, spec ordinal as scope.
- `/code-review`: review the working diff for correctness bugs, spec drift, and illegitimate mid-build spec edits.

Every skill is repository-invariant and byte-identical to the spec-spine kit:
the project layer (the binary invocation, the version pin, the gate command
list, the stack gate, the never-touch artefacts) lives in this file and in the
path-scoped rules, and each skill says what it reads from where under
`## Project layer`. A kit update is therefore a copy, not a merge.

## Conventions

- Items added to the "New Sessions" init protocol are auto-loaded on the next init.
- Orchestrated workflows read compiled artifacts (`.derived/**`) through
  `spec-spine` subcommands, never via ad-hoc parsers (see
  `.claude/rules/governed-artifact-reads.md`).
- Every substantive change is bound to a spec; owned paths and their owning
  `spec.md` move together (`spec-spine couple` enforces this at PR time).

## Project layer

The corpus lives in `specs/`, the derived shard trees in `.derived/`, and both
are committed. Never edit `.derived/` by hand: it is compiler output,
regenerated with `make refresh` and committed with the change that staled it.
The one exception is `.derived/**/build-meta.json`, which carries a wall-clock
`builtAt` and is gitignored for that reason.

`src/lib.rs` is the whole crate. It is listed in `[index]
extra_hashed_inputs`, so any edit to it stales every shard until the corpus is
recompiled, which is deliberate: spec 000 section 4 is the normative
cross-language canonical form and the code implementing it must not drift from
the corpus stating it silently.

**Changing what `to_canonical_string` emits is a breaking change even when the
new output is more correct.** Downstream record hashes in `attest-ledger` and
`action-gate` were computed over the old bytes. Byte-affecting changes need
spec 000 amended first, not a patch.

Ratification is a human act. An agent never advances a spec's `status` from
`draft` to `approved`, including a spec it just wrote. Specs 000 and 001 were
ratified by the maintainer on 2026-09-09 and are `approved`; a spec authored by
`/spec` is born `draft` and stays there until a human flips it.

Spec 001 governs this harness, and claims every file it is judged by: `Makefile`,
`AGENTS.md`, `CLAUDE.md`, `.mcp.json`, `spec-spine.toml`, `.gitattributes`,
`.claude/settings.json`, `.claude/skills/`, `.claude/agents/`, `.claude/rules/`,
`.githooks/`, `standards/spec/`, `.github/workflows/govern.yml` and
`.github/workflows/ci.yml`.
Editing any of them means editing spec 001 in the same change, or `couple`
refuses it. `.github/workflows/release.yml` belongs to spec 000, which owns
release and publication.

That list is not shorter than `[index] extra_hashed_inputs` by accident: every
path in that table is owned by exactly one spec, which is what makes the
coupling gate total rather than partial. A file that stales the ledger but no
spec claims is one the gate cannot refuse, which is how an edit to
`.claude/rules/` once passed untouched.

`.githooks/` carries the opt-in merge driver for the committed shard trees. It
does nothing until `./.githooks/enable-merge-driver.sh` registers it in your
clone, and it never replaces the staleness gate. Registration lives in
`.git/config`, which is not committed, so every clone runs it once; worktrees
inherit it from the clone they came from.
