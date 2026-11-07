//! Cooperative execution with an end-to-end deadline.
//!
//! Readiness of the first byte does not bound a complete frame. Socket-backed
//! stdin/stdout let every partial read and write share one remaining budget,
//! including a peer that stops reading requests or trickles a reply. The wire
//! protocol is unchanged; targets still read stdin and write stdout.

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::exec::{Executor, RunResult};
use crate::protocol::ForkserverReply;

#[cfg(unix)]
use {
    crate::protocol::{read_hello, read_reply, write_hello, write_run},
    std::io::{Read, Write},
    std::os::{fd::OwnedFd, unix::net::UnixStream},
    std::process::{Child, Command, Stdio},
    std::time::Instant,
};

pub struct ForkserverTarget {
    program: PathBuf,
    timeout: Duration,
    #[cfg(unix)]
    process: Option<Process>,
}

#[cfg(unix)]
struct Process {
    child: Child,
    stdin: UnixStream,
    stdout: UnixStream,
}

impl ForkserverTarget {
    pub fn spawn(path: impl AsRef<Path>, timeout: Duration) -> io::Result<Self> {
        let mut target = Self {
            program: path.as_ref().to_path_buf(),
            timeout,
            #[cfg(unix)]
            process: None,
        };
        target.restart()?;
        Ok(target)
    }

    #[cfg(unix)]
    pub fn run_with_coverage(&mut self, input: &[u8]) -> ForkserverReply {
        if self.process.is_none() {
            if let Err(error) = self.restart() {
                return harness_error(error);
            }
        }
        let Some(process) = self.process.as_mut() else {
            return harness_error("forkserver is unavailable");
        };
        let deadline = Deadline::new(self.timeout);
        let result = write_run(&mut deadline.stream(&mut process.stdin), input)
            .and_then(|()| read_reply(&mut deadline.stream(&mut process.stdout)));
        match result {
            Ok(reply) => reply,
            Err(error) => {
                self.stop();
                if error.kind() == io::ErrorKind::TimedOut {
                    ForkserverReply {
                        result: RunResult::Hang,
                        coverage: Vec::new(),
                    }
                } else {
                    harness_error(error)
                }
            }
        }
    }

    #[cfg(not(unix))]
    pub fn run_with_coverage(&mut self, _input: &[u8]) -> ForkserverReply {
        harness_error(unsupported())
    }

    #[cfg(unix)]
    fn restart(&mut self) -> io::Result<()> {
        self.stop();
        let (stdin, child_stdin) = UnixStream::pair()?;
        let (stdout, child_stdout) = UnixStream::pair()?;
        let mut command = Command::new(&self.program);
        command
            .stdin(Stdio::from(OwnedFd::from(child_stdin)))
            .stdout(Stdio::from(OwnedFd::from(child_stdout)))
            .stderr(Stdio::null());
        crate::exec::configure_process_group(&mut command);
        let child = command.spawn()?;
        // Command retains explicit Stdio handles after spawn. Close the
        // parent's copies of the peer endpoints so child exit produces EOF.
        drop(command);
        let mut process = Process {
            child,
            stdin,
            stdout,
        };
        let deadline = Deadline::new(self.timeout);
        write_hello(&mut deadline.stream(&mut process.stdin))?;
        read_hello(&mut deadline.stream(&mut process.stdout))?;
        // Publish only a fully handshaken process. Any setup error drops and
        // reaps this process rather than leaving a poisoned connection behind.
        self.process = Some(process);
        Ok(())
    }

    #[cfg(not(unix))]
    fn restart(&mut self) -> io::Result<()> {
        let _ = (&self.program, self.timeout);
        Err(unsupported())
    }

    #[cfg(unix)]
    fn stop(&mut self) {
        self.process = None;
    }
}

impl Executor for ForkserverTarget {
    fn run(&mut self, input: &[u8]) -> RunResult {
        self.run_with_coverage(input).result
    }
}

#[cfg(unix)]
impl Drop for Process {
    fn drop(&mut self) {
        crate::exec::terminate(&mut self.child);
        let _ = self.child.wait();
    }
}

fn harness_error(error: impl std::fmt::Display) -> ForkserverReply {
    ForkserverReply {
        result: RunResult::HarnessError(error.to_string()),
        coverage: Vec::new(),
    }
}

#[cfg(not(unix))]
fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "forkserver requires Unix")
}

#[cfg(unix)]
struct Deadline {
    started: Instant,
    timeout: Duration,
}

#[cfg(unix)]
impl Deadline {
    fn new(timeout: Duration) -> Self {
        Self {
            started: Instant::now(),
            timeout,
        }
    }

    fn remaining(&self) -> io::Result<Duration> {
        self.timeout
            .checked_sub(self.started.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(timed_out)
    }

    fn stream<'a>(&'a self, stream: &'a mut UnixStream) -> TimedStream<'a> {
        TimedStream {
            stream,
            deadline: self,
        }
    }
}

#[cfg(unix)]
struct TimedStream<'a> {
    stream: &'a mut UnixStream,
    deadline: &'a Deadline,
}

#[cfg(unix)]
impl Read for TimedStream<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream
            .set_read_timeout(Some(self.deadline.remaining()?))?;
        self.stream.read(buffer).map_err(normalize_timeout)
    }
}

#[cfg(unix)]
impl Write for TimedStream<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.stream
            .set_write_timeout(Some(self.deadline.remaining()?))?;
        self.stream.write(buffer).map_err(normalize_timeout)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.deadline.remaining().map(|_| ())
    }
}

#[cfg(unix)]
fn normalize_timeout(error: io::Error) -> io::Error {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => timed_out(),
        _ => error,
    }
}

#[cfg(unix)]
fn timed_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "forkserver exchange timed out")
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn an_unread_request_cannot_block_past_its_deadline() {
        let (mut writer, _reader) = UnixStream::pair().unwrap();
        writer.set_nonblocking(true).unwrap();
        let bytes = [0; 4096];
        loop {
            match writer.write(&bytes) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => panic!("fill socket: {error}"),
            }
        }
        writer.set_nonblocking(false).unwrap();
        let deadline = Deadline::new(Duration::from_millis(100));
        let error = deadline.stream(&mut writer).write_all(b"x").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(deadline.started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn zero_timeout_expires_before_io() {
        let (mut reader, _writer) = UnixStream::pair().unwrap();
        let deadline = Deadline::new(Duration::ZERO);
        let error = deadline.stream(&mut reader).read(&mut [0]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }
}
