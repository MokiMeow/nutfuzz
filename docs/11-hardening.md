# Execution and finding hardening

This review concentrates on the tool's own trust boundaries and reproducibility.
It does not fuzz a third-party target or claim a third-party vulnerability.

## Research and design

- Linux [`poll(2)`](https://man7.org/linux/man-pages/man2/poll.2.html)
  explicitly distinguishes readiness from enough capacity for a large write.
  First-byte readiness also does not promise a complete protocol frame.
- Rust [`UnixStream`](https://doc.rust-lang.org/std/os/unix/net/struct.UnixStream.html)
  exposes read/write timeouts. The client uses local stream sockets behind
  ordinary child stdin/stdout and recalculates the remaining budget for each
  partial operation, without adding libc dependencies or platform-specific
  nonblocking flag constants.
- Rust [`OpenOptions::create_new`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new)
  atomically reserves a new file and rejects existing files and symlinks.
  [`rename`](https://doc.rust-lang.org/std/fs/fn.rename.html) publishes the new
  inode instead of opening the deterministic artifact destination for writing.
- Rust [`Child`](https://doc.rust-lang.org/std/process/struct.Child.html)
  does not kill or wait automatically on drop. Forkserver process ownership
  explicitly terminates the group and waits, including failed handshakes.
- [AFL++ technical details](https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/technical_details.md)
  motivate edge maps and hit buckets. This change retains NutFuzz's existing
  edge hash, bucket thresholds, crash signatures, and sorted snapshot order;
  it optimizes bookkeeping rather than claiming a new search strategy.

## Fixed failure modes

1. A partial forkserver hello or reply could block after the initial readiness
   poll. A request write could also block before polling began. Every complete
   handshake and established request/reply now has one deadline.
2. A failed restart could retain an unvalidated connection. A process is now
   installed only after its hello validates, and RAII owns cleanup. The parent
   closes its duplicate peer endpoints after spawning so an exited peer is
   observed immediately rather than mistaken for a timeout.
3. Encoders could allocate an oversized body before checking the frame cap.
   They now validate the complete wire size first.
4. An external target could exit while descendants remained active. Group
   cleanup now runs for clean exits, crashes, wait failures, and timeouts.
5. Artifact writes could follow symlinks, modify hardlinked files, or keep a
   pre-existing public mode. Publication now uses a fresh private inode and
   atomic rename; failed publication removes the temporary file.
6. Minimization accepted any crash, potentially changing the reported bug.
   It now preserves the campaign's coverage signature. Input-dependent
   diagnostics may change, so the CLI prints the reduced input's actual reason.
7. Trimming could apply a non-idempotent fixup twice, admitting different bytes
   from those tested and underflowing the saved-byte count. It retains the
   exact checked bytes and rejects growth beyond the original candidate.
8. Progress callbacks could disappear when stability or trimming executions
   crossed a reporting interval without landing exactly on it. They now report
   the actual count after crossing a threshold.
9. Replay commands lost dictionaries, fixups, and other search settings. They
   retain and shell-quote the original options, then pin the effective seed
   and finding budget.

## Coverage bookkeeping

For sparse runs the recorder tracks each touched edge once. Reset clears only
those counters, edge counting uses the list length, and snapshots sort the
pairs to preserve the previous dense scan's canonical ordering. Above 4,096
distinct edges it stops growing the list and uses dense scans until reset,
preventing large sorting costs and bounding the additional bookkeeping. Differential
regressions compare full counters, snapshots, global-map decisions, and
signatures against a dense reference across seeded random traces, empty resets,
repeated hits, bucket boundaries, saturation, and full-map occupancy.

Run `cargo run --release --example coverage-bench` to compare complete
trace/snapshot/reset cycles with the previous dense algorithm. The benchmark
uses three seeds, alternating measurement order, and five samples for empty,
short, medium, repeated-loop, and dense workloads. Dense workloads are included
because sorting and touched-edge tracking can cost more than a sequential
scan. Recorder timings are not executions-to-first-finding results and must
not be presented as universal fuzzing speedups.

## Validation and remaining boundaries

Run the release, debug, and doctest commands in [Testing](08-testing.md), plus
the CLI proof checks in CI. Regression tests use owned fixtures only and have
bounded delays; timeout tests allow scheduling slack rather than asserting
millisecond precision.

- Process groups and resource limits are not an OS sandbox. A hostile target
  can escape the group or access files/network with the invoking user's rights.
- Artifact output and parent directories must be controlled by the caller.
  Renaming an artifact file does not protect against directory replacement.
- The v1 frame header leaves 65,531 bytes for a request, distinct from the
  mutation engine's 65,536-byte maximum. No silent wire-version change is made.
- Coverage signatures can collide and cannot prove identical root causes.
  Arbitrary external binaries are still deduplicated by result class.
- Seed-outcome reporting, stronger flaky-crash calibration, and explicit
  whole-corpus/dictionary memory budgets remain follow-up work.

## Measurements on 2026-11-07

Measured with Rust 1.99.1, release optimization level 3, Linux x86_64, and an
Intel Xeon Platinum 8370C cloud CPU. Timings on a shared host are noisy; these
results describe the included synthetic workloads, not arbitrary targets.

Ten `parse_nested` campaigns used seeds 1 through 10 and a 300,000-iteration
budget, comparing baseline `584179b` with this implementation. Execution order
alternated between baseline and improved versions. Finding inputs, coverage
signatures, discovery iterations, corpus summaries, and edge counts matched
for every seed. Both versions found the planted bug in 8/10 campaigns.

- Median campaign wall time: **7.325 seconds to 0.08495 seconds**, about 86x in
  this sparse, synthetic workload.
- Median executions-to-first-finding among successful seeds: **126,059** in
  both versions (the same upper-median convention as the existing CLI).
- The speed change removes coverage bookkeeping overhead; it does not improve
  the search strategy's probability of finding a bug.

The focused recorder benchmark uses 15 samples per workload (five per seed):

| Workload | Baseline median ns/cycle | Adaptive median ns/cycle |
|---|---:|---:|
| Empty | 24,957 | 13.5 |
| 16 trace callbacks | 44,655 | 161.4 |
| 256 trace callbacks | 50,855 | 3,814 |
| 4,096 repeated callbacks | 51,971 | 11,470 |
| 65,536 callbacks, high occupancy | 713,938 | 654,512 |

An initial implementation sorting every touched map regressed dense workloads
by about 2.6x. Capping the tracked list and switching to dense scans removed
that regression in these measurements; high-occupancy results are roughly at
parity and should not be generalized into a universal speed claim.

Raw samples: [recorder CSV](../bench/results/2026-11-07-coverage.csv) and
[campaign CSV](../bench/results/2026-11-07-campaigns.csv). The campaign
fingerprint hashes the matched CLI edge/corpus summary, finding iteration,
signature, and rendered original input. It is a comparison record, not a new
crash identity or a cryptographic guarantee about target behavior.
