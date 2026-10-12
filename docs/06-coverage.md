# 06: Coverage feedback

*Milestone 1.* The idea that turns random search into guided search: and its
real limits, measured rather than asserted.

## The problem with blind mutation

Milestone 0 mutated a seed and threw the result away. To reach a bug behind
several gates, blind mutation must satisfy all of them **at once**. It has no
way of knowing that getting the first one right was progress.

Measured on `parse_nested` (seven gates), 300 000 executions × 10 seeds:

| fuzzer | found | median executions |
|---|---|---|
| blind mutation | **0/10** | not found |
| coverage-guided | **8/10** | 126 059 |

Three million blind executions, zero findings. That gap is the whole argument.

## The fix

Record which **edges** of the control-flow graph each execution takes. If an
input reaches an edge never seen before, it is *interesting*: keep it in the
corpus and mutate it further. Now each gate is solved separately, and every
partial success is preserved.

```
pick an input from the corpus (weighted by energy)
mutate it
run it, recording edges
if it hit a new edge or a higher hit-count bucket:
    add it to the corpus          ← the entire feedback loop
if it crashed:
    deduplicate by signature, minimise, record
```

## The edge bitmap

```rust
index = hash(block) ^ prev
prev  = hash(block) >> 1
```

XOR-ing the previous and current block ids identifies the **edge** A→B rather
than just "B was reached", so different paths through the same code are
distinguishable. The `>> 1` keeps A→B and B→A apart.

### Why the ids are hashed: a bug worth keeping in the docs

The first implementation used the small sequential ids written at each call
site. That collides catastrophically:

```
block 22, prev 10  ->  22 ^ 10 = 28
block 23, prev 11  ->  23 ^ 11 = 28     ← a different edge, same index
```

Two distinct edges became one, coverage stopped growing, and guided fuzzing
performed exactly like blind. This is why AFL assigns every block a **random
compile-time constant** rather than a counter. nutfuzz hashes the readable
call-site id (Fibonacci hashing) to get the same spread, and a test asserts
that each correct magic byte uncovers a *new* edge.

The failure mode is worth internalising: a broken coverage map does not crash
or error: the fuzzer just quietly stops finding things.

## Sparse coverage collection

The 64 KiB counter map stays in place, but the in-process recorder also keeps
an unsorted list of the indices touched during the current execution. Each edge
is recorded once in that list, even when its hit counter is incremented many
times. Reset clears only those indices instead of all 65,536 counters.

Snapshots sort just the touched indices before applying the same hit-count
bucket rules. That preserves the exact ascending order and coverage signatures
used by the original full-map scan, so deterministic seeded runs are unchanged.
`edges_hit()` also reads the tracked cardinality instead of scanning the map.

The change is about low-overhead coverage bookkeeping, not a different
coverage heuristic. To measure a host's throughput of short synthetic runs,
use `cargo run --release --example coverage-overhead`; compare its executions
per second across Git revisions on the same machine.

## Hit-count bucketing

Counts are bucketed (1, 2, 3, 4–7, 8–15, 16–31, 32–127, 128+) so that "this
loop ran 11 times instead of 10" is not treated as new coverage, while "ran
twice instead of once" is. Without it, loop-heavy targets flood the corpus with
near-identical inputs.

## What coverage feedback **cannot** do

Guidance works on partial credit. A single multi-byte comparison gives none:

```rust
if &data[0..4] != b"FUZZ" { return; }
```

Three of four bytes correct reaches exactly the same edges as zero of four, so
there is no gradient to climb and a guided fuzzer stalls like a blind one.
`parse_magic` exists to demonstrate this, and the test suite pins it down:

- a guided campaign **fails** to open that gate in 150 000 executions,
- the same campaign **with a dictionary** containing `"FUZZ"` opens it.

This is precisely why real fuzzers ship dictionaries and comparison tracing
(AFL's cmplog, libFuzzer's value profile): not as extras, but because coverage
alone has this hole.

Written up rather than hidden, because a limitation you can state and
demonstrate is worth more than a benchmark that quietly avoids it.

## Where the coverage comes from

Production fuzzers get edge callbacks from the compiler
(`-fsanitize-coverage=trace-pc-guard`). nutfuzz's in-process targets call
`cov.trace(id)` by hand, so the machinery is visible instead of hidden behind a
flag. The feedback loop, bitmap, bucketing and scheduling are identical either
way: only the source of the callbacks differs.

## Traps

- **Bitmap collisions** hide coverage; `GlobalMap::fill_rate()` reports how full
  the map is.
- **Instrument only the target**, never the fuzzer.
- **Non-deterministic targets** (time, addresses, threads) look "interesting"
  forever and flood the corpus. Detect by running an input twice and comparing.

## References

- [AFL technical whitepaper](https://lcamtuf.coredump.cx/afl/technical_details.txt)
- libFuzzer documentation: value profiles and comparison tracing.
