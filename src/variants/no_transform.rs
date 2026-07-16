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
use crate::solver::{choose_params, PhaseTimes, Solver};

pub struct VariantRun {
    pub delta: u64,
    pub t: u64,
    pub k: u64,
    pub l_top: u64,
    pub dist: Vec<f64>,
    /// Per-phase wall clock (all zeros unless built with `phase-timer`).
    pub phase_times: PhaseTimes,
    /// Total wall clock of the run (zero unless built with `phase-timer`).
    pub total: f64,
    /// Settlement events in the top-level log minus distinct vertices —
    /// the Q10 re-settlement count the V5 experiment reports.
    pub dup_settlements: usize,
    /// Base-case returns the V5 cap actually lowered (0 unless V5).
    pub base_cap_hits: u64,
    /// Q14 duplicate-settlement classification (feature `dup-classify`).
    #[cfg(feature = "dup-classify")]
    pub dup_class: crate::solver::DupClassCounts,
    /// Restricted-invariant counters (Q14 item 1 pre-test; the exact
    /// `incomplete_*` half needs the runner's oracle argument).
    #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
    pub restr: crate::solver::RestrCounts,
    /// Direct (H) counter: settlement events at a non-true label length
    /// (needs the runner's oracle argument; see `Solver::settle_incomplete`).
    #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
    pub settle_incomplete: u64,
    /// Escort-invariant counters (Q14 addendum 10; needs the runner's
    /// oracle argument — see `crate::solver::EscortCounts`).
    #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
    pub escort: crate::solver::EscortCounts,
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
    sssp_run_inner(n, edges, source, 0, false, false, override_params, None)
}

/// V3 (VARIANTS.md): V1 with levels ≤ `hybrid_level` running the base
/// case at the level's own cutoff. `hybrid_level` = 0 is V1 itself;
/// ≥ l_top makes the whole run one bounded tuple-label Dijkstra.
pub fn sssp_hybrid(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> VariantRun {
    sssp_run_inner(n, edges, source, hybrid_level, false, false, override_params, None)
}

/// V5 (VARIANTS.md): like `sssp_hybrid`, with the fixed base case
/// (B′ capped at dB[D] — the Lemma 3.1 contract restoration).
pub fn sssp_v5(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> VariantRun {
    sssp_run_inner(n, edges, source, hybrid_level, true, false, override_params, None)
}

/// V8 (VARIANTS.md): like `sssp_hybrid`, with the per-vertex settled bit
/// (settled vertices are never re-adopted, re-frontiered, or re-settled).
pub fn sssp_v8(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> VariantRun {
    sssp_run_inner(n, edges, source, hybrid_level, false, true, override_params, None)
}

/// Restricted-invariant pre-test runner (QUESTIONS.md Q14 item 1): any of
/// the engines above, plus an exact-completeness oracle (true shortest-path
/// lengths per vertex) feeding the `incomplete_*` counters. Observation
/// only — distances are unchanged.
#[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
#[allow(clippy::too_many_arguments)]
pub fn sssp_pretest(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    fixed_base: bool,
    settled_dedup: bool,
    override_params: Option<(u64, u64, u64)>,
    oracle_len: Vec<f64>,
) -> VariantRun {
    sssp_run_inner(
        n,
        edges,
        source,
        hybrid_level,
        fixed_base,
        settled_dedup,
        override_params,
        Some(oracle_len),
    )
}

#[allow(clippy::too_many_arguments)]
fn sssp_run_inner(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    fixed_base: bool,
    settled_dedup: bool,
    override_params: Option<(u64, u64, u64)>,
    // Restricted-invariant oracle; only read under verify-contract +
    // dup-classify (None and unused otherwise).
    #[allow(unused_variables)] oracle_len: Option<Vec<f64>>,
) -> VariantRun {
    #[cfg(feature = "phase-timer")]
    let total_t0 = std::time::Instant::now();
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
    solver.hybrid_level = hybrid_level;
    solver.fixed_base = fixed_base;
    solver.settled_dedup = settled_dedup;
    #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
    {
        solver.oracle_len = oracle_len;
    }
    solver.d[source] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: source as u32,
        pred: -1,
    });
    let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[source as u32], l_top);
    // Totality assert (QUESTIONS.md Q10) — same as the mainline entry:
    // a partial top-level execution would return silently wrong
    // distances; fail loudly instead.
    assert!(
        b_final == Label::Inf && !ds_top.non_empty(),
        "top-level BMSSP exited partially (re-settlement volume exceeded \
         the |U| cutoff) — see QUESTIONS.md Q10"
    );
    let mut seen = vec![false; n];
    let mut dup_settlements = 0usize;
    for &v in &settle {
        if seen[v as usize] {
            dup_settlements += 1;
        }
        seen[v as usize] = true;
    }

    let dist = solver
        .d
        .iter()
        .map(|l| match l {
            Label::Fin(l) => l.len,
            Label::Inf => f64::INFINITY,
        })
        .collect();
    #[cfg(feature = "phase-timer")]
    let total = total_t0.elapsed().as_secs_f64();
    #[cfg(not(feature = "phase-timer"))]
    let total = 0.0;
    VariantRun {
        delta,
        t,
        k,
        l_top,
        dist,
        phase_times: solver.phase.clone(),
        total,
        dup_settlements,
        base_cap_hits: solver.base_cap_hits,
        #[cfg(feature = "dup-classify")]
        dup_class: solver.dup_class.clone(),
        #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
        restr: solver.restr.clone(),
        #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
        settle_incomplete: solver.settle_incomplete,
        #[cfg(all(feature = "verify-contract", feature = "dup-classify"))]
        escort: solver.escort.clone(),
    }
}
