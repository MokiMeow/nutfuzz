# AGENTS.md — how this repo is built

The working agreement for this repository: anyone contributing to nutfuzz should
read it fully before making changes. If anything here conflicts with a note
elsewhere, **this file wins.**

---

## 1. How the work is organised

- **Planning** — defines milestones and their Definitions of Done, reviews
  diffs, keeps the claims honest.
- **Implementation** — proceed one milestone at a time against
  `docs/milestones/`, keeping the build clean and every test green.

The loop: **pick the lowest-numbered unfinished milestone → implement → build
clean → prove it with tests that try to break it → measure → tick the Definition
of Done → update docs/CHANGELOG → commit → next.**

## 2. Ground rules (non-negotiable)

1. **Determinism is the contract.** Every random decision comes from the seeded
   `Rng`. A campaign must replay exactly from `--seed`. Never call a system
   RNG, never depend on hash iteration order or wall-clock time in the search
   path. A fuzzer that cannot reproduce its findings is useless.
2. **Prove the absence of false positives.** Every milestone must keep the
   `reports_nothing_when_there_is_nothing_to_find` test passing against a fixed
   target. A fuzzer that always reports a crash proves nothing.
3. **A finding must come with a minimised reproducer.** Raw crashing inputs are
   not bug reports.
4. **No dependencies.** `std` only — the PRNG, mutations, scheduler, coverage
   map and minimiser are the project
   ([ADR 0001](docs/decisions/0001-no-dependencies.md)).
5. **The build must never break.** `cargo build --release` and `cargo test` are
   green at every commit, with zero warnings.
6. **Measure, don't assert.** Claims about a strategy finding bugs faster must
   come with executions-to-first-crash numbers over several seeds.
7. **No feature without a doc.** Update the matching `docs/` page in the same
   milestone.

## 3. Build, run, verify

| Command | What it does |
|---------|--------------|
| `cargo build --release` | Build (zero warnings expected). |
| `cargo test` | Unit + bug-finding + false-positive tests. |
| `cargo run --release` | Fuzz the buggy target and minimise the crash. |
| `cargo run --release -- --target parse_record_fixed` | Must find nothing. |
| `cargo run --release -- --seed N --iterations M` | A reproducible campaign. |

**Definition of "it works":** builds warning-free, `cargo test` passes, the
buggy target still yields a minimised reproducer, and the fixed target still
yields nothing.

## 4. Coding standards

- Rust 2021, `rustfmt` defaults. `Result` over panics in library code; no
  `unwrap()` outside tests and `main`.
- Mutations take `&mut Vec<u8>` and must be **total**: never panic on empty,
  one-byte, or maximum-length input. Every new mutation gets a test for each of
  those cases.
- Respect `MAX_LEN`; unbounded growth turns a fuzzer into a memory-exhaustion
  bug of its own.
- Module-level `//!` comments explain *why* a strategy exists and what class of
  bug it targets.

## 5. Commit and branch style

- `type(scope): outcome`, imperative, lower case.
  Examples: `feat(coverage): record edge hits in a shared bitmap`,
  `fix(mutate): stop splice from exceeding the length cap`.
- Types: `feat`, `fix`, `docs`, `refactor`, `build`, `chore`, `test`, `perf`.
- **No AI/co-author trailers.**
- Branch per milestone (`milestone-1-coverage`), PR into `main`, CI green.

## 6. The milestone path

Specs with Definitions of Done live in `docs/milestones/`.

| # | Milestone | Adds | Spec |
|---|-----------|------|------|
| 0 | Mutation core | seeded RNG, mutations, fuzz loop, minimiser | [spec](docs/milestones/milestone-0-mutation-core.md) ✅ |
| 1 | Coverage | instrumentation, edge bitmap, feedback loop | [spec](docs/milestones/milestone-1-coverage.md) |
| 2 | Scheduling | corpus energy, favoured entries, crash dedup | [spec](docs/milestones/milestone-2-scheduling.md) |
| 3 | External targets | run real binaries, fork server, timeouts | [spec](docs/milestones/milestone-3-external-targets.md) |
| 4 | Smarter input | dictionaries, structure-aware mutation | [spec](docs/milestones/milestone-4-structure.md) |
| 5 | A real bug | fuzz a real library, triage, disclose | [spec](docs/milestones/milestone-5-real-target.md) |
| 6 | Polish | benchmarks vs AFL++, CI, `v1.0.0` | [spec](docs/milestones/milestone-6-polish.md) |

**Definition of Done (whole project):** nutfuzz uses coverage feedback to find
bugs a blind mutation fuzzer cannot, demonstrates the difference with measured
executions-to-first-crash, and has found at least one genuine bug in real
third-party code, responsibly disclosed.

## 7. What NOT to do

- Do not introduce non-determinism into the search path.
- Do not weaken or delete the false-positive test to make something pass.
- Do not add a dependency (no `rand`, no `libfuzzer-sys`, no `arbitrary`).
- Do not claim a strategy is better without executions-to-first-crash data.
- Do not commit a weaponised exploit, or any reproducer for an unfixed
  third-party bug, to this repository — see §8.

## 8. Scope and ethics

This is a bug-finding tool for code you own or are authorised to test. Real
findings follow **coordinated disclosure**: report privately, agree a timeline,
publish after a fix. Store third-party reproducers outside the repo until the
issue is public. The contribution here is the search technique.

## 9. Tools reference

- **cargo** — build, test, bench.
- **`cargo test -- --nocapture`** — see fuzzer output while debugging.
- **AFL++ / libFuzzer docs** — the reference designs for coverage feedback and
  corpus scheduling; cite them when implementing an equivalent idea.
- **`perf stat`** — when execution throughput drops, find out why.

Build one milestone, prove it finds bugs *and* stays quiet on clean code,
document it, commit. Then the next.
