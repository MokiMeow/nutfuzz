//! Targets to fuzz.
//!
//! Milestone 0 fuzzes in-process Rust functions. `parse_record` is a
//! deliberately buggy toy parser used to prove the fuzzer works: it contains a
//! planted out-of-bounds read that only triggers on a specific, structured
//! input, so finding it by chance is implausible — the mutation engine has to
//! actually work.
//!
//! Later milestones point the same engine at external binaries and real
//! libraries.

/// The result of one execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Ran to completion without incident.
    Ok,
    /// The target panicked, was killed, or read out of bounds.
    Crash(String),
}

/// A fuzzable target: bytes in, outcome out.
pub type TargetFn = fn(&[u8]) -> Outcome;

/// A toy binary-record parser with a planted bug.
///
/// Format it *believes* it is reading:
///
/// ```text
/// "REC"  magic (3 bytes)
/// kind   1 byte
/// len    1 byte   — number of payload bytes that follow
/// payload len bytes
/// ```
///
/// The bug: for `kind == 0x07` the parser trusts `len` **without checking it
/// against the bytes actually present**, then reads `payload[len - 1]`. Any
/// record claiming more payload than it carries reads past the end.
///
/// This is the single most common real-world parser bug — a length field
/// trusted before it is validated — which is why it makes an honest test of
/// whether a fuzzer works.
pub fn parse_record(data: &[u8]) -> Outcome {
    if data.len() < 5 {
        return Outcome::Ok;
    }
    if &data[0..3] != b"REC" {
        return Outcome::Ok;
    }

    let kind = data[3];
    let len = data[4] as usize;
    let payload = &data[5..];

    match kind {
        0x01 => Outcome::Ok, // a well-behaved record type
        0x07 => {
            // The planted bug: `len` is used without validating it.
            if len == 0 {
                return Outcome::Ok;
            }
            if len > payload.len() {
                // A real parser would return an error here. This one does not,
                // and the read below is out of bounds.
                return Outcome::Crash(format!(
                    "out-of-bounds read: record claims {len} payload bytes, \
                     only {} present",
                    payload.len()
                ));
            }
            let _ = payload[len - 1];
            Outcome::Ok
        }
        _ => Outcome::Ok,
    }
}

/// A correct version of the same parser, used to show the fuzzer reports no
/// crashes when there is genuinely nothing to find.
pub fn parse_record_fixed(data: &[u8]) -> Outcome {
    if data.len() < 5 || &data[0..3] != b"REC" {
        return Outcome::Ok;
    }
    let len = data[4] as usize;
    let payload = &data[5..];
    if data[3] == 0x07 && len > 0 && len <= payload.len() {
        let _ = payload[len - 1];
    }
    Outcome::Ok
}

/// Look a target up by name, for the CLI.
pub fn by_name(name: &str) -> Option<TargetFn> {
    match name {
        "parse_record" => Some(parse_record),
        "parse_record_fixed" => Some(parse_record_fixed),
        _ => None,
    }
}

pub const NAMES: [&str; 2] = ["parse_record", "parse_record_fixed"];

#[cfg(test)]
mod tests {
    use super::{parse_record, parse_record_fixed, Outcome};

    #[test]
    fn ignores_input_that_is_not_a_record() {
        assert_eq!(parse_record(b""), Outcome::Ok);
        assert_eq!(parse_record(b"hello"), Outcome::Ok);
        assert_eq!(parse_record(b"XYZ\x07\x05abcde"), Outcome::Ok);
    }

    #[test]
    fn accepts_a_well_formed_record() {
        assert_eq!(parse_record(b"REC\x07\x03abc"), Outcome::Ok);
        assert_eq!(parse_record(b"REC\x01\xffanything"), Outcome::Ok);
    }

    #[test]
    fn the_planted_bug_is_reachable() {
        // Claims 5 payload bytes, carries 1.
        match parse_record(b"REC\x07\x05a") {
            Outcome::Crash(message) => assert!(message.contains("out-of-bounds")),
            other => panic!("expected a crash, got {other:?}"),
        }
    }

    #[test]
    fn the_fixed_parser_has_no_crash_to_find() {
        for input in [
            &b"REC\x07\x05a"[..],
            b"REC\x07\xffshort",
            b"REC\x07\x00",
            b"REC\x07\x01x",
        ] {
            assert_eq!(parse_record_fixed(input), Outcome::Ok);
        }
    }
}
