# Getting started

## Requirements

Use a Rust 2021 compatible `cargo` and `rustc`. NutFuzz has no third-party crate
dependencies. Linux is required for the complete external resource-control and
forkserver paths.

## Build and test

```bash
cargo build --release --all-targets
cargo test --release --all-targets
```

The release suite currently runs 82 tests.

## First campaigns

```bash
# Find the planted record-parser bug.
./target/release/nutfuzz

# Stay quiet on the corrected parser.
./target/release/nutfuzz --target parse_record_fixed --iterations 200000

# Replay one deterministic campaign.
./target/release/nutfuzz --target parse_nested \
  --seed 12648430 --iterations 300000

# Compare guided and blind mutation.
./target/release/nutfuzz --bench
```

The seed controls every random mutation and scheduling decision. The same
binary, seed, inputs, configuration, and budget reproduce the same search path.
Elapsed time and executions per second remain host-dependent.

## Important options

| Flag | Meaning |
|---|---|
| `--target NAME` | Select an included in-process target |
| `--list` | List included targets |
| `--seed N` | Select the deterministic random stream |
| `--iterations N` | Set the campaign loop budget |
| `--mode guided|blind` | Enable or disable coverage feedback |
| `--schedule weighted|round-robin` | Select corpus scheduling |
| `--progress N` | Print progress every N executions |
| `--stability-runs N` | Rerun interesting inputs before admission |
| `--trim` | Trim stable coverage-increasing corpus entries |
| `--dict A,B` | Supply dictionary tokens |
| `--auto-dict` | Learn bounded printable tokens from admitted inputs |
| `--grammar json` | Generate small JSON objects as a mutation strategy |
| `--fixup checksum` | Repair the included wrapping-checksum format |
| `--external PROGRAM` | Run an authorized local program |
| `--arg VALUE` | Add one external target argument; repeat as needed |
| `--corpus DIR` | Load external seeds in sorted path order |
| `--timeout-ms N` | Set the external execution timeout |
| `--memory-mb N` | Set the Linux address-space limit |
| `--cpu-seconds N` | Set the Linux CPU-time limit |
| `--file-size-bytes N` | Set the Linux output-file limit |
| `--artifacts DIR` | Set the private finding output directory |

## Structured campaign example

```bash
./target/release/nutfuzz --target parse_checksum \
  --dict CHECK --fixup checksum --seed 41 --iterations 2000
```

The owned target accepts a structured token only when its final byte matches
the wrapping sum of the preceding bytes. The regression test proves the same
campaign stays quiet without the fixup.

## Safe external smoke test

```bash
./target/release/nutfuzz --external /bin/true --iterations 20 \
  --timeout-ms 500 --memory-mb 64 --cpu-seconds 1 \
  --file-size-bytes 1024
```

It must report 20 executions and no crashes or hangs. Read
[external targets](07-external-targets.md) before using another program.

## Reading a finding

- `iteration` identifies when the finding first appeared.
- `reason` records the owned target message or external result class.
- `signature` groups in-process paths by root-cause coverage.
- `input` is the original candidate with non-printable bytes escaped.
- `minimised` is the smallest candidate retained by greedy deletion.
- `replay` records the target, seed, and budget required to reproduce the
  campaign.
