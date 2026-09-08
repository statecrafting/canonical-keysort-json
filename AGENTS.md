# AGENTS.md

The cross-agent authority for this repository, read by Claude Code,
Codex CLI, Cursor, Copilot and any other agent via the AAIF/Linux
Foundation `AGENTS.md` standard. Edit this file to evolve the protocol;
the skills defer to it rather than restating it.

## New Sessions

Run these reads before doing any work. Nothing here mutates the tree,
so there is no required ordering.

- `spec-spine compile --check`: is the committed spec registry what the
  corpus compiles to? `0` fresh, `2` stale (report it and continue;
  repairing the tree is later, committed work, not a side effect of
  reading it), `1` the corpus fails validation, which is the first
  task of the session rather than an aside.
- `spec-spine index check`: the same question for the codebase index.
- `spec-spine registry status-report --nonzero-only`: lifecycle counts.
- `spec-spine registry plan`: what can be worked on now, and what blocks
  the rest.
- `spec-spine index coverage`: which source files no spec claims.
- `git log --oneline -10`: recent history.

Do **not** substitute a writing `compile` or `index` for the checks. A
read that repairs the tree hides the fact that the *committed* copy was
stale, so the drift then reads as an uncommitted local edit rather than
as a defect on the branch.

**Ask `spec-spine --version` before believing any exit code.** Every
binary ever released answers it, and it exits 0. If the version predates
the flag you are about to pass, upgrade; do not interpret the exit code
of a flag the binary does not have. Where `[meta] required_version` is
set in `spec-spine.toml`, the CLI checks this on every run and the
manual step is unnecessary.

## Working the backlog

One spec per pull request, then stop.

1. **Pick the spec.** `spec-spine registry plan --next` names it.
2. **Branch.** A feature branch named after the spec id. Never commit to
   the default branch.
3. **Re-read the design before coding.** If the design is imprecise,
   record the choice in the spec. If it is wrong, stop and report;
   never rewrite an approved spec to match code you just wrote.
4. **Implement within the territory.** Claim every new file in the
   spec's ownership edges. Touching a unit another spec owns is an
   `extends` edge naming that spec and unit; that amends nobody.
   Never edit `.derived/` by hand.
5. **Run the gate before every commit** (below), and commit the
   regenerated shards with the code they describe.
6. **Verify, then ship.** `spec-spine verify <id>` runs the spec's
   declared acceptance. A `Spec-Drift-Waiver:` line needs explicit
   human approval and is cited in the pull request body; an agent
   never writes one on its own authority.

## The gate

This list is the definition. Every skill that says "the gate as
`AGENTS.md` lists it" means exactly this, in this order:

```sh
spec-spine compile
spec-spine index
spec-spine lint --fail-on-warn
spec-spine index check --fail-on-unresolved
spec-spine index coverage --fail-on-untraced
spec-spine couple --base origin/main --head HEAD
```

In CI, `compile --check` replaces `compile` and the writing `index` is
dropped: a gate must never repair the tree it is judging. `make gate`
runs exactly that read-only form if you installed the kit's `Makefile`.

The stack gate runs alongside it, and CI runs both:

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo build --locked
cargo test --locked
```

`make refresh` is the writing half; `make gate` is the read-only form;
`make fmt clippy build test` runs the stack gate through the same file.

## Project layer

The corpus lives in `specs/`, the derived shard trees in `.derived/`,
and both are committed. Never edit `.derived/` by hand: it is compiler
output, regenerated with `make refresh` and committed with the change
that staled it.

`src/lib.rs` is the whole crate. It is listed in `[index]
extra_hashed_inputs`, so any edit to it stales every shard until the
corpus is recompiled, which is deliberate: spec 000 section 4 is the
normative cross-language canonical form and the code implementing it
must not drift from the corpus stating it silently.

**Changing what `to_canonical_string` emits is a breaking change even
when the new output is more correct.** Downstream record hashes in
`attest-ledger` and `action-gate` were computed over the old bytes.
Byte-affecting changes need spec 000 amended first, not a patch.

Ratification is a human act. An agent never advances a spec's `status`
from `draft` to `approved`, including a spec it just wrote. Spec 000 is
`draft` today and its approval is the maintainer's to give.

Spec 001 governs this harness. Editing `Makefile`, `AGENTS.md`,
`CLAUDE.md`, `.claude/**`, `.githooks/**` or `.github/workflows/govern.yml`
means editing spec 001 in the same change, or `couple` refuses it.
