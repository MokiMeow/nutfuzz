# 01 — Getting started

## Requirements

A Rust toolchain. No dependencies.

```bash
sudo apt-get update && sudo apt-get install -y cargo rustc
# or: curl https://sh.rustup.rs -sSf | sh
```

## Find the planted bug

```bash
cargo run --release
```

```
nutfuzz milestone 0 — deterministic mutation fuzzing

  target     parse_record
  seed       24301
  iterations 200000

  2976 executions in 0.00s  (5943774 exec/s)

  1 crash(es) found:

    iteration 2975
    reason    out-of-bounds read: record claims 67 payload bytes, only 3 present
    input     8 bytes  "REC\x07C\x01\x01\x00"
    minimised 5 bytes  "REC\x07\x01"
    replay    cargo run --release -- --seed 24301 --iterations 2976
```

## Prove it isn't crying wolf

The same fuzzer against a corrected version of the parser:

```bash
cargo run --release -- --target parse_record_fixed --iterations 200000
```

```
  200000 executions in 0.03s  (6344359 exec/s)

  no crashes found
```

This is the check that makes the finding above mean anything.

## Reproduce a campaign

Every decision comes from the seed, so a run replays exactly:

```bash
cargo run --release -- --seed 24301 --iterations 2976
```

Same seed, same mutations, same crash, same iteration number.

## Options

| Flag | Meaning |
|---|---|
| `--target NAME` | which target to fuzz (`--list` to see them) |
| `--seed N` | PRNG seed; the reproducibility handle |
| `--iterations N` | execution budget |
| `--keep-going` | don't stop at the first crash |
| `--list` | list available targets |

## Tests

```bash
cargo test
```

23 tests: mutation totality (empty / one-byte / max-length inputs), PRNG
determinism, bug-finding, **false positives**, and minimisation.

## Reading the output

- **iteration** — which execution produced the crash; with the seed, it replays.
- **input** — the raw crashing bytes, non-printables escaped.
- **minimised** — the smallest input that still crashes. This is the one to put
  in a bug report.
- **exec/s** — throughput. In-process targets are fast; external binaries
  (milestone 3) will be orders of magnitude slower until the fork server lands.
