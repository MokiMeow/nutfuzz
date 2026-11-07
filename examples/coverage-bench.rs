//! Compare complete trace/snapshot/reset cycles against the old dense map.
//! Run with `cargo run --release --example coverage-bench`.
//! This measures recorder overhead, not bug-finding effectiveness.
use std::hint::black_box;
use std::time::Instant;

use nutfuzz::coverage::{Coverage, MAP_SIZE};
use nutfuzz::rng::Rng;

struct Dense {
    counts: Vec<u32>,
    prev: usize,
}

impl Dense {
    fn new() -> Self {
        Self {
            counts: vec![0; MAP_SIZE],
            prev: 0,
        }
    }

    fn trace(&mut self, location: usize) {
        let spread = location.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 16;
        let index = (spread ^ self.prev) % MAP_SIZE;
        self.counts[index] = self.counts[index].saturating_add(1);
        self.prev = (spread >> 1) % MAP_SIZE;
    }

    fn reset(&mut self) {
        self.counts.fill(0);
        self.prev = 0;
    }

    fn snapshot(&self) -> Vec<(usize, u8)> {
        self.counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count > 0)
            .map(|(index, &count)| {
                let bucket = match count {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    4..=7 => 8,
                    8..=15 => 16,
                    16..=31 => 32,
                    32..=127 => 64,
                    _ => 128,
                };
                (index, bucket)
            })
            .collect()
    }
}

fn main() {
    println!("workload,seed,hits,edges,runs,sample,dense_ns_per_run,sparse_ns_per_run");
    for seed in [1, 42, 2026] {
        for (name, hits, runs) in [
            ("empty", 0, 10_000),
            ("short", 16, 10_000),
            ("medium", 256, 5_000),
            ("loop", 4_096, 1_000),
            ("dense", 65_536, 100),
        ] {
            let mut rng = Rng::new(seed);
            let trace: Vec<usize> = (0..hits)
                .map(|_| {
                    if name == "loop" {
                        0
                    } else {
                        rng.next_u64() as usize
                    }
                })
                .collect();
            let mut sparse = Coverage::new();
            let mut dense = Dense::new();
            // Validate output and warm retained buffers before timing.
            for &location in &trace {
                sparse.trace(location);
                dense.trace(location);
            }
            assert_eq!(sparse.snapshot(), dense.snapshot());
            let edges = sparse.edges_hit();
            sparse.reset();
            dense.reset();
            for sample in 0..5 {
                let mut elapsed = [0.0; 2];
                // Alternate ordering to avoid consistently favouring one map.
                for offset in 0..2 {
                    let implementation = (sample + offset) % 2;
                    let start = Instant::now();
                    for _ in 0..runs {
                        if implementation == 0 {
                            for &location in black_box(&trace) {
                                dense.trace(location);
                            }
                            black_box(dense.snapshot());
                            dense.reset();
                        } else {
                            for &location in black_box(&trace) {
                                sparse.trace(location);
                            }
                            black_box(sparse.snapshot());
                            sparse.reset();
                        }
                    }
                    elapsed[implementation] = start.elapsed().as_nanos() as f64 / runs as f64;
                }
                println!(
                    "{name},{seed},{hits},{edges},{runs},{sample},{:.1},{:.1}",
                    elapsed[0], elapsed[1]
                );
            }
        }
    }
}
