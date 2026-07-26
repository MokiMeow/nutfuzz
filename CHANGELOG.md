# Changelog

All notable changes to nutfuzz are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-07-25

Coverage-guided fuzzing that provably finds bugs blind mutation cannot.

### Added
- Milestone 1: **edge coverage feedback** — an AFL-style bitmap indexed by
  `hash(block) ^ prev`, hit-count bucketing, and a corpus that grows whenever an
  input reaches a new edge.
- Milestone 2: **corpus scheduling** — per-entry energy weighted by size and
  usage, a greedy set-cover "favoured" subset that still covers every known
  edge, and **crash deduplication by coverage signature** so one bug reached ten
  ways is reported once.
- Milestone 3: **external targets** — run a real binary, classify a fatal signal
  (SIGSEGV/SIGABRT/SIGFPE/SIGILL/SIGBUS) as a crash, a timeout as a **hang**, and
  a non-zero exit as a program correctly rejecting input.
- Milestone 4: **dictionaries** — token insertion and overwrite, plus
  auto-learning, to open gates coverage feedback cannot.
- Milestone 6: a `--bench` mode comparing blind and guided across ten seeds,
  `--mode`/`--dict` flags, and CI gating on build, tests, and both the
  bug-finding and no-false-positive checks.
- 56 tests, including the measurement that guided beats blind and the check
  that a dictionary opens a gate coverage alone cannot.

### Fixed
- **Edge ids are now hashed before use.** Small sequential block ids collide
  badly under XOR (`22^10` and `23^11` both give 28), which silently merged
  distinct edges and made guided fuzzing perform exactly like blind. AFL uses
  random compile-time ids for this reason; nutfuzz uses Fibonacci hashing to get
  the same spread. A test now asserts each correct magic byte uncovers a new
  edge.

### Measured
- `parse_nested` (bug behind seven gates), 300 000 executions × 10 seeds:
  blind **0/10 found**; coverage-guided **8/10**, median **126 059** executions.
- An 85-byte crashing input minimises to the true 7-byte reproducer.
- 200 000 executions against corrected targets: **no false positives**.

### Known limitations
- **Coverage feedback cannot open a multi-byte magic comparison.** There is no
  partial credit for getting three of four bytes right, so a guided fuzzer
  stalls like a blind one. Demonstrated by the `parse_magic` target and its
  tests; a dictionary is the workaround shipped here, and comparison tracing
  (cmplog / value profile) is the proper fix, listed as a stretch goal.
- **Milestone 5 is not complete.** The harness for fuzzing real external
  software is built and tested, but no genuine third-party bug has been found or
  disclosed through it. That is a campaign to run, not code to write, and it is
  marked ◐ rather than ✅.
- No fork server yet, so external-target throughput is `exec`-bound.

## [0.1.0] — milestone 0
- First working version: a deterministic mutation fuzzer with minimisation.
