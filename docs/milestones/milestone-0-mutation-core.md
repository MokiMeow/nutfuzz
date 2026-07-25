# Milestone 0 — Mutation core ✅ (done)

**Goal:** a deterministic mutation fuzzer that finds a planted bug, minimises
the reproducer, and reports nothing on correct code.

## Concepts

Seeded PRNGs and reproducibility, AFL-style mutation stages, havoc stacking,
crash minimisation, and false-positive testing.

## What shipped

- [x] `src/rng.rs` — xorshift64* PRNG; every search decision comes from it.
- [x] `src/mutate.rs` — 10 mutations (bit/byte flips, interesting integers at
      8/16/32 bits in both endiannesses, arithmetic deltas, insert, delete,
      block duplication, splice) plus `havoc` stacking 1–8 per round, with a
      64 KiB length cap.
- [x] `src/target.rs` — a toy record parser with a **planted length-field bug**,
      and a fixed version of the same parser.
- [x] `src/fuzzer.rs` — the fuzzing loop, crash recording, and a greedy
      chunk-deletion minimiser.
- [x] `src/main.rs` — CLI with `--target`, `--seed`, `--iterations`,
      `--keep-going`, `--list`.
- [x] 23 tests: 15 unit, 7 bug-finding/false-positive, 1 doc test.

## Verified result

```
target parse_record        → crash found at iteration 2975 of 2976 executions
                             8-byte input minimised to 5 bytes "REC\x07\x01"
target parse_record_fixed  → 200,000 executions, no crashes found
```

~6M executions/second in release mode.

## Definition of Done

- [x] `cargo build --release` — zero warnings.
- [x] Finds the planted bug, and the reproducer really does reach the buggy
      branch (asserted on the magic bytes and record kind, not just "a crash").
- [x] **The same seed reproduces the same run** — identical reproducer and
      identical iteration number.
- [x] Different seeds explore differently.
- [x] **No false positives**: 200,000 executions against the fixed parser report
      nothing.
- [x] The minimised reproducer still crashes and shrinks to ≤ 16 bytes (it
      reaches the true 5-byte minimum).
- [x] Every mutation is total — no panic on empty, one-byte, or max-length
      input.

## What was learned (worth keeping)

The false-positive test is the one that gives the others meaning. It is easy to
write something that reports crashes constantly and looks productive; only a
clean 200,000-execution run against corrected code shows the findings are real.
That test is load-bearing and must survive every later milestone.

## References

- [AFL technical whitepaper](https://lcamtuf.coredump.cx/afl/technical_details.txt)
- [docs/05 — The mutation engine](../05-mutation.md)

**Next:** [Milestone 1 — Coverage feedback](milestone-1-coverage.md).
