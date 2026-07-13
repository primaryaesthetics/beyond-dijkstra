//! V1 — no-degree-reduction: run the solver directly on the original
//! graph, skipping the §2.1 zero-weight-cycle substitution.
//!
//! Class 2 (distance-preserving, structure-changing); the claim, the
//! degree-bound audit, and the falsification-review record are in
//! VARIANTS.md. Port constraint from that review: no assertion of the
//! paper's degree-dependent size bounds (|D| ≤ |S| + δ|U|, |K| ≤ 2k,
//! p ≤ |S|, base-case |D| ≤ t⁴) may exist on this path — the solver only
//! carries the degree-free |U| cutoffs.

use crate::label::{Lab, Label};
use crate::solver::{choose_params, Solver};

pub struct VariantRun {
    pub delta: u64,
    pub t: u64,
    pub k: u64,
    pub l_top: u64,
    pub dist: Vec<f64>,
}

/// Shortest-path lengths from `source` on the original graph. Parameters
/// follow the mainline recipe with the original n as the working vertex
/// count (δ survives only inside the t-formula — VARIANTS.md V1);
/// `override_params` forces (delta, t, k) exactly as in the mainline.
pub fn sssp(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    override_params: Option<(u64, u64, u64)>,
) -> VariantRun {
    let (delta, t, k, l_top) = match override_params {
        None => choose_params(n, edges.len(), n),
        Some((df, tf, kf)) => {
            let log_n = (n.max(2) as f64).log2().max(1.0);
            (df, tf, kf, (log_n / tf as f64).ceil() as u64)
        }
    };

    // Adjacency in input edge order per vertex — the same relaxation-order
    // pin the mainline uses for cross edges (no cycle edges exist here).
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }

    let mut solver = Solver::new(&adj, t, k);
    solver.d[source] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: source as u32,
        pred: -1,
    });
    solver.bmssp(Label::Inf, &[source as u32], l_top);

    let dist = solver
        .d
        .iter()
        .map(|l| match l {
            Label::Fin(l) => l.len,
            Label::Inf => f64::INFINITY,
        })
        .collect();
    VariantRun {
        delta,
        t,
        k,
        l_top,
        dist,
    }
}
