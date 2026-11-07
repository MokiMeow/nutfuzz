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
Artifact publication writes a fresh `0600` file and atomically renames it over
the deterministic destination. Existing symlinks and hardlinks are replaced,
not followed; existing permissive modes are not inherited. Newly created
artifact directories use `0700`. Use a directory and parent directories you
control: this is not a defense against another process replacing directories
or a hostile program running as your own user.

## Classification

- `Crashed`: SIGILL, SIGABRT, SIGBUS, SIGFPE, or SIGSEGV.
- `Hang`: execution exceeded the configured wall-clock timeout.
- `Exited`: normal completion, including nonzero malformed-input rejection.
- `HarnessError`: setup, limit configuration, or spawn failure.

A classified crash is evidence for triage, not automatic proof of a security
defect.

## Isolation and cleanup

Target stdin is backed by a file because a pipe can fill before the timeout
loop begins. Unix targets start in their own process group. Cleanup kills the
group after normal exits, crashes, wait errors, and timeouts, so a parent that
exits early does not leave its ordinary descendants running. Process groups
and resource limits are not a sandbox: a hostile target can create a new
session, access the filesystem, or use the network. Run only authorized code
and use OS/container isolation when the target is not trusted.

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

The client backs stdin/stdout with local Unix stream sockets; the target's
framed stdin/stdout API is unchanged. A handshake has one deadline, and each
request write plus complete reply has one shared deadline. Each partial read
or write uses only the remaining budget. A peer cannot bypass the timeout by
sending one byte, slowly trickling a frame, or refusing to read a request.
Timeouts stop and reap the process group. Failed handshakes are discarded, and
the next request starts a new connection. Startup/handshake failures are
harness errors; an established request exceeding its deadline is a hang.

The v1 protocol limits an encoded frame payload to 65,536 bytes. Its five-byte
version/kind header leaves at most 65,531 input bytes per request. Oversize
requests and replies are rejected before building their encoded body. This
limit is separate from the mutation engine's 65,536-byte input cap.

References: Rust's [UnixStream timeout APIs](https://doc.rust-lang.org/std/os/unix/net/struct.UnixStream.html),
[atomic file creation](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new),
and Linux [poll readiness semantics](https://man7.org/linux/man-pages/man2/poll.2.html).

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
