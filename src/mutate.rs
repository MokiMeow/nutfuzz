//! The mutation engine: how one input becomes the next candidate.
//!
//! These are the classic AFL-style mutations. Each is cheap, and together they
//! cover the ways real parsers break: a flipped bit in a length field, an
//! off-by-one at a boundary, an interesting integer (0, -1, INT_MAX), a
//! repeated block that overflows a buffer.
//!
//! `havoc` applies a random stack of them, which is what actually drives
//! discovery — single mutations explore too slowly.

use crate::rng::Rng;

/// Values that disproportionately trigger bugs: boundaries, overflow points,
/// and sign-flip points at each width.
const INTERESTING_8: [u8; 9] = [0x00, 0x01, 0x10, 0x20, 0x40, 0x7f, 0x80, 0xfe, 0xff];
const INTERESTING_16: [u16; 8] =
    [0x0000, 0x0001, 0x007f, 0x0080, 0x00ff, 0x7fff, 0x8000, 0xffff];
const INTERESTING_32: [u32; 8] = [
    0x0000_0000, 0x0000_0001, 0x0000_7fff, 0x0000_8000,
    0x0000_ffff, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff,
];

pub(crate) const MAX_LEN: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutation {
    BitFlip,
    ByteFlip,
    InterestingByte,
    Interesting16,
    Interesting32,
    Arithmetic,
    InsertBytes,
    DeleteBytes,
    DuplicateBlock,
    Splice,
}

pub const ALL: [Mutation; 10] = [
    Mutation::BitFlip,
    Mutation::ByteFlip,
    Mutation::InterestingByte,
    Mutation::Interesting16,
    Mutation::Interesting32,
    Mutation::Arithmetic,
    Mutation::InsertBytes,
    Mutation::DeleteBytes,
    Mutation::DuplicateBlock,
    Mutation::Splice,
];

/// Apply one mutation in place. `other` is another corpus input, used by
/// splice; pass an empty slice if none is available.
pub fn apply(kind: Mutation, input: &mut Vec<u8>, other: &[u8], rng: &mut Rng) {
    match kind {
        Mutation::BitFlip => {
            if input.is_empty() {
                return;
            }
            let at = rng.below(input.len());
            input[at] ^= 1 << rng.below(8);
        }

        Mutation::ByteFlip => {
            if input.is_empty() {
                return;
            }
            let at = rng.below(input.len());
            input[at] = rng.byte();
        }

        Mutation::InterestingByte => {
            if input.is_empty() {
                return;
            }
            let at = rng.below(input.len());
            input[at] = *rng.pick(&INTERESTING_8);
        }

        Mutation::Interesting16 => {
            if input.len() < 2 {
                return;
            }
            let at = rng.below(input.len() - 1);
            let value = *rng.pick(&INTERESTING_16);
            // Both byte orders matter: the target may read either.
            let bytes = if rng.one_in(2) {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            };
            input[at..at + 2].copy_from_slice(&bytes);
        }

        Mutation::Interesting32 => {
            if input.len() < 4 {
                return;
            }
            let at = rng.below(input.len() - 3);
            let value = *rng.pick(&INTERESTING_32);
            let bytes = if rng.one_in(2) {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            };
            input[at..at + 4].copy_from_slice(&bytes);
        }

        Mutation::Arithmetic => {
            if input.is_empty() {
                return;
            }
            let at = rng.below(input.len());
            // Small deltas find off-by-ones that random bytes miss.
            let delta = rng.between(1, 35) as u8;
            input[at] = if rng.one_in(2) {
                input[at].wrapping_add(delta)
            } else {
                input[at].wrapping_sub(delta)
            };
        }

        Mutation::InsertBytes => {
            if input.len() >= MAX_LEN {
                return;
            }
            let at = rng.below(input.len() + 1);
            let count = rng.between(1, 16).min(MAX_LEN - input.len());
            let fill = if rng.one_in(2) {
                rng.byte()
            } else if input.is_empty() {
                rng.byte()
            } else {
                input[rng.below(input.len())]
            };
            for i in 0..count {
                input.insert(at + i, fill);
            }
        }

        Mutation::DeleteBytes => {
            if input.len() < 2 {
                return;
            }
            let count = rng.between(1, (input.len() / 2).max(1));
            let at = rng.below(input.len() - count + 1);
            input.drain(at..at + count);
        }

        Mutation::DuplicateBlock => {
            // The classic buffer-overflow finder: repeat an existing chunk.
            if input.is_empty() || input.len() >= MAX_LEN {
                return;
            }
            let len = rng.between(1, input.len().min(256));
            let from = rng.below(input.len() - len + 1);
            let block: Vec<u8> = input[from..from + len].to_vec();
            let at = rng.below(input.len() + 1);
            if input.len() + block.len() > MAX_LEN {
                return;
            }
            input.splice(at..at, block);
        }

        Mutation::Splice => {
            // Graft a piece of another input; how structure crosses over.
            if other.is_empty() || input.is_empty() {
                return;
            }
            let take = rng.between(1, other.len());
            let from = rng.below(other.len() - take + 1);
            let at = rng.below(input.len() + 1);
            if input.len() + take > MAX_LEN {
                return;
            }
            input.splice(at..at, other[from..from + take].iter().copied());
        }
    }
}

/// Apply a random stack of 1..=8 mutations — AFL's "havoc" stage.
///
/// Stacking matters: many bugs need two or three coordinated changes (a length
/// field *and* the data it describes), which single mutations rarely produce.
pub fn havoc(input: &mut Vec<u8>, other: &[u8], rng: &mut Rng) {
    let rounds = rng.between(1, 8);
    for _ in 0..rounds {
        apply(*rng.pick(&ALL), input, other, rng);
    }
    if input.is_empty() {
        input.push(rng.byte()); // never hand the target nothing
    }
}

#[cfg(test)]
mod tests {
    use super::{apply, havoc, ALL, MAX_LEN};
    use crate::rng::Rng;

    #[test]
    fn every_mutation_survives_an_empty_input() {
        let mut rng = Rng::new(1);
        for kind in ALL {
            let mut input = Vec::new();
            apply(kind, &mut input, b"other", &mut rng);
        }
    }

    #[test]
    fn every_mutation_survives_a_one_byte_input() {
        let mut rng = Rng::new(2);
        for kind in ALL {
            let mut input = vec![0x41];
            apply(kind, &mut input, b"", &mut rng);
        }
    }

    #[test]
    fn havoc_never_returns_an_empty_input() {
        let mut rng = Rng::new(3);
        for _ in 0..2000 {
            let mut input = vec![0u8; rng.below(8)];
            havoc(&mut input, b"seed", &mut rng);
            assert!(!input.is_empty());
        }
    }

    #[test]
    fn havoc_respects_the_length_cap() {
        let mut rng = Rng::new(4);
        let mut input = vec![0x41; MAX_LEN];
        for _ in 0..5000 {
            havoc(&mut input, b"abcd", &mut rng);
            assert!(input.len() <= MAX_LEN);
        }
    }

    #[test]
    fn mutations_are_deterministic_for_a_seed() {
        let run = |seed| {
            let mut rng = Rng::new(seed);
            let mut input = b"hello world".to_vec();
            for _ in 0..50 {
                havoc(&mut input, b"other input", &mut rng);
            }
            input
        };
        assert_eq!(run(99), run(99));
    }

    #[test]
    fn havoc_actually_changes_things() {
        let mut rng = Rng::new(5);
        let original = b"the quick brown fox".to_vec();
        let mut changed = 0;
        for _ in 0..100 {
            let mut input = original.clone();
            havoc(&mut input, b"", &mut rng);
            if input != original {
                changed += 1;
            }
        }
        assert!(changed > 90, "havoc should almost always mutate ({changed}/100)");
    }
}
