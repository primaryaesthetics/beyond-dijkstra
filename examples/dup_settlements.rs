//! Q10/V5/V8 experiment: duplicate settlement counts in the top-level
//! log — paper base case (V1 engine) vs the fixed base case (V5) vs the
//! settled bit (V8) — on the zero-heavy tie-rich family sized at the
//! minimum-slack points plus the standard random inputs, with V1-vs-V8
//! wall-clock (median of 3, same process). Duplicates are the
//! re-settlements that eat the |U| cutoff's margin (QUESTIONS.md Q10);
//! V8 removes them by construction (VARIANTS.md V8).
//!
//! ```text
//! cargo run --release --example dup_settlements
//! ```

use beyond_dijkstra::variants::no_transform;
use std::time::Instant;

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

fn graph(n: usize, mult: usize, zero_heavy: bool, seed: u64) -> Vec<(u32, u32, f64)> {
    let m = mult * n;
    let mut r = SplitMix64::new(seed);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = if zero_heavy {
            if r.next_u64() % 3 == 0 { 1.0 } else { 0.0 }
        } else {
            (r.next_u64() % 4) as f64
        };
        edges.push((u, v, w));
    }
    edges
}

/// Continuous weights — exact ties essentially never happen; the control
/// row for whether V8's win needs ties.
fn float_graph(n: usize, mult: usize, seed: u64) -> Vec<(u32, u32, f64)> {
    let m = mult * n;
    let mut r = SplitMix64::new(seed);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = ((r.next_u64() % 1_000_000) + 1) as f64 / 1e6;
        edges.push((u, v, w));
    }
    edges
}

fn median3(mut xs: [f64; 3]) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs[1]
}

fn main() {
    println!(
        "{:>28} {:>8} {:>10} {:>10} {:>10} {:>9} {:>8} {:>9} {:>9}",
        "input", "n", "paper dups", "V5 dups", "V8 dups", "cap hits", "dup/n", "V1 ms", "V8 ms"
    );
    let cases: Vec<(String, usize, Vec<(u32, u32, f64)>, Option<(u64, u64, u64)>)> = vec![
        (
            "zero-heavy minslack".into(),
            4094,
            graph(4094, 6, true, 0x0010_C0DE + 4094),
            Some((3, 2, 2)),
        ),
        (
            "zero-heavy minslack".into(),
            16382,
            graph(16382, 4, true, 0x0010_C0DE + 16382),
            Some((3, 2, 2)),
        ),
        (
            "tie-rich 0..3".into(),
            170_000,
            graph(170_000, 6, false, 0x5EED_0001),
            None,
        ),
        (
            "zero-heavy large".into(),
            500_000,
            graph(500_000, 4, true, 0xD1FF),
            None,
        ),
        (
            "float-random".into(),
            500_000,
            float_graph(500_000, 4, 0xF10A_7001),
            None,
        ),
    ];
    for (name, n, edges, ov) in cases {
        let paper = no_transform::sssp_hybrid(n, &edges, 0, 0, ov);
        let fixed = no_transform::sssp_v5(n, &edges, 0, 0, ov);
        let dedup = no_transform::sssp_v8(n, &edges, 0, 0, ov);
        assert_eq!(paper.dist, fixed.dist, "V5 changed distances on {name}");
        assert_eq!(paper.dist, dedup.dist, "V8 changed distances on {name}");
        let mut t1 = [0f64; 3];
        let mut t8 = [0f64; 3];
        for i in 0..3 {
            let s = Instant::now();
            let _ = no_transform::sssp_hybrid(n, &edges, 0, 0, ov);
            t1[i] = s.elapsed().as_secs_f64() * 1e3;
            let s = Instant::now();
            let _ = no_transform::sssp_v8(n, &edges, 0, 0, ov);
            t8[i] = s.elapsed().as_secs_f64() * 1e3;
        }
        println!(
            "{:>28} {:>8} {:>10} {:>10} {:>10} {:>9} {:>8.4} {:>9.1} {:>9.1}",
            name,
            n,
            paper.dup_settlements,
            fixed.dup_settlements,
            dedup.dup_settlements,
            fixed.base_cap_hits,
            paper.dup_settlements as f64 / n as f64,
            median3(t1),
            median3(t8),
        );
    }
}
