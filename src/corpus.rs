//! The corpus and its scheduler.
//!
//! With coverage feedback the corpus grows, so *which* entry to mutate next
//! starts to matter as much as how. Two ideas from AFL:
//!
//! * **Energy** — favour entries that are small, fast, and reached rare edges.
//!   A bloated entry costs the same budget as a lean one and mutates worse.
//! * **Favoured set** — a minimal subset of the corpus that still covers every
//!   known edge (a greedy set cover). Fuzzing mostly within it avoids spending
//!   the budget on a hundred inputs that all exercise the same path.

use crate::rng::Rng;

#[derive(Debug, Clone)]
pub struct Entry {
    pub input: Vec<u8>,
    /// Edge indices this input reached.
    pub edges: Vec<usize>,
    /// Times it has been chosen as a mutation parent.
    pub picked: u32,
    /// Whether it is in the minimal covering set.
    pub favoured: bool,
}

impl Entry {
    fn new(input: Vec<u8>, edges: Vec<usize>) -> Self {
        Self {
            input,
            edges,
            picked: 0,
            favoured: false,
        }
    }

    /// How many mutations this entry deserves.
    ///
    /// Short inputs execute faster and mutate more usefully; entries that have
    /// already been mined heavily get less. Kept deliberately simple — the
    /// value is in *having* a policy that is not round-robin.
    fn energy(&self) -> u32 {
        let mut score: u32 = 100;
        if self.favoured {
            score = score.saturating_mul(4);
        }
        if self.input.len() < 32 {
            score = score.saturating_mul(2);
        } else if self.input.len() > 512 {
            score /= 2;
        }
        // Diminishing returns on an entry already used a lot.
        score / (1 + self.picked / 50).max(1)
    }
}

#[derive(Default)]
pub struct Corpus {
    entries: Vec<Entry>,
}

impl Corpus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn favoured_count(&self) -> usize {
        self.entries.iter().filter(|e| e.favoured).count()
    }

    pub fn add(&mut self, input: Vec<u8>, edges: Vec<usize>) {
        self.entries.push(Entry::new(input, edges));
        self.recompute_favoured();
    }

    /// Greedy set cover: repeatedly take the entry covering the most
    /// not-yet-covered edges. The result is a small set that still reaches
    /// everything the whole corpus reaches.
    fn recompute_favoured(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.favoured = false;
        }

        let mut covered: std::collections::HashSet<usize> = std::collections::HashSet::new();
        loop {
            let mut best: Option<(usize, usize)> = None; // (index, new edges)
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.favoured {
                    continue;
                }
                let gain = entry.edges.iter().filter(|e| !covered.contains(e)).count();
                if gain > 0 && best.map_or(true, |(_, b)| gain > b) {
                    best = Some((i, gain));
                }
            }
            match best {
                Some((i, _)) => {
                    self.entries[i].favoured = true;
                    for &edge in &self.entries[i].edges {
                        covered.insert(edge);
                    }
                }
                None => break,
            }
        }
    }

    /// Choose a mutation parent, weighted by energy.
    pub fn pick(&mut self, rng: &mut Rng) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }
        let total: u64 = self.entries.iter().map(|e| e.energy() as u64).sum();
        if total == 0 {
            return Some(rng.below(self.entries.len()));
        }
        let mut ticket = (rng.next_u64() % total) as i64;
        for (i, entry) in self.entries.iter().enumerate() {
            ticket -= entry.energy() as i64;
            if ticket < 0 {
                self.entries[i].picked += 1;
                return Some(i);
            }
        }
        Some(self.entries.len() - 1)
    }

    pub fn input(&self, index: usize) -> &[u8] {
        &self.entries[index].input
    }

    /// A random entry, used as a splice donor.
    pub fn random_input(&self, rng: &mut Rng) -> &[u8] {
        &self.entries[rng.below(self.entries.len())].input
    }

    pub fn mean_len(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.entries.iter().map(|e| e.input.len()).sum::<usize>() as f64
            / self.entries.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::Corpus;
    use crate::rng::Rng;

    #[test]
    fn favoured_is_a_minimal_covering_set() {
        let mut corpus = Corpus::new();
        corpus.add(b"a".to_vec(), vec![1, 2, 3]);  // covers everything below
        corpus.add(b"b".to_vec(), vec![1, 2]);     // redundant
        corpus.add(b"c".to_vec(), vec![3]);        // redundant
        corpus.add(b"d".to_vec(), vec![9]);        // unique edge

        assert_eq!(
            corpus.favoured_count(),
            2,
            "only the entry covering 1,2,3 and the one covering 9 are needed"
        );
    }

    #[test]
    fn picking_is_weighted_but_reaches_everything() {
        let mut corpus = Corpus::new();
        corpus.add(vec![0u8; 8], vec![1]);
        corpus.add(vec![0u8; 8], vec![2]);
        corpus.add(vec![0u8; 8], vec![3]);

        let mut rng = Rng::new(1);
        let mut seen = [false; 3];
        for _ in 0..2000 {
            seen[corpus.pick(&mut rng).unwrap()] = true;
        }
        assert!(seen.iter().all(|&s| s), "every entry must be reachable");
    }

    #[test]
    fn small_inputs_get_more_energy_than_large_ones() {
        let mut corpus = Corpus::new();
        corpus.add(vec![0u8; 8], vec![1]);      // small
        corpus.add(vec![0u8; 4096], vec![2]);   // bloated

        let mut rng = Rng::new(7);
        let mut small = 0;
        for _ in 0..4000 {
            if corpus.pick(&mut rng).unwrap() == 0 {
                small += 1;
            }
        }
        assert!(small > 2400, "the small entry should dominate, got {small}/4000");
    }

    #[test]
    fn an_empty_corpus_picks_nothing() {
        let mut corpus = Corpus::new();
        let mut rng = Rng::new(1);
        assert!(corpus.pick(&mut rng).is_none());
    }
}
