# Milestone 3: External targets (complete)

**Goal:** fuzz programs outside the NutFuzz process while distinguishing a
crash, hang, clean rejection, and broken harness.

## What shipped

- [x] `ExternalTarget` runs a binary with file-backed stdin and classifies its
      result.
- [x] Fatal Unix signals are findings, timeouts are hangs, normal and nonzero
      exits are target behavior, and spawn failures are harness errors.
- [x] Unix timeouts terminate the complete target process group.
- [x] Deterministic external campaigns support sorted seed corpora, repeated
      arguments, dictionaries, minimisation, deduplication, and private stable
      artifacts.
- [x] Linux `setrlimit` controls cap address space, CPU time, and file size,
      while core dumps are disabled.
- [x] A bounded version 1 local forkserver protocol returns run status and
      coverage from the repository-owned synthetic harness.
- [x] The forkserver parent initializes once, forks a worker per input, closes
      unused descriptors, uses `_exit` in the child, and recovers after a
      timed-out worker by restarting its process group.
- [x] `Executor` separates campaign logic from exec-per-run and persistent
      targets without changing the reference external API.

## Verified behavior

- `/bin/true` is clean and `/bin/false` is a non-finding rejection.
- `kill -SEGV $$` is a crash and an infinite shell loop is a hang.
- A missing executable is a harness error.
- A 1,024-byte file-size limit prevents an owned helper from growing its output
  past 1,024 bytes.
- Repeated forkserver requests return deterministic coverage.
- Truncated, oversized, and wrong-version protocol frames are rejected.
- A forkserver timeout is reported as a hang and the next request succeeds.
- The 1,000-run local comparison measured 0.238 seconds through the forkserver
  versus 1.862 seconds for fresh `/bin/true` processes, a 7.82x speedup on the
  test host.

## Scope boundary

Arbitrary external binaries do not automatically provide edge coverage. They
remain on the safe exec-per-run path unless explicitly adapted to the
cooperative protocol or built with a future instrumentation layer. No
third-party campaign was run.

## Definition of Done

- [x] Signals, hangs, exits, and harness failures are classified correctly.
- [x] Large unread inputs cannot bypass the timeout.
- [x] Resource controls are enforced and tested on Linux.
- [x] External campaigns replay from their seed and write stable private
      artifacts.
- [x] The local persistent executor returns coverage and recovers from timeout.
- [x] Its local speedup is measured without claiming a universal result.
- [x] Build warning-free; all earlier tests green.
- A genuine authorized third-party campaign and disclosure remains outside the
  safe local engineering authorization; see
  [milestone 5](milestone-5-real-target.md).

**Next:** [Milestone 4: Structure-aware input](milestone-4-structure.md).
