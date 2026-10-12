//! Measure sparse coverage bookkeeping with repeatable synthetic traces.
use std::time::Instant;

use nutfuzz::Coverage;

const ITERATIONS: usize = 100_000;
const BLOCKS: usize = 8;

fn main() {
    let mut coverage = Coverage::new();
    let started = Instant::now();
    let mut recorded_edges = 0usize;

    for _ in 0..ITERATIONS {
        coverage.reset();
        for location in 0..BLOCKS {
            coverage.trace(location);
        }
        recorded_edges += coverage.snapshot().len();
    }

    let elapsed = started.elapsed().as_secs_f64();
    println!(
        "coverage bookkeeping: {ITERATIONS} executions, {BLOCKS} traces each, \
         {recorded_edges} recorded edges, {:.0} executions/s ({elapsed:.3} s)",
        ITERATIONS as f64 / elapsed
    );
}
