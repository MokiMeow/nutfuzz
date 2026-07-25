//! End-to-end checks for the defensive external-target CLI.

use std::process::Command;

#[cfg(unix)]
#[test]
fn clean_external_program_completes_without_a_finding() {
    let output = Command::new(env!("CARGO_BIN_EXE_nutfuzz"))
        .args([
            "--external",
            "/bin/true",
            "--iterations",
            "20",
            "--seed",
            "42",
        ])
        .output()
        .expect("run nutfuzz");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(stdout.contains("20 executions"));
    assert!(stdout.contains("no crashes or hangs found"));
}

#[cfg(unix)]
#[test]
fn missing_external_program_is_a_harness_failure_not_a_finding() {
    let output = Command::new(env!("CARGO_BIN_EXE_nutfuzz"))
        .args([
            "--external",
            "/not/a/real/program",
            "--iterations",
            "1",
        ])
        .output()
        .expect("run nutfuzz");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(stderr.contains("external harness"));
}

#[test]
fn zero_timeout_is_rejected_before_running_any_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_nutfuzz"))
        .args(["--external", "unused", "--timeout-ms", "0"])
        .output()
        .expect("run nutfuzz");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(stderr.contains("--timeout-ms needs a positive number"));
}
