//! Q10/V5 experiment: duplicate settlement counts in the top-level log,
//! paper base case (V1 engine) vs the fixed base case (V5), on the
//! zero-heavy tie-rich family sized at the minimum-slack points plus the
//! standard random inputs. Duplicates are the re-settlements that eat the
//! |U| cutoff's margin (QUESTIONS.md Q10); V5's claim predicts the
//! base-case escape channel closes.
//!
//! ```text
//! cargo run --release --example dup_settlements
//! ```

use beyond_dijkstra::variants::no_transform;

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

fn main() {
    println!(
        "{:>28} {:>8} {:>10} {:>10} {:>10} {:>9}",
        "input", "n", "paper dups", "V5 dups", "dup/n", "cap hits"
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
    ];
    for (name, n, edges, ov) in cases {
        let paper = no_transform::sssp_hybrid(n, &edges, 0, 0, ov);
        let fixed = no_transform::sssp_v5(n, &edges, 0, 0, ov);
        assert_eq!(paper.dist, fixed.dist, "V5 changed distances on {name}");
        println!(
            "{:>28} {:>8} {:>10} {:>10} {:>10.4} {:>9}",
            name,
            n,
            paper.dup_settlements,
            fixed.dup_settlements,
            paper.dup_settlements as f64 / n as f64,
            fixed.base_cap_hits
        );
    }
}
