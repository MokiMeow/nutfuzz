use std::time::{Duration, Instant};

use nutfuzz::exec::ExternalTarget;
use nutfuzz::forkserver::ForkserverTarget;

fn main() {
    const RUNS: u64 = 1_000;

    let direct = ExternalTarget::new("/bin/true")
        .with_timeout(Duration::from_millis(500));
    let started = Instant::now();
    for _ in 0..RUNS {
        let _ = direct.run(b"AAAAAAA");
    }
    let direct_elapsed = started.elapsed();

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/release/nutfuzz-forkserver-target".into());
    let mut forkserver =
        ForkserverTarget::spawn(path, Duration::from_millis(500)).unwrap();
    let started = Instant::now();
    for _ in 0..RUNS {
        let _ = forkserver.run_with_coverage(b"AAAAAAA");
    }
    let forkserver_elapsed = started.elapsed();

    println!("runs              {RUNS}");
    println!("exec per run      {:.3}s", direct_elapsed.as_secs_f64());
    println!(
        "local forkserver  {:.3}s",
        forkserver_elapsed.as_secs_f64()
    );
    println!(
        "speedup           {:.2}x",
        direct_elapsed.as_secs_f64() / forkserver_elapsed.as_secs_f64()
    );
}
