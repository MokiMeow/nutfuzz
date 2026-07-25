//! A small deterministic PRNG.
//!
//! Determinism is not a nicety here, it is the whole contract: a fuzzer that
//! cannot replay a run cannot hand you a reproducer. Every mutation decision
//! comes from this generator, so `--seed N` replays a campaign byte for byte.
//!
//! xorshift64* — fast, tiny, and statistically fine for choosing mutations.

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // A zero state is a fixed point for xorshift, so fold it away.
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..bound` (bound must be non-zero).
    pub fn below(&mut self, bound: usize) -> usize {
        debug_assert!(bound > 0, "rng: bound must be non-zero");
        (self.next_u64() % bound as u64) as usize
    }

    /// Uniform in `low..=high`.
    pub fn between(&mut self, low: usize, high: usize) -> usize {
        debug_assert!(low <= high);
        low + self.below(high - low + 1)
    }

    pub fn byte(&mut self) -> u8 {
        (self.next_u64() >> 24) as u8
    }

    /// True with probability `1 / n`.
    pub fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn same_seed_gives_the_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn zero_seed_still_produces_varied_output() {
        let mut rng = Rng::new(0);
        let first = rng.next_u64();
        assert_ne!(first, 0);
        assert_ne!(first, rng.next_u64());
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);
        for _ in 0..10_000 {
            assert!(rng.below(10) < 10);
        }
    }

    #[test]
    fn between_covers_its_bounds() {
        let mut rng = Rng::new(9);
        let mut low_seen = false;
        let mut high_seen = false;
        for _ in 0..10_000 {
            let v = rng.between(3, 6);
            assert!((3..=6).contains(&v));
            low_seen |= v == 3;
            high_seen |= v == 6;
        }
        assert!(low_seen && high_seen, "range endpoints must be reachable");
    }
}
