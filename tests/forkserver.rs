#![cfg(target_os = "linux")]

use std::io::Cursor;
use std::time::Duration;

use nutfuzz::exec::RunResult;
use nutfuzz::forkserver::ForkserverTarget;
use nutfuzz::protocol::{
    read_hello, read_reply, write_reply, ForkserverReply, MAX_FRAME_SIZE,
};

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
