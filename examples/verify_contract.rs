//! Contract verification runs (QUESTIONS.md Q3, Q10, Q12).
//!
//! Part 1 — the Q3 demonstrator. A crafted star (no ties needed): with
//! t = 2 the base-case cutoff is t³ = 8, so the ninth settlement ends the
//! loop with B′ = the next-smallest stored value, and a zero-weight edge
//! out of the last settled vertex inserts a label BELOW that bound. The
//! paper's Algorithm 4 returns the violating pair; the V5 cap repairs it.
//!
//! Part 2 — the adversarial family from the V5 experiment, both engines'
//! counters: base-case and recursive-return violations of dB[D] ≥ B′
//! (a recursive-return hit would be Q12's leak observed live) and
//! full-execution dichotomy violations (Q10's sub-finding).
//!
//! ```text
//! cargo run --release --features verify-contract --example verify_contract
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::{choose_params, Solver};

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

/// Same generator as `dup_settlements.rs` so part 2 re-runs the recorded
/// experiment's inputs.
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

struct Counters {
    bound_base: u64,
    bound_rec: u64,
    dichotomy: u64,
    cap_hits: u64,
}

fn run(
    n: usize,
    edges: &[(u32, u32, f64)],
    (t, k): (u64, u64),
    level: u64,
    b: Label,
    fixed_base: bool,
) -> Counters {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    let mut solver = Solver::new(&adj, t, k);
    solver.fixed_base = fixed_base;
    solver.d[0] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });
    let _ = solver.bmssp(b, &[0], level);
    Counters {
        bound_base: solver.viol_bound_base,
        bound_rec: solver.viol_bound_rec,
        dichotomy: solver.viol_dichotomy,
        cap_hits: solver.base_cap_hits,
    }
}

fn main() {
    // Part 1: the crafted Q3 demonstrator. Vertices: source 0; leaves
    // 1..=9 at weights 1..=9; vertex 10 behind a zero-weight edge from
    // leaf 8. Settlement order: 0, then leaves ascending. The ninth
    // settlement (leaf 8) exceeds the t³ = 8 cutoff; the final pull's
    // bound is leaf 9's label (len 9), and relaxing 8 →(w=0) 10 inserts
    // len 8 < 9 into D after that pull.
    let mut edges: Vec<(u32, u32, f64)> = (1..=9u32).map(|i| (0, i, i as f64)).collect();
    edges.push((8, 10, 0.0));
    let b = Label::Fin(Lab {
        len: 100.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });
    println!("Q3 demonstrator (base case, t=2, cutoff 8, B=len 100):");
    println!(
        "{:>28} {:>11} {:>10} {:>10} {:>9}",
        "engine", "bound viol", "rec viol", "dichotomy", "cap hits"
    );
    for (name, fixed) in [("paper (Algorithm 4)", false), ("V5 fixed base", true)] {
        let c = run(11, &edges, (2, 2), 0, b, fixed);
        println!(
            "{:>28} {:>11} {:>10} {:>10} {:>9}",
            name, c.bound_base, c.bound_rec, c.dichotomy, c.cap_hits
        );
    }

    // Part 2: the recorded adversarial family, full recursion from the
    // top (B = INF, level = l_top), counters over every return.
    println!();
    println!("adversarial family (full runs, counters over all returns):");
    println!(
        "{:>28} {:>8} {:>6} {:>11} {:>10} {:>10} {:>9}",
        "input", "n", "engine", "base viol", "rec viol", "dichotomy", "cap hits"
    );
    let cases: Vec<(String, usize, Vec<(u32, u32, f64)>, Option<(u64, u64)>)> = vec![
        (
            "zero-heavy minslack".into(),
            4094,
            graph(4094, 6, true, 0x0010_C0DE + 4094),
            Some((2, 2)),
        ),
        (
            "zero-heavy minslack".into(),
            16382,
            graph(16382, 4, true, 0x0010_C0DE + 16382),
            Some((2, 2)),
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
        // Parameter recipe mirrors the V1 engine (original n throughout).
        let (t, k) = match ov {
            Some(tk) => tk,
            None => {
                let (_, t, k, _) = choose_params(n, edges.len(), n);
                (t, k)
            }
        };
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;
        for (engine, fixed) in [("paper", false), ("V5", true)] {
            let c = run(n, &edges, (t, k), l_top, Label::Inf, fixed);
            println!(
                "{:>28} {:>8} {:>6} {:>11} {:>10} {:>10} {:>9}",
                name, n, engine, c.bound_base, c.bound_rec, c.dichotomy, c.cap_hits
            );
        }
    }
}
