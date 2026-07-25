# 06 — Coverage feedback

*Milestone 1.* The idea that turns random search into guided search, and the
reason modern fuzzers find bugs blind ones never reach.

## The problem with blind mutation

Milestone 0 mutates a seed and throws the result away. To reach a bug behind

```c
if (a == 0xDEADBEEF)
    if (b == 0x1234)
        if (len > cap)
            boom();
```

blind mutation must hit all three conditions **at once** — astronomically
unlikely. It has no way of knowing that getting the first condition right was
progress.

## The fix

Instrument the target so each execution reports **which edges of the control-flow
graph it took**. If an input reaches an edge never seen before, it is
*interesting*: keep it in the corpus and mutate it further. Now the fuzzer
solves one condition at a time, and each partial success is preserved.

That single idea is why AFL and libFuzzer work.

## The edge bitmap

The standard representation is a fixed-size byte array (AFL uses 64 KiB), where
each edge maps to an index:

```c
// injected at every branch
cur_location = <compile-time random>;
bitmap[cur_location ^ prev_location]++;
prev_location = cur_location >> 1;
```

The XOR of the previous and current block ids identifies the *edge* (A→B),
not just the block — so it distinguishes different paths through the same
code. The `>> 1` stops A→A and A→B/B→A colliding.

Hit counts are bucketed (1, 2, 3, 4–7, 8–15, …) so "this loop ran 11 times
instead of 10" is not treated as new coverage while "ran twice instead of once"
is.

## Getting coverage in Rust

For in-process targets, `-C instrument-coverage` or
`-Z sanitizer=...` / `sanitizer-coverage-trace-pc-guard` gives edge callbacks.
For external binaries (milestone 3) the options are compile-time instrumentation
of the target, or binary rewriting / QEMU mode for targets you cannot rebuild.

Start with in-process instrumentation — it keeps milestone 1 focused on the
*feedback loop* rather than on process plumbing.

## The loop, with feedback

```
pick an input from the corpus (weighted by energy — milestone 2)
mutate it
run it, recording the edge bitmap
if it hit a new edge or a new hit-count bucket:
    add it to the corpus          ← the crucial step
if it crashed:
    minimise and record it
```

## The measurement that proves it works

Milestone 1 is not done because the code exists — it is done when this table has
real numbers in it:

| fuzzer | executions to first crash (median of 10 seeds) |
|---|---|
| blind mutation (M0) | *measured* |
| coverage-guided (M1) | *measured* |

Use a target with a bug behind several nested conditions, so the difference is
visible. If coverage guidance does *not* win by a wide margin, the
implementation is wrong — most likely edges are not being recorded, or the
corpus is not actually growing.

## Traps

- **Bitmap collisions.** Two different edges hashing to one index hides
  coverage. With 64 KiB and a large target, collisions are real; measure the
  fill rate.
- **Coverage of the fuzzer itself.** Only instrument the target.
- **Non-determinism in the target** (time, addresses, threads) makes inputs look
  "interesting" forever and floods the corpus. Detect it by running the same
  input twice and comparing bitmaps.

## References

- [AFL technical whitepaper](https://lcamtuf.coredump.cx/afl/technical_details.txt)
- libFuzzer documentation — value profiles and comparison tracing.
