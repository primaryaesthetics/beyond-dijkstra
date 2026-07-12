//! Phase-level profile of the mainline solver.
//!
//! Build with the timer enabled:
//!
//! ```text
//! cargo run --release --features phase-timer --example profile_phases -- [n] [gr-file]
//! ```
//!
//! Same graph family as `bench_fast` (random m = 4n, weights in [0.01, 1),
//! source 0, or a DIMACS `.gr` file). Prints wall-clock seconds attributed
//! to each phase. One warmup + one measured run: the phase *ratios* are the
//! product, and they are stable well within run-to-run noise.

use beyond_dijkstra::solver::sssp_run;

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn dimacs_graph(path: &str) -> (usize, Vec<(u32, u32, f64)>) {
    let text = std::fs::read_to_string(path).expect("read .gr file");
    let mut n = 0usize;
    let mut edges = Vec::new();
    for line in text.lines() {
        let mut it = line.split_ascii_whitespace();
        match it.next() {
            Some("p") => {
                assert_eq!(it.next(), Some("sp"));
                n = it.next().unwrap().parse().unwrap();
            }
            Some("a") => {
                let u: u32 = it.next().unwrap().parse::<u32>().unwrap() - 1;
                let v: u32 = it.next().unwrap().parse::<u32>().unwrap() - 1;
                let w: f64 = it.next().unwrap().parse().unwrap();
                edges.push((u, v, w));
            }
            _ => {}
        }
    }
    (n, edges)
}

fn random_graph(n: usize, seed: u64) -> Vec<(u32, u32, f64)> {
    let m = 4 * n;
    let mut rng = SplitMix64::new(seed);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (rng.next_u64() % n as u64) as u32;
        let v = (rng.next_u64() % n as u64) as u32;
        let w = 0.01 + (rng.next_u64() % 990_000) as f64 / 1_000_000.0;
        edges.push((u, v, w));
    }
    edges
}

fn main() {
    if cfg!(not(feature = "phase-timer")) {
        eprintln!("NOTE: built without --features phase-timer; all zeros below.");
    }
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(1_000_000);
    let (n, edges) = match args.get(2) {
        Some(path) => {
            eprintln!("graph: {path}");
            dimacs_graph(path)
        }
        None => {
            eprintln!("graph: random m=4n, n={n}");
            (n, random_graph(n, 0xC0FFEE))
        }
    };
    eprintln!("n={n}, m={}", edges.len());

    let _ = sssp_run(n, &edges, 0, None); // warmup
    let run = sssp_run(n, &edges, 0, None);
    eprintln!(
        "params: delta={} t={} k={} l_top={} n2={}",
        run.delta, run.t, run.k, run.l_top, run.n2
    );

    let p = &run.phase_times;
    let phases = [
        ("degree reduction", p.reduce),
        ("find_pivots (incl. partition)", p.find_pivots),
        ("  of which tree partition", p.partition),
        ("base_case", p.base_case),
        ("BlockDs::pull (levels >= 1)", p.ds_pull),
        ("BlockDs::merge", p.ds_merge),
        ("pivot bookkeeping + re-selection", p.pivot_bookkeeping),
        ("relax sweep (incl. DS inserts)", p.relax_sweep),
        ("boundary sweeps (S re-insert + W')", p.boundary_sweeps),
    ];
    // `partition` is a subset of find_pivots: exclude it from the sum.
    let accounted: f64 = phases
        .iter()
        .filter(|(name, _)| !name.starts_with("  "))
        .map(|(_, s)| s)
        .sum();
    println!("total {:9.3} s", run.total);
    for (name, secs) in phases {
        println!(
            "{name:38} {secs:9.3} s  {:5.1}%",
            100.0 * secs / run.total.max(1e-12)
        );
    }
    println!(
        "{:38} {:9.3} s  {:5.1}%",
        "unattributed (recursion glue, alloc)",
        run.total - accounted,
        100.0 * (run.total - accounted) / run.total.max(1e-12)
    );
}
