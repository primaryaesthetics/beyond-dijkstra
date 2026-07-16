//! Q15 experiment: force the wave-ESCAPE regime and watch the progress
//! measure B′ regress (QUESTIONS.md Q14 addendum 4 named this as unproved
//! item (i); this driver decides it).
//!
//! Construction: the run's value range has three phases. The settled path
//! p_1..p_L (labels 11..L+10) is phase one; phase A (~E_A leaf values above
//! the path) drags the climbing leftover D up to a mid-level call whose
//! CHILD budget t³·2^((j−1)t) is far below L; the single decoy trigger sits
//! at value A just above phase A, so it is pulled exactly there. The
//! re-settlement wave (length ≈ L) cannot fit in the child, the child exits
//! partial, and its returned B′ = the wave front's small value arrives at a
//! call whose B′ has already progressed to A-scale: a B′ regression, with
//! Lemma 3.8's per-iteration containment {dis(U_i)} ⊆ [B′_{i−1}, B′_i)
//! witnessed on an EMPTY interval.
//!
//! Every run is distance-checked against the crate's Dijkstra. Build with
//! `dup-classify` (counters); add `verify-contract` to also see the
//! return-time bound-clause counters on the same run.
//!
//! ```text
//! cargo run --release --features dup-classify,verify-contract --example escape
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::Solver;

/// (n, edges). Vertex ids: 0 = s, 1 = d (decoy), 2..=9 = fillers,
/// 10 = e (path gate), 11..=L+10 = path, then phase A, then phase B.
fn escape_graph(l_path: usize, e_a: usize, e_b: usize) -> (usize, Vec<(u32, u32, f64)>) {
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let path0 = 11u32; // first path vertex id
    let a_val0 = (l_path + 12) as f64; // phase-A values start above the path
    let trigger = a_val0 + e_a as f64 + 50.0; // decoy offer value A

    // Decoy first (label 0.5, settles in the first base chunk while
    // d[p_1] = ∞), fillers 0.6..0.74 close that chunk before e settles.
    edges.push((0, 1, 0.5));
    edges.push((1, path0, trigger - 0.5));
    for g in 0..8u32 {
        edges.push((0, 2 + g, 0.6 + 0.02 * g as f64));
    }
    // Path gate e (label 10) and the unit path: dis(p_i) = 10 + i.
    edges.push((0, 10, 10.0));
    edges.push((10, path0, 1.0));
    for i in 0..(l_path as u32 - 1) {
        edges.push((path0 + i, path0 + i + 1, 1.0));
    }
    // Phase A: consumed before the trigger fires; sets the climb level.
    let mut id = path0 + l_path as u32;
    for g in 0..e_a {
        edges.push((0, id, a_val0 + g as f64));
        id += 1;
    }
    // Phase B: values above the trigger, so the run continues past it.
    for g in 0..e_b {
        edges.push((0, id, trigger + 100.0 + g as f64));
        id += 1;
    }
    (id as usize, edges)
}

fn dijkstra_lengths(n: usize, edges: &[(u32, u32, f64)], source: usize) -> Vec<f64> {
    let mut counts = vec![0i64; n + 1];
    for &(u, _, _) in edges {
        counts[u as usize + 1] += 1;
    }
    for i in 0..n {
        counts[i + 1] += counts[i];
    }
    let mut indices = vec![0i32; edges.len()];
    let mut weights = vec![0f64; edges.len()];
    let mut next = counts.clone();
    for &(u, v, w) in edges {
        let slot = next[u as usize] as usize;
        indices[slot] = v as i32;
        weights[slot] = w;
        next[u as usize] += 1;
    }
    let mut dist = vec![0f64; n];
    let mut pred = vec![0i32; n];
    let mut heap = beyond_dijkstra::dijkstra::Heap::new();
    let ok = beyond_dijkstra::dijkstra::dijkstra(
        &counts, &indices, &weights, source, &mut dist, &mut pred, &mut heap,
    );
    assert!(ok.is_ok(), "oracle rejected the escape graph");
    dist
}

fn main() {
    println!(
        "{:>7} {:>7} {:>8} {:>7} {:>7} {:>8} {:>8}  examples (level, B' before -> after, |U_i|)",
        "L", "E_A", "n", "dups", "dup/n", "B'-regr", "v8-regr"
    );
    // (L, E_A): the wave escapes when L exceeds the child budget of the
    // call holding the trigger — position that call via E_A. Control rows
    // keep the wave contained (small L or the trigger firing high).
    for &(l_path, e_a) in &[
        (200usize, 3000usize), // contained: wave 200 « child budget
        (5000, 3000),          // escape regime: wave 5000 » budget 2048
        (5000, 40),            // trigger fires low, D still low: contained?
        (20000, 3000),         // longer wave, same climb level
    ] {
        let (n, edges) = escape_graph(l_path, e_a, 500);
        let oracle = dijkstra_lengths(n, &edges, 0);

        let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
        for &(u, v, w) in &edges {
            adj[u as usize].push((v, w));
        }
        let (t, k) = (2u64, 2u64);
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;

        let mut runs: Vec<(&str, u64, usize, String)> = Vec::new();
        for &(tag, fixed_base, settled_dedup) in
            &[("paper", false, false), ("V5", true, false), ("V8", false, true)]
        {
            let mut solver = Solver::new(&adj, t, k);
            solver.fixed_base = fixed_base;
            solver.settled_dedup = settled_dedup;
            solver.d[0] = Label::Fin(Lab {
                len: 0.0,
                hops: 0,
                curr: 0,
                pred: -1,
            });
            let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
            assert!(
                b_final == Label::Inf && !ds_top.non_empty(),
                "top-level partial execution at L={l_path}"
            );
            let dist: Vec<f64> = solver
                .d
                .iter()
                .map(|lab| match lab {
                    Label::Fin(x) => x.len,
                    Label::Inf => f64::INFINITY,
                })
                .collect();
            assert_eq!(dist, oracle, "distances diverged at L={l_path}");
            let mut seen = vec![false; n];
            let mut dups = 0usize;
            for &v in &settle {
                if seen[v as usize] {
                    dups += 1;
                }
                seen[v as usize] = true;
            }
            let ex: Vec<String> = solver
                .dup_class
                .regression_examples
                .iter()
                .map(|&(lv, b, a, u)| format!("(l{lv} {b:.0}->{a:.0} |U|={u})"))
                .collect();
            #[cfg(feature = "verify-contract")]
            {
                // The paper base case CAN violate its bound clause (Q3);
                // this family exercises it in the wild. Recursive-return
                // violations surviving under V5 would be the line-29
                // channel (Q12) live on its own.
                println!(
                    "  [{tag:>5}] L={l_path}: viol_bound_base={} viol_bound_rec={} viol_dichotomy={} cap_hits={} dups={dups}",
                    solver.viol_bound_base,
                    solver.viol_bound_rec,
                    solver.viol_dichotomy,
                    solver.base_cap_hits
                );
            }
            runs.push((
                tag,
                solver.dup_class.bprime_regressions,
                dups,
                ex.join(" "),
            ));
        }
        let (_, regr, dups, ex) = &runs[0];
        let (_, v8_regr, v8_dups, _) = &runs[2];
        assert_eq!(*v8_dups, 0, "V8 must have zero dups");
        println!(
            "{:>7} {:>7} {:>8} {:>7} {:>7.3} {:>8} {:>8}  {}",
            l_path,
            e_a,
            n,
            dups,
            *dups as f64 / n as f64,
            regr,
            v8_regr,
            ex
        );
    }
}
