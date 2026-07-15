//! Q14 experiment: classify every duplicate settlement event of the paper
//! engine by its channel (QUESTIONS.md Q14 addendum) and test Lemma D's
//! prediction for the foreign-W′ ones — a pred-chain ancestor within k−1
//! hops must be a failed root (∈ Q) of the appending call, exactly so on
//! inputs without cross-walk ties (float-random), possibly undercounted on
//! tie-rich inputs where the arborescence path may be a tied-sibling path.
//!
//! Same input family as `dup_settlements` (Q10/V5/V8 experiments) so the
//! totals line up with the recorded runs.
//!
//! ```text
//! cargo run --release --features dup-classify --example dup_classify
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

fn main() {
    println!(
        "{:>22} {:>8} {:>6} {:>9} {:>8} {:>8} {:>9} {:>8} {:>8} {:>8}",
        "input",
        "n",
        "dups",
        "base-seed",
        "base-ins",
        "own-root",
        "own-swept",
        "foreign",
        "f-anc-Q",
        "f-anc-S"
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
        let run = no_transform::sssp_hybrid(n, &edges, 0, 0, ov);
        let c = &run.dup_class;
        let classified =
            c.base_seeded + c.base_inserted + c.wprime_own_root + c.wprime_own_swept
                + c.wprime_foreign;
        assert_eq!(
            classified as usize, run.dup_settlements,
            "classification must partition the duplicate count on {name}"
        );
        println!(
            "{:>22} {:>8} {:>6} {:>9} {:>8} {:>8} {:>9} {:>8} {:>8} {:>8}",
            name,
            n,
            run.dup_settlements,
            c.base_seeded,
            c.base_inserted,
            c.wprime_own_root,
            c.wprime_own_swept,
            c.wprime_foreign,
            c.foreign_anc_in_q,
            c.foreign_anc_in_s,
        );
    }
}
