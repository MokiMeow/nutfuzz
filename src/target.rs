//! Targets to fuzz.
//!
//! Milestone 0 fuzzed uninstrumented functions. From milestone 1 targets also
//! report **edge coverage** by calling `cov.trace(id)` at each branch, which is
//! what lets the fuzzer keep inputs that reach new code.
//!
//! Production fuzzers get these callbacks from the compiler
//! (`-fsanitize-coverage=trace-pc-guard`); here they are written by hand so the
//! feedback machinery is visible rather than hidden behind a compiler flag. The
//! machinery itself is identical.

use crate::coverage::Coverage;

/// The result of one execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Crash(String),
}

/// A fuzzable target: bytes in, outcome out, edges recorded along the way.
pub type TargetFn = fn(&[u8], &mut Coverage) -> Outcome;

// ---------------------------------------------------------------- record ---

/// A toy binary-record parser with a planted length-field bug.
///
/// ```text
/// "REC"  magic (3 bytes)
/// kind   1 byte
/// len    1 byte   — payload bytes that follow
/// payload len bytes
/// ```
///
/// For `kind == 0x07` the parser trusts `len` without validating it against
/// the bytes present, then reads `payload[len - 1]`. A length field trusted
/// before it is checked is the most common real parser bug.
pub fn parse_record(data: &[u8], cov: &mut Coverage) -> Outcome {
    cov.trace(1);
    if data.len() < 5 {
        cov.trace(2);
        return Outcome::Ok;
    }
    cov.trace(3);
    if &data[0..3] != b"REC" {
        cov.trace(4);
        return Outcome::Ok;
    }
    cov.trace(5);

    let kind = data[3];
    let len = data[4] as usize;
    let payload = &data[5..];

    match kind {
        0x01 => {
            cov.trace(6);
            Outcome::Ok
        }
        0x07 => {
            cov.trace(7);
            if len == 0 {
                cov.trace(8);
                return Outcome::Ok;
            }
            cov.trace(9);
            if len > payload.len() {
                cov.trace(10);
                return Outcome::Crash(format!(
                    "out-of-bounds read: record claims {len} payload bytes, \
                     only {} present",
                    payload.len()
                ));
            }
            cov.trace(11);
            let _ = payload[len - 1];
            Outcome::Ok
        }
        _ => {
            cov.trace(12);
            Outcome::Ok
        }
    }
}

/// The same parser, corrected. Used to prove the fuzzer reports nothing when
/// there is genuinely nothing to find.
pub fn parse_record_fixed(data: &[u8], cov: &mut Coverage) -> Outcome {
    cov.trace(1);
    if data.len() < 5 || &data[0..3] != b"REC" {
        cov.trace(2);
        return Outcome::Ok;
    }
    cov.trace(3);
    let len = data[4] as usize;
    let payload = &data[5..];
    if data[3] == 0x07 && len > 0 && len <= payload.len() {
        cov.trace(4);
        let _ = payload[len - 1];
    }
    Outcome::Ok
}

// ---------------------------------------------------------------- nested ---

/// A bug behind seven nested gates — the target that separates a
/// coverage-guided fuzzer from a blind one.
///
/// ```text
/// magic   'F' 'U' 'Z' 'Z'   checked one byte at a time
/// version 0x02
/// mode    0x42
/// size            1 byte, must be > 200   <- the bug
/// ```
///
/// **Why the magic is compared byte by byte, not with one `==` on a slice.**
/// Coverage feedback gives partial credit: getting `data[0] == 'F'` right
/// reaches an edge nobody had reached, so that input is kept and mutated
/// further, and the next byte becomes a separate small problem. Seven gates in
/// sequence is then seven easy steps instead of one impossible leap.
///
/// A single `&data[0..4] != b"FUZZ"` is **opaque** to coverage — there is no
/// edge for "three of four bytes correct", so the fuzzer gets no signal and
/// stalls exactly like a blind one. That is a real limitation of coverage
/// guidance, not of this implementation: it is precisely why dictionaries and
/// comparison tracing (AFL's cmplog, libFuzzer's value profile) exist. The
/// per-byte form models what a compiler's instrumentation actually sees in code
/// that checks a header field at a time.
pub fn parse_nested(data: &[u8], cov: &mut Coverage) -> Outcome {
    cov.trace(20);
    if data.len() < 7 {
        cov.trace(21);
        return Outcome::Ok;
    }
    cov.trace(22);

    if data[0] != b'F' { cov.trace(23); return Outcome::Ok; }
    cov.trace(24);                                   // gate 1
    if data[1] != b'U' { cov.trace(25); return Outcome::Ok; }
    cov.trace(26);                                   // gate 2
    if data[2] != b'Z' { cov.trace(27); return Outcome::Ok; }
    cov.trace(28);                                   // gate 3
    if data[3] != b'Z' { cov.trace(29); return Outcome::Ok; }
    cov.trace(30);                                   // gate 4

    if data[4] != 0x02 { cov.trace(31); return Outcome::Ok; }
    cov.trace(32);                                   // gate 5
    if data[5] != 0x42 { cov.trace(33); return Outcome::Ok; }
    cov.trace(34);                                   // gate 6

    if data[6] as usize > 200 {
        cov.trace(35);                               // gate 7 — the bug
        return Outcome::Crash(format!(
            "integer overflow: size {} exceeds the 200-byte buffer",
            data[6]
        ));
    }
    cov.trace(36);
    Outcome::Ok
}

/// The same gates, but the magic is one opaque comparison.
///
/// Kept deliberately: it is the honest demonstration that coverage feedback
/// alone **cannot** open a multi-byte magic gate, and that a dictionary can.
pub fn parse_magic(data: &[u8], cov: &mut Coverage) -> Outcome {
    cov.trace(40);
    if data.len() < 5 {
        cov.trace(41);
        return Outcome::Ok;
    }
    cov.trace(42);
    if &data[0..4] != b"FUZZ" {
        cov.trace(43);
        return Outcome::Ok;   // no partial credit — coverage learns nothing
    }
    cov.trace(44);
    if data[4] as usize > 200 {
        cov.trace(45);
        return Outcome::Crash(format!(
            "integer overflow: size {} exceeds the 200-byte buffer",
            data[4]
        ));
    }
    cov.trace(46);
    Outcome::Ok
}

/// `parse_nested` with the final check corrected.
pub fn parse_nested_fixed(data: &[u8], cov: &mut Coverage) -> Outcome {
    cov.trace(20);
    if data.len() < 7 || &data[0..4] != b"FUZZ" || data[4] != 0x02 || data[5] != 0x42 {
        cov.trace(21);
        return Outcome::Ok;
    }
    cov.trace(22);
    let size = (data[6] as usize).min(200); // clamped
    let _ = size;
    Outcome::Ok
}

// ------------------------------------------------------------------ misc ---

pub fn by_name(name: &str) -> Option<TargetFn> {
    match name {
        "parse_record" => Some(parse_record),
        "parse_record_fixed" => Some(parse_record_fixed),
        "parse_nested" => Some(parse_nested),
        "parse_nested_fixed" => Some(parse_nested_fixed),
        "parse_magic" => Some(parse_magic),
        _ => None,
    }
}

pub const NAMES: [&str; 5] = [
    "parse_record",
    "parse_record_fixed",
    "parse_nested",
    "parse_nested_fixed",
    "parse_magic",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: TargetFn, data: &[u8]) -> Outcome {
        let mut cov = Coverage::new();
        f(data, &mut cov)
    }

    #[test]
    fn record_bug_is_reachable() {
        match run(parse_record, b"REC\x07\x05a") {
            Outcome::Crash(m) => assert!(m.contains("out-of-bounds")),
            other => panic!("expected a crash, got {other:?}"),
        }
    }

    #[test]
    fn record_accepts_valid_input() {
        assert_eq!(run(parse_record, b"REC\x07\x03abc"), Outcome::Ok);
        assert_eq!(run(parse_record, b"hello"), Outcome::Ok);
    }

    #[test]
    fn fixed_record_has_nothing_to_find() {
        for input in [&b"REC\x07\x05a"[..], b"REC\x07\xffshort", b"REC\x07\x00"] {
            assert_eq!(run(parse_record_fixed, input), Outcome::Ok);
        }
    }

    #[test]
    fn nested_bug_needs_every_gate() {
        assert_eq!(run(parse_nested, b"FUZZ\x02\x42\x10"), Outcome::Ok, "size too small");
        assert_eq!(run(parse_nested, b"FUZZ\x02\x00\xff"), Outcome::Ok, "wrong mode");
        assert_eq!(run(parse_nested, b"FUZZ\x01\x42\xff"), Outcome::Ok, "wrong version");
        assert_eq!(run(parse_nested, b"XUZZ\x02\x42\xff"), Outcome::Ok, "wrong magic byte 0");
        assert_eq!(run(parse_nested, b"FXZZ\x02\x42\xff"), Outcome::Ok, "wrong magic byte 1");

        match run(parse_nested, b"FUZZ\x02\x42\xff") {
            Outcome::Crash(m) => assert!(m.contains("overflow")),
            other => panic!("expected a crash, got {other:?}"),
        }
    }

    #[test]
    fn each_correct_magic_byte_reaches_a_new_edge() {
        // This is the property coverage guidance steers on: partial progress
        // must be visible as coverage, or there is nothing to steer with.
        let edges = |input: &[u8]| {
            let mut cov = Coverage::new();
            parse_nested(input, &mut cov);
            cov.edges_hit()
        };
        let none = edges(b"XXXXXXX");
        let one = edges(b"FXXXXXX");
        let two = edges(b"FUXXXXX");
        let three = edges(b"FUZXXXX");
        assert!(one > none && two > one && three > two,
                "each correct byte must uncover a new edge ({none} {one} {two} {three})");
    }

    #[test]
    fn magic_target_gives_no_partial_credit() {
        // The opaque comparison: getting 3 of 4 bytes right reaches nothing new.
        let edges = |input: &[u8]| {
            let mut cov = Coverage::new();
            parse_magic(input, &mut cov);
            cov.edges_hit()
        };
        assert_eq!(edges(b"XXXXX"), edges(b"FUZXX"),
                   "a single memcmp leaks no progress — this is the blind spot \
                    that dictionaries exist to cover");
    }

    #[test]
    fn nested_fixed_has_nothing_to_find() {
        assert_eq!(run(parse_nested_fixed, b"FUZZ\x02\x42\xff"), Outcome::Ok);
    }

    #[test]
    fn passing_a_gate_reaches_new_edges() {
        let mut none = Coverage::new();
        parse_nested(b"XXXX\x02\x42\xff", &mut none);

        let mut some = Coverage::new();
        parse_nested(b"FUZZ\x02\x00\x00", &mut some);

        assert!(
            some.edges_hit() > none.edges_hit(),
            "getting the magic right must show up as coverage — that is the \
             signal the fuzzer steers on"
        );
    }
}
