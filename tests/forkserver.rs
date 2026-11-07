#![cfg(target_os = "linux")]

use std::io::Cursor;
use std::time::Duration;

use nutfuzz::exec::RunResult;
use nutfuzz::forkserver::ForkserverTarget;
use nutfuzz::protocol::{read_hello, read_reply, write_reply, ForkserverReply, MAX_FRAME_SIZE};

#[test]
fn protocol_round_trips_results_and_coverage() {
    let replies = [
        ForkserverReply {
            result: RunResult::Exited(0),
            coverage: vec![(1, 1), (7, 3)],
        },
        ForkserverReply {
            result: RunResult::Crashed(11, "SIGSEGV".into()),
            coverage: vec![(9, 1)],
        },
        ForkserverReply {
            result: RunResult::Hang,
            coverage: Vec::new(),
        },
        ForkserverReply {
            result: RunResult::HarnessError("bad frame".into()),
            coverage: Vec::new(),
        },
    ];

    for expected in replies {
        let mut bytes = Vec::new();
        write_reply(&mut bytes, &expected).unwrap();
        assert_eq!(read_reply(&mut Cursor::new(bytes)).unwrap(), expected);
    }
}

#[test]
fn protocol_rejects_truncated_and_oversized_frames() {
    let truncated = 10u32.to_le_bytes().to_vec();
    assert!(read_reply(&mut Cursor::new(truncated)).is_err());

    let oversized = ((MAX_FRAME_SIZE + 1) as u32).to_le_bytes().to_vec();
    assert!(read_reply(&mut Cursor::new(oversized)).is_err());

    let payload = [2u32.to_le_bytes().as_slice(), &[1]].concat();
    let wrong_version = [
        (payload.len() as u32).to_le_bytes().as_slice(),
        payload.as_slice(),
    ]
    .concat();
    assert!(read_hello(&mut Cursor::new(wrong_version)).is_err());
}

#[test]
fn persistent_target_reuses_process_and_returns_deterministic_coverage() {
    let mut target = ForkserverTarget::spawn(
        env!("CARGO_BIN_EXE_nutfuzz-forkserver-target"),
        Duration::from_millis(500),
    )
    .unwrap();

    let first = target.run_with_coverage(b"AAAAAAA");
    let second = target.run_with_coverage(b"AAAAAAA");
    assert_eq!(first, second);
    assert_eq!(first.result, RunResult::Exited(0));
    assert!(!first.coverage.is_empty());
}

#[test]
fn persistent_target_classifies_crashes_and_recovers_after_timeout() {
    let mut target = ForkserverTarget::spawn(
        env!("CARGO_BIN_EXE_nutfuzz-forkserver-target"),
        Duration::from_millis(100),
    )
    .unwrap();

    let crash = target.run_with_coverage(b"FUZZ\x02\x42\xff");
    assert!(crash.result.is_crash(), "got {:?}", crash.result);

    let hang = target.run_with_coverage(b"HANG");
    assert_eq!(hang.result, RunResult::Hang);

    let clean = target.run_with_coverage(b"AAAAAAA");
    assert_eq!(clean.result, RunResult::Exited(0));
}

#[test]
fn encoders_reject_oversize_messages_before_writing() {
    use nutfuzz::protocol::write_run;
    let mut bytes = Vec::new();
    assert!(write_run(&mut bytes, &vec![0; MAX_FRAME_SIZE - 4]).is_err());
    assert!(bytes.is_empty());
    write_run(&mut bytes, &vec![0; MAX_FRAME_SIZE - 5]).unwrap();
    assert_eq!(bytes.len(), MAX_FRAME_SIZE + 4);

    let mut bytes = Vec::new();
    assert!(write_reply(
        &mut bytes,
        &ForkserverReply {
            result: RunResult::Exited(0),
            coverage: vec![(1, 1); MAX_FRAME_SIZE / 5],
        }
    )
    .is_err());
    assert!(bytes.is_empty());
}

#[test]
fn partial_handshake_obeys_the_complete_exchange_deadline() {
    let script = Script::new("partial-hello", "printf '\\005'; sleep 2");
    let started = std::time::Instant::now();
    let error = ForkserverTarget::spawn(&script.0, Duration::from_millis(100))
        .err()
        .expect("partial handshake must time out");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn partial_reply_body_times_out_and_the_next_request_restarts() {
    let script = Script::new(
        "partial-reply",
        &format!(
            "{HELLO}\ndd bs=1 count=10 of=/dev/null 2>/dev/null\n\
         printf '\\020\\000\\000\\000\\001'; sleep 2"
        ),
    );
    let mut target = ForkserverTarget::spawn(&script.0, Duration::from_millis(100)).unwrap();
    let started = std::time::Instant::now();
    assert_eq!(target.run_with_coverage(b"x").result, RunResult::Hang);
    assert!(started.elapsed() < Duration::from_secs(1));
    // A failed restart must not leave a half-handshaken process usable.
    script.replace("printf '\\001\\000\\000\\000\\001'");
    assert!(matches!(
        target.run_with_coverage(b"x").result,
        RunResult::HarnessError(_)
    ));
    script.replace(&format!(
        "exec '{}'",
        env!("CARGO_BIN_EXE_nutfuzz-forkserver-target")
    ));
    assert_eq!(target.run_with_coverage(b"x").result, RunResult::Exited(0));
}

#[test]
fn trickled_handshake_cannot_reset_the_deadline() {
    let script = Script::new(
        "trickle-hello",
        "\
        printf '\\005'; sleep .08; printf '\\000'; sleep .08; \
        printf '\\000'; sleep .08; printf '\\000'; sleep .08; \
        printf '\\001\\000\\000\\000\\001'",
    );
    let started = std::time::Instant::now();
    let error = ForkserverTarget::spawn(&script.0, Duration::from_millis(200))
        .err()
        .expect("each byte must share the original deadline");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(1));
}

const HELLO: &str = "dd bs=1 count=9 of=/dev/null 2>/dev/null; \
    printf '\\005\\000\\000\\000\\001\\000\\000\\000\\001'";

struct Script(std::path::PathBuf);

impl Script {
    fn new(name: &str, body: &str) -> Self {
        let path = std::env::temp_dir().join(format!("nutfuzz-{name}-{}", std::process::id()));
        let script = Self(path);
        script.replace(body);
        script
    }

    fn replace(&self, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(&self.0, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn exited_peer_reports_eof_without_waiting_for_the_handshake_timeout() {
    let started = std::time::Instant::now();
    let error = ForkserverTarget::spawn("/bin/true", Duration::from_secs(2))
        .err()
        .expect("a program that exits cannot handshake");
    // Either the hello write races exit, or the subsequent read sees EOF.
    assert!(matches!(
        error.kind(),
        std::io::ErrorKind::UnexpectedEof
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::ConnectionReset
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}
