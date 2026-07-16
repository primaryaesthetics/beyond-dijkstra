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
            #[cfg(feature = "verify-contract")]
            {
                solver.oracle_len = Some(oracle.clone());
            }
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
                let rc = &solver.restr;
                println!(
                    "  [{tag:>5}]           restricted: unsettled base/rec/iter={}/{}/{}  incomplete base/rec/iter={}/{}/{}",
                    rc.unsettled_base,
                    rc.unsettled_rec,
                    rc.unsettled_iter,
                    rc.incomplete_base,
                    rc.incomplete_rec,
                    rc.incomplete_iter
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
        let (_, v5_regr, v5_dups, v5_ex) = &runs[1];
        let (_, v8_regr, v8_dups, _) = &runs[2];
        assert_eq!(*v8_dups, 0, "V8 must have zero dups");
        println!(
            "{:>7} {:>7} {:>8} {:>7} {:>7.3} {:>8} {:>8} {:>8}({})  paper-ex: {}  V5-ex: {}",
            l_path,
            e_a,
            n,
            dups,
            *dups as f64 / n as f64,
            regr,
            v8_regr,
            v5_regr,
            v5_dups,
            ex,
            v5_ex
        );
    }
    println!();
    staggered();
    println!();
    reuse();
}

/// Part 2: the staggered multi-trigger escape — R rounds of (phase-A
/// block of E values, then a trigger planted at p_r), so waves fire
/// sequentially and cannot batch-merge unless the coarsening law forces
/// it: the block must outgrow the live pull capacity M ≈ (consumed
/// events)/(t²·2^t), which rises by L per round. The coarsening
/// optimization predicts a dup/n ceiling of about t·2^((t+1)/2)
/// (≈ 5.7 at t = 2); blowing past it would refute the law.
fn staggered() {
    println!(
        "{:>7} {:>4} {:>7} {:>9} {:>9} {:>9} {:>8} {:>9} {:>8} {:>8}",
        "L", "R", "E", "n", "paper", "V5", "V5/n", "V8", "p-regr", "v5-regr"
    );
    for &(l_path, r_trig, e_block, t, k) in &[
        (20_000usize, 3usize, 20_000usize, 2u64, 2u64),
        (20_000, 8, 20_000, 2, 2),
        (20_000, 8, 60_000, 2, 2),
        (20_000, 16, 20_000, 2, 2),
        (20_000, 16, 20_000, 4, 2),
        (20_000, 64, 5_000, 4, 2),
    ] {
        let (n, edges) = staggered_graph(l_path, r_trig, e_block);
        let oracle = dijkstra_lengths(n, &edges, 0);
        let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
        for &(u, v, w) in &edges {
            adj[u as usize].push((v, w));
        }
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;

        let mut out: Vec<(usize, u64)> = Vec::new();
        for &(_tag, fixed_base, settled_dedup) in
            &[("paper", false, false), ("V5", true, false), ("V8", false, true)]
        {
            let mut solver = Solver::new(&adj, t, k);
            solver.fixed_base = fixed_base;
            solver.settled_dedup = settled_dedup;
            #[cfg(feature = "verify-contract")]
            {
                solver.oracle_len = Some(oracle.clone());
            }
            solver.d[0] = Label::Fin(Lab {
                len: 0.0,
                hops: 0,
                curr: 0,
                pred: -1,
            });
            let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
            assert!(b_final == Label::Inf && !ds_top.non_empty(), "top partial");
            let dist: Vec<f64> = solver
                .d
                .iter()
                .map(|lab| match lab {
                    Label::Fin(x) => x.len,
                    Label::Inf => f64::INFINITY,
                })
                .collect();
            assert_eq!(dist, oracle, "distances diverged (staggered)");
            let mut seen = vec![false; n];
            let mut dups = 0usize;
            for &v in &settle {
                if seen[v as usize] {
                    dups += 1;
                }
                seen[v as usize] = true;
            }
            #[cfg(feature = "verify-contract")]
            {
                let rc = &solver.restr;
                println!(
                    "  [{_tag:>5}] R={r_trig} E={e_block}: restricted unsettled base/rec/iter={}/{}/{}  incomplete base/rec/iter={}/{}/{}",
                    rc.unsettled_base,
                    rc.unsettled_rec,
                    rc.unsettled_iter,
                    rc.incomplete_base,
                    rc.incomplete_rec,
                    rc.incomplete_iter
                );
            }
            out.push((dups, solver.dup_class.bprime_regressions));
        }
        assert_eq!(out[2].0, 0, "V8 must have zero dups");
        println!(
            "{:>7} {:>4} {:>7} {:>9} {:>9} {:>9} {:>8.3} {:>9} {:>8} {:>8}  t={t}",
            l_path,
            r_trig,
            e_block,
            n,
            out[0].0,
            out[1].0,
            out[1].0 as f64 / n as f64,
            out[2].0,
            out[0].1,
            out[1].1,
        );
    }
}

/// Part 3: the reusable-ballast attack on the gap-capacity lemma. Each
/// shared ballast vertex has q in-edges from chunk-spaced early sources,
/// offering values in q DIFFERENT trigger gaps in descending order — if
/// the entries survived independently, one vertex would fill q gaps and
/// the coarsening ceiling would gain a δ-factor. The closure argument
/// predicts collapse: climb consolidation merges the same key into one D,
/// keep-min dedup erases all but the smallest offer, and relax-≤ forbids
/// post-settlement re-plants — so only the LOWEST gap gets real entries
/// and the upper triggers batch-merge.
fn reuse() {
    println!(
        "{:>7} {:>4} {:>7} {:>3} {:>9} {:>9} {:>9} {:>8} {:>8}",
        "L", "R", "E", "q", "n", "paper", "V5", "waves", "dup/n"
    );
    for &(l_path, r_trig, e_block, q) in &[
        (20_000usize, 8usize, 20_000usize, 8usize),
        (20_000, 8, 20_000, 4),
        (20_000, 8, 20_000, 1), // fresh-equivalent control at reuse n
    ] {
        let (n, edges) = reuse_graph(l_path, r_trig, e_block, q);
        let oracle = dijkstra_lengths(n, &edges, 0);
        let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
        for &(u, v, w) in &edges {
            adj[u as usize].push((v, w));
        }
        let (t, k) = (2u64, 2u64);
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;

        let mut out: Vec<usize> = Vec::new();
        for &(_tag, fixed_base, settled_dedup) in
            &[("paper", false, false), ("V5", true, false), ("V8", false, true)]
        {
            let mut solver = Solver::new(&adj, t, k);
            solver.fixed_base = fixed_base;
            solver.settled_dedup = settled_dedup;
            #[cfg(feature = "verify-contract")]
            {
                solver.oracle_len = Some(oracle.clone());
            }
            solver.d[0] = Label::Fin(Lab {
                len: 0.0,
                hops: 0,
                curr: 0,
                pred: -1,
            });
            let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
            assert!(b_final == Label::Inf && !ds_top.non_empty(), "top partial");
            let dist: Vec<f64> = solver
                .d
                .iter()
                .map(|lab| match lab {
                    Label::Fin(x) => x.len,
                    Label::Inf => f64::INFINITY,
                })
                .collect();
            assert_eq!(dist, oracle, "distances diverged (reuse)");
            let mut seen = vec![false; n];
            let mut dups = 0usize;
            for &v in &settle {
                if seen[v as usize] {
                    dups += 1;
                }
                seen[v as usize] = true;
            }
            #[cfg(feature = "verify-contract")]
            {
                let rc = &solver.restr;
                println!(
                    "  [{_tag:>5}] q={q}: restricted unsettled base/rec/iter={}/{}/{}  incomplete base/rec/iter={}/{}/{}",
                    rc.unsettled_base,
                    rc.unsettled_rec,
                    rc.unsettled_iter,
                    rc.incomplete_base,
                    rc.incomplete_rec,
                    rc.incomplete_iter
                );
            }
            out.push(dups);
        }
        assert_eq!(out[2], 0, "V8 must have zero dups");
        println!(
            "{:>7} {:>4} {:>7} {:>3} {:>9} {:>9} {:>9} {:>8.2} {:>8.3}",
            l_path,
            r_trig,
            e_block,
            q,
            n,
            out[0],
            out[1],
            out[0] as f64 / l_path as f64,
            out[0] as f64 / n as f64,
        );
    }
}

/// Shared-ballast staggered graph: q early sources u_1..u_q (chunk-spaced
/// via 9 spacer leaves each), E ballast vertices with q in-edges offering
/// descending values across the top q trigger gaps; gaps below R−q+1 get
/// no ballast (they rely on the shared vertices' final entries).
fn reuse_graph(
    l_path: usize,
    r_trig: usize,
    e_block: usize,
    q: usize,
) -> (usize, Vec<(u32, u32, f64)>) {
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let path0 = 11u32;
    for g in 0..8u32 {
        edges.push((0, 2 + g, 0.6 + 0.02 * g as f64));
    }
    edges.push((0, 10, 10.0));
    edges.push((10, path0, 1.0));
    for i in 0..(l_path as u32 - 1) {
        edges.push((path0 + i, path0 + i + 1, 1.0));
    }
    let mut id = path0 + l_path as u32;
    // Early sources with chunk-sized spacer blocks between them.
    let mut src: Vec<(u32, f64)> = Vec::new();
    let mut lab = 0.8f64;
    for _ in 0..q {
        edges.push((0, id, lab));
        src.push((id, lab));
        id += 1;
        for _ in 0..9 {
            lab += 0.002;
            edges.push((0, id, lab));
            id += 1;
        }
        lab += 0.002;
    }
    // Trigger values A_g = v0 + g·(E+10); gap g slots at +10+j.
    let v0 = (l_path + 12) as f64;
    let stride = (e_block + 10) as f64;
    // Decoys: trigger g planted at p_g.
    let decoy0 = id;
    for g in 1..=r_trig {
        let d = decoy0 + (g - 1) as u32;
        let eps = 0.01 * g as f64;
        edges.push((0, d, eps));
        edges.push((d, path0 + (g - 1) as u32, (v0 + g as f64 * stride) - eps));
    }
    id += r_trig as u32;
    // Shared ballast: vertex b_j offered gap (R−i+1) slot j by source u_i,
    // i = 1..q — descending values as sources settle in order.
    for j in 0..e_block {
        let b = id;
        id += 1;
        for (i, &(u, ulab)) in src.iter().enumerate() {
            // Source i (settling i-th) offers the gap below trigger R−i:
            // values in (A_{R−1−i}, A_{R−i}) — descending across sources.
            let gap = r_trig - 1 - i;
            let val = v0 + gap as f64 * stride + 10.0 + j as f64;
            edges.push((u, b, val - ulab));
        }
    }
    (id as usize, edges)
}

/// R rounds: block_r of E leaf values, then trigger value A_r planted at
/// path vertex p_r (one decoy per target, planted while d[p_r] = ∞).
fn staggered_graph(l_path: usize, r_trig: usize, e_block: usize) -> (usize, Vec<(u32, u32, f64)>) {
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let path0 = 11u32;
    // Decoys d_r (labels 0.01·r, settle first) and fillers to close their
    // chunks before the path gate e (label 10) settles.
    for g in 0..8u32 {
        edges.push((0, 2 + g, 0.6 + 0.02 * g as f64));
    }
    edges.push((0, 10, 10.0));
    edges.push((10, path0, 1.0));
    for i in 0..(l_path as u32 - 1) {
        edges.push((path0 + i, path0 + i + 1, 1.0));
    }
    let mut id = path0 + l_path as u32;
    let decoy0 = id;
    id += r_trig as u32;
    let mut val = (l_path + 12) as f64;
    for r in 0..r_trig {
        // phase block r
        for g in 0..e_block {
            edges.push((0, id, val + g as f64));
            id += 1;
        }
        val += e_block as f64;
        // trigger r at value A_r = val, targeting p_{r+1}
        let d = decoy0 + r as u32;
        let eps = 0.01 * (r + 1) as f64;
        edges.push((0, d, eps));
        edges.push((d, path0 + r as u32, val - eps));
        val += 10.0;
    }
    (id as usize, edges)
}
