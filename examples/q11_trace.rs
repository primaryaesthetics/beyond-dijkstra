//! Q11 confirmation: trace one duplicate settlement end-to-end.
//!
//! Pass 1 runs the V1 engine (no degree reduction) on the zero-heavy
//! min-slack input from `dup_settlements.rs` and finds the vertex whose
//! SECOND settlement appears earliest in the top-level log. Pass 2 re-runs
//! the identical computation with the trace pointed at that vertex and
//! prints every event that touched it — relax acceptances (exact ties
//! flagged), D inserts by site, pulls, K/W membership, settlements — each
//! with a call-path snapshot. The dump is the evidence for QUESTIONS.md
//! Q11's mechanism claim.
//!
//! ```text
//! cargo run --release --features q11-trace --example q11_trace
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::Solver;

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

/// Same generator and seed as `dup_settlements.rs`'s zero-heavy min-slack
/// case, so the traced run is the recorded experiment's run.
fn graph(n: usize, mult: usize, seed: u64) -> Vec<(u32, u32, f64)> {
    let m = mult * n;
    let mut r = SplitMix64::new(seed);
    let mut edges = Vec::with_capacity(m);
    for _ in 0..m {
        let u = (r.next_u64() % n as u64) as u32;
        let v = (r.next_u64() % n as u64) as u32;
        let w = if r.next_u64() % 3 == 0 { 1.0 } else { 0.0 };
        edges.push((u, v, w));
    }
    edges
}

/// One V1 run (original graph, no reduction) with forced (t, k) = (2, 2) —
/// the min-slack regime of the recorded experiment.
fn run(n: usize, edges: &[(u32, u32, f64)], trace: Option<u32>) -> (Vec<u32>, Vec<String>) {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    let (t, k) = (2u64, 2u64);
    let log_n = (n.max(2) as f64).log2().max(1.0);
    let l_top = (log_n / t as f64).ceil() as u64;
    let mut solver = Solver::new(&adj, t, k);
    solver.trace_vertex = trace;
    solver.d[0] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: 0,
        pred: -1,
    });
    let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
    assert!(b_final == Label::Inf && !ds_top.non_empty());
    (settle, solver.trace_events)
}

fn main() {
    let n = 4094usize;
    let edges = graph(n, 6, 0x0010_C0DE + 4094);

    let (settle, _) = run(n, &edges, None);
    let mut first_pos = vec![usize::MAX; n];
    let mut target = None;
    for (i, &v) in settle.iter().enumerate() {
        if first_pos[v as usize] != usize::MAX {
            target = Some((v, first_pos[v as usize], i));
            break;
        }
        first_pos[v as usize] = i;
    }
    let Some((v, p1, p2)) = target else {
        println!("no duplicate settlements in the log ({} events)", settle.len());
        return;
    };
    println!(
        "first duplicated vertex: v{v} (log positions {p1} and {p2}; log length {})",
        settle.len()
    );
    println!();

    let (_, events) = run(n, &edges, Some(v));
    for e in &events {
        println!("{e}");
    }
}
