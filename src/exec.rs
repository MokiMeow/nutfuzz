//! Running external binaries.
//!
//! In-process targets are fast but limited to code you can link. Real fuzzing
//! points at a program: feed it bytes, watch how it dies.
//!
//! Three outcomes must be told apart, and conflating them is a classic mistake:
//!
//! * **Crash** — killed by SIGSEGV/SIGABRT/SIGFPE/SIGILL/SIGBUS. A real bug.
//! * **Hang** — exceeded its time budget. Also a real bug, but a *different*
//!   one; filing hangs as crashes wastes everyone's time.
//! * **Exit** — ran to completion, whatever its status. A non-zero exit is a
//!   program rejecting bad input, which is correct behaviour, not a finding.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    /// Ran to completion with this exit status.
    Exited(i32),
    /// Killed by a signal — the signal number and its name.
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

/// Signals that mean "the program broke", as opposed to being shut down.
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
        let child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => return RunResult::HarnessError(format!("spawn {}: {e}", self.program)),
        };

        if let Some(mut stdin) = child.stdin.take() {
            // A target that exits before reading everything gives EPIPE; that
            // is the target's choice, not a harness failure.
            let _ = stdin.write_all(input);
        }

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return classify(status),
                Ok(None) => {
                    if started.elapsed() > self.timeout {
                        let _ = child.kill();
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
}
