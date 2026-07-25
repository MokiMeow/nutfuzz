//! Edge coverage: the feedback that turns random search into guided search.
//!
//! Each branch a target takes is recorded as an *edge* — the transition from
//! the previous block to the current one — in a fixed-size bitmap:
//!
//! ```text
//! index = cur_location ^ prev_location
//! prev_location = cur_location >> 1
//! ```
//!
//! XOR-ing the two block ids identifies the transition A→B rather than just
//! "B was reached", so two different paths through the same code are
//! distinguishable. The `>> 1` stops A→B and B→A colliding on the same index.
//! This is AFL's scheme, and it is used here for the same reason: it is one
//! integer of work per branch.
//!
//! How targets report edges: production fuzzers get these callbacks from the
//! compiler (`-fsanitize-coverage=trace-pc-guard`). nutfuzz's in-process
//! targets call [`Coverage::trace`] by hand. External binaries currently run
//! blind; compiler instrumentation plus a shared map remains future work.

pub const MAP_SIZE: usize = 1 << 16; // 64 KiB, as AFL uses

/// Hit counts are bucketed so that "this loop ran 11 times instead of 10" is
/// not treated as new coverage, while "ran twice instead of once" is. Without
/// bucketing, loop-heavy targets flood the corpus with near-identical inputs.
#[inline]
fn bucket(count: u32) -> u8 {
    match count {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 4,
        4..=7 => 8,
        8..=15 => 16,
        16..=31 => 32,
        32..=127 => 64,
        _ => 128,
    }
}

/// Per-execution edge recorder.
pub struct Coverage {
    counts: Vec<u32>,
    prev: usize,
}

impl Default for Coverage {
    fn default() -> Self {
        Self::new()
    }
}

impl Coverage {
    pub fn new() -> Self {
        Self {
            counts: vec![0; MAP_SIZE],
            prev: 0,
        }
    }

    /// Record that control reached block `location`.
    ///
    /// The id is hashed before use. AFL gives each block a *random*
    /// compile-time constant for a reason: small sequential ids collide
    /// constantly under XOR — with ids 20..30, `22^10` and `23^11` both land on
    /// 28, so two different edges look like one and the fuzzer loses the signal
    /// it needs. Hashing spreads readable call-site ids across the map the way
    /// random constants would.
    #[inline]
    pub fn trace(&mut self, location: usize) {
        let spread = Self::spread(location);
        let index = (spread ^ self.prev) % MAP_SIZE;
        self.counts[index] = self.counts[index].saturating_add(1);
        self.prev = (spread >> 1) % MAP_SIZE;
    }

    #[inline]
    fn spread(location: usize) -> usize {
        // Fibonacci hashing: cheap, and scatters consecutive ids widely.
        location.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 16
    }

    pub fn reset(&mut self) {
        for c in self.counts.iter_mut() {
            *c = 0;
        }
        self.prev = 0;
    }

    /// Bucketed snapshot of this run, as (index, bucket) pairs. Sparse because
    /// a single execution touches a tiny fraction of a 64 KiB map.
    pub fn snapshot(&self) -> Vec<(usize, u8)> {
        self.counts
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(i, &c)| (i, bucket(c)))
            .collect()
    }

    pub fn edges_hit(&self) -> usize {
        self.counts.iter().filter(|&&c| c > 0).count()
    }
}

/// The union of everything seen across the whole campaign.
pub struct GlobalMap {
    seen: Vec<u8>,
    edges: usize,
}

impl Default for GlobalMap {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalMap {
    pub fn new() -> Self {
        Self {
            seen: vec![0; MAP_SIZE],
            edges: 0,
        }
    }

    /// Fold a run into the global map.
    ///
    /// Returns true when the run reached an edge never seen before, or hit a
    /// known edge enough more times to land in a higher bucket. That boolean is
    /// the entire feedback signal: true means "keep this input".
    pub fn merge(&mut self, snapshot: &[(usize, u8)]) -> bool {
        let mut interesting = false;
        for &(index, bucket) in snapshot {
            let previous = self.seen[index];
            if previous == 0 {
                self.edges += 1;
                interesting = true;
            } else if bucket > previous {
                interesting = true;
            }
            if bucket > previous {
                self.seen[index] = bucket;
            }
        }
        interesting
    }

    /// Would this run be interesting, without recording it?
    pub fn would_be_interesting(&self, snapshot: &[(usize, u8)]) -> bool {
        snapshot
            .iter()
            .any(|&(i, b)| self.seen[i] == 0 || b > self.seen[i])
    }

    pub fn edges_found(&self) -> usize {
        self.edges
    }

    /// Fraction of the map in use — a collision warning when it climbs.
    pub fn fill_rate(&self) -> f64 {
        self.edges as f64 / MAP_SIZE as f64
    }
}

/// A stable signature for a run's coverage, used to group crashes by root
/// cause rather than by input bytes (milestone 2).
pub fn signature(snapshot: &[(usize, u8)]) -> u64 {
    // FNV-1a over the edge indices. Two inputs that crash through the same
    // path share a signature and are therefore the same bug.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &(index, _) in snapshot {
        for byte in index.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{signature, Coverage, GlobalMap};

    #[test]
    fn distinguishes_edges_not_just_blocks() {
        // A->B and B->A must land on different indices.
        let mut forward = Coverage::new();
        forward.trace(10);
        forward.trace(20);

        let mut backward = Coverage::new();
        backward.trace(20);
        backward.trace(10);

        assert_ne!(forward.snapshot(), backward.snapshot());
    }

    #[test]
    fn new_edges_are_interesting_once() {
        let mut global = GlobalMap::new();
        let mut cov = Coverage::new();
        cov.trace(1);
        cov.trace(2);

        assert!(global.merge(&cov.snapshot()), "first sighting is interesting");
        assert!(!global.merge(&cov.snapshot()), "the same path is not");
    }

    #[test]
    fn a_higher_hit_bucket_is_interesting() {
        let mut global = GlobalMap::new();

        let mut once = Coverage::new();
        once.trace(5);
        global.merge(&once.snapshot());

        // Same edge, many more hits -> a higher bucket -> interesting again.
        let mut many = Coverage::new();
        for _ in 0..50 {
            many.trace(5);
        }
        assert!(global.merge(&many.snapshot()));
    }

    #[test]
    fn small_hit_count_changes_are_not_interesting() {
        let mut global = GlobalMap::new();
        let mut a = Coverage::new();
        for _ in 0..10 {
            a.trace(7);
        }
        global.merge(&a.snapshot());

        // 11 hits is the same bucket as 10 — not new coverage.
        let mut b = Coverage::new();
        for _ in 0..11 {
            b.trace(7);
        }
        assert!(!global.merge(&b.snapshot()));
    }

    #[test]
    fn edges_found_counts_distinct_edges() {
        let mut global = GlobalMap::new();
        let mut cov = Coverage::new();
        cov.trace(1);
        cov.trace(2);
        cov.trace(3);
        global.merge(&cov.snapshot());
        assert_eq!(global.edges_found(), cov.snapshot().len());
    }

    #[test]
    fn the_same_path_has_the_same_signature() {
        let mut a = Coverage::new();
        a.trace(1);
        a.trace(2);
        let mut b = Coverage::new();
        b.trace(1);
        b.trace(2);
        assert_eq!(signature(&a.snapshot()), signature(&b.snapshot()));

        let mut c = Coverage::new();
        c.trace(9);
        assert_ne!(signature(&a.snapshot()), signature(&c.snapshot()));
    }
}
