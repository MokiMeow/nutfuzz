//! Running external binaries.
//!
//! In-process targets are fast but limited to code you can link. Real fuzzing
//! points at a program: feed it bytes, watch how it dies.
//!
//! Three outcomes must be told apart, and conflating them is a classic mistake:
//!
//! * **Crash** — killed by SIGSEGV/SIGABRT/SIGFPE/SIGILL/SIGBUS. A finding to
//!   triage; intentional aborts still need to be ruled out.
//! * **Hang** — exceeded its time budget. A different finding class; filing
//!   hangs as crashes wastes everyone's time.
//! * **Exit** — ran to completion, whatever its status. A non-zero exit is a
//!   program rejecting bad input, which is correct behaviour, not a finding.

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT_INPUT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    /// Ran to completion with this exit status.
    Exited(i32),
    /// Killed by a finding-class signal — the signal number and its name.
    Crashed(i32, String),
    /// Exceeded the time budget.
    Hang,
    /// The harness itself failed (binary missing, spawn error).
    HarnessError(String),
}

impl RunResult {
    pub fn is_crash(&self) -> bool {
        matches!(self, RunResult::Crashed(..))
    }
    pub fn is_hang(&self) -> bool {
        matches!(self, RunResult::Hang)
    }
}

fn signal_name(signal: i32) -> &'static str {
    match signal {
        4 => "SIGILL",
        6 => "SIGABRT",
        7 => "SIGBUS",
        8 => "SIGFPE",
        11 => "SIGSEGV",
        _ => "signal",
    }
}

/// Signals worth triaging as a crash, as opposed to normal shutdown signals.
fn is_crash_signal(signal: i32) -> bool {
    matches!(signal, 4 | 6 | 7 | 8 | 11)
}

pub struct ExternalTarget {
    pub program: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}

impl ExternalTarget {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            timeout: Duration::from_secs(2),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Feed `input` on stdin and classify how the process ended.
    pub fn run(&self, input: &[u8]) -> RunResult {
        let (stdin, _input_file) = match InputFile::create(input) {
            Ok(input_file) => input_file,
            Err(error) => {
                return RunResult::HarnessError(format!("prepare target stdin: {error}"));
            }
        };
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .stdin(Stdio::from(stdin))
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        configure_process_group(&mut command);
        let child = command.spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => return RunResult::HarnessError(format!("spawn {}: {e}", self.program)),
        };

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return classify(status),
                Ok(None) => {
                    if started.elapsed() > self.timeout {
                        terminate(&mut child);
                        let _ = child.wait();
                        return RunResult::Hang;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return RunResult::HarnessError(e.to_string()),
            }
        }
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
                Err(std::io::Error::last_os_error())
            }
        });
    }
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate(child: &mut std::process::Child) {
    const SIGKILL: i32 = 9;
    let process_group = -(child.id() as i32);
    unsafe {
        let _ = kill(process_group, SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate(child: &mut std::process::Child) {
    let _ = child.kill();
}

#[cfg(unix)]
unsafe extern "C" {
    fn setpgid(pid: i32, pgid: i32) -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
}

/// Back stdin with a file instead of a pipe. Writing a large input to a pipe
/// before polling the child can deadlock forever when the target never reads
/// stdin, bypassing the timeout the harness is meant to enforce.
struct InputFile {
    path: PathBuf,
}

impl InputFile {
    fn create(input: &[u8]) -> std::io::Result<(File, Self)> {
        for _ in 0..32 {
            let id = NEXT_INPUT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nutfuzz-stdin-{}-{id}",
                std::process::id()
            ));
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let opened = options.open(&path);
            match opened {
                Ok(mut file) => {
                    let input_file = Self { path };
                    file.write_all(input)?;
                    file.seek(SeekFrom::Start(0))?;
                    return Ok((file, input_file));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not reserve a unique temporary input file",
        ))
    }
}

impl Drop for InputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(unix)]
fn classify(status: std::process::ExitStatus) -> RunResult {
    use std::os::unix::process::ExitStatusExt;
    if let Some(signal) = status.signal() {
        if is_crash_signal(signal) {
            return RunResult::Crashed(signal, signal_name(signal).to_string());
        }
        return RunResult::Exited(-signal);
    }
    RunResult::Exited(status.code().unwrap_or(-1))
}

#[cfg(not(unix))]
fn classify(status: std::process::ExitStatus) -> RunResult {
    RunResult::Exited(status.code().unwrap_or(-1))
}

#[cfg(test)]
mod tests {
    use super::{ExternalTarget, RunResult};
    use std::time::Duration;

    #[test]
    fn a_clean_exit_is_not_a_crash() {
        let target = ExternalTarget::new("/bin/true");
        let result = target.run(b"");
        assert!(!result.is_crash(), "got {result:?}");
        assert!(!result.is_hang());
    }

    #[test]
    fn a_nonzero_exit_is_not_a_crash() {
        // Rejecting bad input with a non-zero status is correct behaviour.
        let target = ExternalTarget::new("/bin/false");
        assert!(matches!(target.run(b""), RunResult::Exited(c) if c != 0));
    }

    #[test]
    fn an_infinite_loop_is_a_hang_not_a_crash() {
        let target = ExternalTarget::new("/bin/sh")
            .with_args(vec!["-c".into(), "while :; do :; done".into()])
            .with_timeout(Duration::from_millis(300));
        let result = target.run(b"");
        assert!(result.is_hang(), "got {result:?}");
        assert!(!result.is_crash(), "a hang must not be reported as a crash");
    }

    #[test]
    fn a_fatal_signal_is_a_crash() {
        // `kill -SEGV $$` makes the shell die of SIGSEGV.
        let target = ExternalTarget::new("/bin/sh")
            .with_args(vec!["-c".into(), "kill -SEGV $$".into()])
            .with_timeout(Duration::from_secs(2));
        let result = target.run(b"");
        assert!(result.is_crash(), "got {result:?}");
        if let RunResult::Crashed(_, name) = result {
            assert_eq!(name, "SIGSEGV");
        }
    }

    #[test]
    fn a_missing_binary_is_a_harness_error_not_a_finding() {
        let target = ExternalTarget::new("/definitely/not/a/real/binary");
        let result = target.run(b"");
        assert!(matches!(result, RunResult::HarnessError(_)));
        assert!(!result.is_crash(), "a broken harness must never look like a bug");
    }

    #[test]
    fn a_large_unread_input_cannot_bypass_the_timeout() {
        let target = ExternalTarget::new("/bin/sh")
            .with_args(vec!["-c".into(), "sleep 10".into()])
            .with_timeout(Duration::from_millis(100));
        assert_eq!(target.run(&vec![b'x'; 1024 * 1024]), RunResult::Hang);
    }
}
