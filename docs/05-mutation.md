# 05 — The mutation engine

*Milestone 0.* How one input becomes the next candidate — the part that decides
whether a fuzzer finds anything.

## The mutations

Each targets a class of real bug rather than being random noise.

| mutation | what it does | the bug it hunts |
|---|---|---|
| `BitFlip` | flips one bit | flag and mask handling |
| `ByteFlip` | replaces a byte | general corruption |
| `InterestingByte` | writes `0x00 0x01 0x7f 0x80 0xff` … | boundary and sign handling |
| `Interesting16` / `32` | writes `0`, `INT_MAX`, `-1` … in **both** endiannesses | integer overflow, sign confusion |
| `Arithmetic` | adds/subtracts 1..35 | off-by-one; length fields nudged past a limit |
| `InsertBytes` | inserts 1..16 bytes | buffers sized from a shorter input |
| `DeleteBytes` | removes a run | truncation and missing-field handling |
| `DuplicateBlock` | repeats an existing chunk | **buffer overflow** — the classic |
| `Splice` | grafts part of another corpus input | crossing structure between inputs |

`InterestingN` writing both byte orders matters: the target may parse either,
and a value that is harmless little-endian can be `0xFFFFFFFF` big-endian.

## Havoc: stacking is what actually works

```rust
pub fn havoc(input: &mut Vec<u8>, other: &[u8], rng: &mut Rng) {
    let rounds = rng.between(1, 8);
    for _ in 0..rounds {
        apply(*rng.pick(&ALL), input, other, rng);
    }
}
```

Single mutations explore too slowly. Many bugs need **two or three coordinated
changes** — a length field raised *and* the payload shortened, which is exactly
the shape of milestone 0's planted bug. Stacking 1–8 mutations per round is
AFL's "havoc" stage and it is what drives discovery.

## Two invariants every mutation must hold

1. **Totality.** No mutation may panic on empty input, one-byte input, or input
   at `MAX_LEN`. Each has a test for all three — those edges are where a fuzzer
   crashes itself instead of the target.
2. **Bounded growth.** `MAX_LEN` (64 KiB) caps every insertion, duplication and
   splice. Without it the corpus grows without limit and the fuzzer becomes its
   own memory-exhaustion bug.

## Determinism

Every decision — which mutation, which offset, how many rounds — comes from the
seeded `Rng`. That makes a campaign replayable:

```
replay    cargo run --release -- --seed 24301 --iterations 2976
```

This is not a convenience feature. A fuzzer that cannot reproduce its own crash
cannot produce a bug report, and a "flaky" finding is indistinguishable from a
bug in the fuzzer.

## What milestone 0 deliberately lacks

There is **no coverage feedback yet**, so the corpus never grows: every
candidate is mutated from a seed input and thrown away. That is a real
limitation, stated plainly — and it is the baseline milestone 1 has to beat with
a measured number. See
[ADR 0002](decisions/0002-mutation-before-coverage.md).

## References

- [AFL technical whitepaper](https://lcamtuf.coredump.cx/afl/technical_details.txt)
  — the origin of these mutation stages.
- AFL++ documentation — the modern refinements.
