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

## External execution

The reference path creates an `ExternalTarget` for each candidate execution:

1. write candidate bytes to a private temporary file;
2. spawn the target in a new Unix process group;
3. apply Linux resource limits before `exec`;
4. poll until exit or timeout;
5. kill the process group on timeout;
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
uses a dedicated pipe and `_exit`. A client timeout kills the harness process
group, including its worker, and the next request starts a clean harness.

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
