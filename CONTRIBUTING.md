# Contributing to nutfuzz

This is primarily a learning/portfolio project, but clean contributions are
welcome.

## Before you start

- Read [AGENTS.md](AGENTS.md) — the working agreement, which applies to
  everyone.
- Skim [docs/00-overview.md](docs/00-overview.md), the
  [roadmap](docs/04-roadmap.md), and
  [docs/05-mutation.md](docs/05-mutation.md).
- You need only a Rust toolchain; there are no dependencies.

## The two rules that matter most

1. **Determinism.** Every random decision draws from the seeded `Rng`, so
   `--seed N` replays a campaign exactly. Never introduce a system RNG,
   hash-order dependence, or wall-clock timing into the search path.
2. **No false positives.** The `reports_nothing_when_there_is_nothing_to_find`
   test must keep passing. Never weaken it to make something else go green — it
   is what makes every finding credible.

## Workflow

1. Pick the lowest-numbered unfinished milestone in
   [docs/milestones/](docs/milestones/), or an open issue.
2. Branch from `main`: `git checkout -b milestone-1-coverage`.
3. `cargo build --release` must be warning-free and `cargo test` must pass at
   every commit.
4. New mutations need tests for empty, one-byte, and max-length input —
   mutations must be **total** and never panic.
5. Performance claims need measured executions-to-first-crash across several
   seeds, not assertions.
6. Open a PR into `main`; CI must be green.

## Commit style

`type(scope): outcome` in the imperative, lower case. Types: `feat`, `fix`,
`docs`, `refactor`, `build`, `chore`, `test`, `perf`. No AI/co-author trailers.

Example: `feat(coverage): record edge hits in a shared bitmap`.

## Scope and ethics

nutfuzz is for code you own or are authorised to test. Findings in third-party
software follow **coordinated disclosure**: report privately, agree a timeline,
publish after a fix. **Do not open a PR containing a reproducer for an unfixed
third-party bug** — see
[ADR 0003](docs/decisions/0003-defensive-scope.md).

## Reporting issues

Include your seed, target, iteration count, and the output. Because campaigns
are deterministic, a seed is usually a complete reproduction.
