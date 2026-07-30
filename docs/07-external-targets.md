# External targets

NutFuzz can run an authorized local binary through the reference exec-per-run
path:

```bash
./target/release/nutfuzz \
  --external /path/to/authorized-target \
  --corpus corpus \
  --iterations 1000 \
  --timeout-ms 500 \
  --memory-mb 256 \
  --cpu-seconds 1 \
  --file-size-bytes 1048576 \
  --artifacts crashes
```

`--arg VALUE` may be repeated. Corpus files are loaded in sorted path order and
mutations derive from the campaign seed. Findings are deduplicated by result
class. Crash minimisation preserves the signal; hang minimisation preserves the
timeout. Unix artifacts and temporary input files use owner-only permissions.

## Classification

- `Crashed`: SIGILL, SIGABRT, SIGBUS, SIGFPE, or SIGSEGV.
- `Hang`: execution exceeded the configured wall-clock timeout.
- `Exited`: normal completion, including nonzero malformed-input rejection.
- `HarnessError`: setup, limit configuration, or spawn failure.

A classified crash is evidence for triage, not automatic proof of a security
defect.

## Isolation and cleanup

Target stdin is backed by a file because a pipe can fill before the timeout
loop begins. Unix targets start in their own process group, and timeout cleanup
kills the group rather than only its direct child.

On Linux, NutFuzz applies:

- `RLIMIT_AS` for address space;
- `RLIMIT_CPU` for CPU seconds;
- `RLIMIT_FSIZE` for created-file size;
- `RLIMIT_CORE=0` for core-dump suppression.

Unsupported platforms reject nonempty limits explicitly.

## Cooperative local forkserver

`ForkserverTarget` uses the versioned protocol documented in
[architecture](02-architecture.md). The included
`nutfuzz-forkserver-target` binary is Linux-only and evaluates only the owned
`parse_nested` target. It demonstrates coverage replies and persistent
initialization without claiming instrumentation of arbitrary programs.

Run the local comparison:

```bash
cargo build --release --all-targets
./target/release/examples/forkserver-bench \
  target/release/nutfuzz-forkserver-target
```

The recorded test-host result was 0.238 seconds for 1,000 forkserver requests
and 1.862 seconds for 1,000 fresh `/bin/true` processes.

## Authorization boundary

External execution is a capability, not permission. Only use it on software
you own or are explicitly authorized to test. The repository's third-party
campaign milestone remains excluded and contains no vulnerability claim.
