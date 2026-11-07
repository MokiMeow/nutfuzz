# Architecture

## In-process feedback loop

```text
Corpus::pick
    |
    v
havoc / grammar / tokens
    |
    v
optional format fixup
    |
    v
TargetFn(input, Coverage)
    |
    +--> finding signature and minimisation
    |
    `--> exact stability snapshots
             |
             v
       optional trimming
             |
             v
       corpus admission and token learning
```

Every random choice draws from one seeded `Rng`. Guided mode schedules the
growing corpus; blind mode mutates only the original seeds.

Corpus trimming retains the exact fixed-up bytes whose coverage was checked.
It does not apply the fixup a second time to the retained result: user-provided
fixups need not be idempotent. A trim trial that grows beyond the original
candidate length is rejected, and failed trials leave the original candidate
unchanged.

Crash minimisation preserves the original edge-coverage signature, matching the
campaign's deduplication identity. Input-dependent diagnostics may change as
bytes are removed. It is a greedy deletion search, not a guarantee of the
globally smallest reproducer. Coverage is a heuristic identity: distinct bugs
may share the same edge set, and bitmap or signature collisions can merge
unrelated paths. Signature preservation is not proof of a shared root cause.

## External execution

The reference path creates an `ExternalTarget` for each candidate execution:

1. write candidate bytes to a private temporary file;
2. spawn the target in a new Unix process group;
3. apply Linux resource limits before `exec`;
4. poll until exit or timeout;
5. kill the process group on every terminal outcome;
6. classify the result and minimize unique findings.

`Executor` is the campaign boundary. `ExternalTarget` implements it for
exec-per-run behavior, and `ForkserverTarget` implements it for a persistent
cooperative harness.

## Local forkserver

The client and owned harness exchange bounded length-prefixed frames:

```text
client                         persistent harness
  |------ hello v1 ------------------>|
  |<----- hello v1 -------------------|
  |------ run(input) ---------------->|
  |                         fork worker
  |                         run owned target
  |<----- result + coverage -----------|
```

Frames are limited to 64 KiB and carry an explicit protocol version. The worker
uses a dedicated pipe and `_exit`. The client's stdin/stdout transport uses
local Unix sockets so writes and complete frame reads share a deadline,
including partial frames. A client timeout kills the harness process group,
including its worker, and the next request starts a clean harness. Process
ownership is RAII-based: handshake errors and target destruction also reap it.

## Components

| File | Responsibility |
|---|---|
| `src/rng.rs` | deterministic random stream |
| `src/mutate.rs` | bounded mutation and stacked havoc |
| `src/coverage.rs` | edge hashing, buckets, novelty, signatures |
| `src/corpus.rs` | weighted and round-robin scheduling, favored cover |
| `src/dictionary.rs` | bounded manual and learned tokens |
| `src/structure.rs` | harvesting, token mutation, JSON, fixup type |
| `src/trim.rs` | coverage-preserving deletion |
| `src/target.rs` | owned planted and corrected targets |
| `src/fuzzer.rs` | guided loop, stability, reports, minimisation |
| `src/limits.rs` | Linux resource-limit setup |
| `src/exec.rs` | process execution and `Executor` |
| `src/external.rs` | deterministic external campaigns and artifacts |
| `src/protocol.rs` | bounded versioned frames |
| `src/forkserver.rs` | persistent client and timeout recovery |
| `src/main.rs` | CLI and human-readable reporting |

## Design boundaries

- No third-party crates.
- No silent fallback when a requested platform protection is unsupported.
- Arbitrary external binaries remain blind unless instrumented or adapted to
  the cooperative protocol.
- The included forkserver evaluates only the repository-owned synthetic target.
