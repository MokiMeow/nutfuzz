use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

use crate::exec::{Executor, RunResult};
use crate::protocol::{
    read_hello, read_reply, write_hello, write_run, ForkserverReply,
};

pub struct ForkserverTarget {
    program: PathBuf,
    timeout: Duration,
    process: Option<Process>,
}

struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl ForkserverTarget {
    pub fn spawn(path: impl AsRef<Path>, timeout: Duration) -> io::Result<Self> {
        let mut target = Self {
            program: path.as_ref().to_path_buf(),
            timeout,
            process: None,
        };
        target.restart()?;
        Ok(target)
    }

    pub fn run_with_coverage(&mut self, input: &[u8]) -> ForkserverReply {
        if self.process.is_none() {
            if let Err(error) = self.restart() {
                return harness_error(error);
            }
        }

        let process = self.process.as_mut().expect("process was started");
        if let Err(error) = write_run(&mut process.stdin, input) {
            self.stop();
            return harness_error(error);
        }
        match wait_readable(&process.stdout, self.timeout) {
            Ok(true) => match read_reply(&mut process.stdout) {
                Ok(reply) => reply,
                Err(error) => {
                    self.stop();
                    harness_error(error)
                }
            },
            Ok(false) => {
                self.stop();
                ForkserverReply {
                    result: RunResult::Hang,
                    coverage: Vec::new(),
                }
            }
            Err(error) => {
                self.stop();
                harness_error(error)
            }
        }
    }

    fn restart(&mut self) -> io::Result<()> {
        self.stop();
        let mut command = Command::new(&self.program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        configure_process_group(&mut command);
        let mut child = command.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("forkserver stdin is unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("forkserver stdout is unavailable"))?;
        self.process = Some(Process {
            child,
            stdin,
            stdout,
        });
        let process = self.process.as_mut().expect("process was stored");
        write_hello(&mut process.stdin)?;
        if !wait_readable(&process.stdout, self.timeout)? {
            self.stop();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "forkserver handshake timed out",
            ));
        }
        read_hello(&mut process.stdout)
    }

    fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            terminate(&mut process.child);
            let _ = process.child.wait();
        }
    }
}

impl Executor for ForkserverTarget {
    fn run(&mut self, input: &[u8]) -> RunResult {
        self.run_with_coverage(input).result
    }
}

impl Drop for ForkserverTarget {
    fn drop(&mut self) {
        self.stop();
    }
}

fn harness_error(error: impl std::fmt::Display) -> ForkserverReply {
    ForkserverReply {
        result: RunResult::HarnessError(error.to_string()),
        coverage: Vec::new(),
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            if setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        });
    }
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn wait_readable(stdout: &ChildStdout, timeout: Duration) -> io::Result<bool> {
    use std::os::fd::AsRawFd;

    let millis = timeout.as_millis().min(i32::MAX as u128) as i32;
    let mut descriptor = PollFd {
        fd: stdout.as_raw_fd(),
        events: POLLIN,
        returned_events: 0,
    };
    let result = unsafe { poll(&mut descriptor, 1, millis) };
    if result > 0 {
        Ok((descriptor.returned_events & (POLLIN | POLLHUP)) != 0)
    } else if result == 0 {
        Ok(false)
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(unix))]
fn wait_readable(_stdout: &ChildStdout, _timeout: Duration) -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "forkserver is supported only on Unix",
    ))
}

#[cfg(unix)]
fn terminate(child: &mut Child) {
    const SIGKILL: i32 = 9;
    unsafe {
        let _ = kill(-(child.id() as i32), SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate(child: &mut Child) {
    let _ = child.kill();
}

#[cfg(unix)]
#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    returned_events: i16,
}

#[cfg(unix)]
const POLLIN: i16 = 0x001;
#[cfg(unix)]
const POLLHUP: i16 = 0x010;

#[cfg(unix)]
unsafe extern "C" {
    fn poll(descriptors: *mut PollFd, count: usize, timeout_ms: i32) -> i32;
    fn setpgid(pid: i32, pgid: i32) -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
}
