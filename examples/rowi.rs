//! Q14 addendum 22 probe — row I stage 1: the assembly attempts. The
//! two-stage protocol (addendum 20) predicted stage 1 — the leak alone,
//! viol_bound_rec >= 1 under V5 with settle_incomplete = 0 — to be
//! assemblable from addendum 18's recipe (tree-shadow member outside W
//! dropped below the final child's returned bound, pivot dying in the
//! final iteration, cutoff exit). This session's construction attempts
//! (the route tree, QUESTIONS.md addendum 22) each closed against an
//! audited mechanism: in-child drops are minted into custody by the
//! one-step complementarity (the in-grandchild refusal cand >= b_i IS the
//! parent sweep's insert gate), base-grade drops ride the V5 cap to
//! equality, pre-pull drops are expanded down (Rep D3), cutoff exits drag
//! B' to at-or-below the dropped value (the refusal bound is the returned
//! bound), and the one entry-less slot — the post-filter W' sweep —
//! requires its source improved after its last pop, whose improvement
//! chain grounds out either in minted custody or in a stale-at-filter W'
//! pass, which is settle_incomplete >= 1 — the (d2) breach itself. The
//! two rows here are the nearest misses, run as exhibits of the walls.
//!
//! Shared core (direct level-1 call, t = 2, k = 3, B = len 52; seeds
//! pre-labeled at oracle values, so S is complete — Lemma 3.7's
//! precondition): q1@50.25 ->0.8125-> w_s (true 51.0625) ->0.75-> v
//! (true 51.8125). q1's FindPivots search pops q1 and w_s, walks v at
//! 51.8125 < 52, and caps at |K| = 3: tree {q1, w_s, v}, P = {q1} — v is
//! the tree-shadow member, outside W, outside S, complete and entry-less
//! before the loop. Fillers f1@50.5, f2@51.5, g@51.75 (2-leaf stars) pin
//! the separators.
//!
//! - Row R1 (custody discharge, n = 13): iteration 1 pulls {q1, f1} at
//!   separator f2@51.5; the base child mints w_s@51.0625 (< 51.5) and
//!   settles it by pull, refusing w_s -> v in-child (51.8125 >= 51.5);
//!   the parent sweep then tie-accepts w_s -> v and the complementary
//!   gate 51.8125 >= 51.5 INSERTS v — the mint, live. Iteration 2 pulls
//!   {f2, g} at separator v; iteration 3 pulls v itself: settled
//!   complete by pull. Drain exit at B' = B = 52.
//! - Row R2 (the equality wall, R1 + 8 ballast stars at 15..18.5,
//!   n = 37): iterations 1-4 burn ballast (24 settles), iteration 5
//!   replays the mint (|U| = 29), iteration 6 pulls {f2, g} at separator
//!   v (|U| = 35 > 32) — cutoff exit whose last child returns exactly
//!   v's tuple as its bound. The returned D is {v@51.8125} with
//!   B' = 51.8125: dB[D] >= B' holds AT EQUALITY — the strict leak is
//!   grazed, not breached, and v returns unsettled, complete,
//!   entry-at-bound: the S2 custody shape, harmless.
//!
//! Predictions, written before the first run (all three engines
//! identical on both rows — every base child drains, so the V5 cap is a
//! no-op, and no refused re-offer to a settled vertex changes any
//! outcome under V8):
//! - R1: B' = 52 (= B, drain exit), |U| = 12; settle-inc = 0,
//!   viol base/rec = 0/0, cap = 0, incomp/stale/wards/missing all 0,
//!   dups = 0; v: d = 51.8125 = true, in-U = TRUE, entry = none;
//!   w_s: d = 51.0625, in-U = TRUE.
//! - R2: B' = 51.8125, |U| = 35; settle-inc = 0, viol base/rec = 0/0
//!   (equality, by strictness of the counter), cap = 0, incomp/stale/
//!   wards/missing all 0, dups = 0; v: d = 51.8125 = true, in-U =
//!   FALSE, entry = 51.8125 == B'; w_s: d = 51.0625, in-U = TRUE.
//! - Part 2 (full recursion, source 0, scoped per the wprime
//!   precedent): distances EXACT on every engine, settle-inc = 0,
//!   wards/missing = 0, dups = 0; viol/cap recorded, not predicted.
//!
//! ```text
//! cargo run --release --features verify-contract,dup-classify --example rowi
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::Solver;

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
    assert!(ok.is_ok(), "oracle rejected the rowi graph");
    dist
}

fn adj_of(n: usize, edges: &[(u32, u32, f64)]) -> Vec<Vec<(u32, f64)>> {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    adj
}

/// Crafted-call graph builder (wprime.rs pattern): vertex 0 is the
/// super-source; `seed` adds a vertex with a 0-edge at its intended
/// label, so the direct call and the full-recursion part share one graph
/// and one oracle.
struct Craft {
    edges: Vec<(u32, u32, f64)>,
    n: u32,
    seeds: Vec<u32>,
}

impl Craft {
    fn new() -> Self {
        Craft {
            edges: Vec::new(),
            n: 1,
            seeds: Vec::new(),
        }
    }
    fn vertex(&mut self) -> u32 {
        let v = self.n;
        self.n += 1;
        v
    }
    fn seed(&mut self, label: f64) -> u32 {
        let v = self.vertex();
        self.edges.push((0, v, label));
        self.seeds.push(v);
        v
    }
    fn edge(&mut self, u: u32, v: u32, w: f64) {
        self.edges.push((u, v, w));
    }
    /// 2-leaf star pivot seeds at `start + 0.5*i`: each search caps at
    /// exactly k = 3 and the root becomes its group's pivot.
    fn ballast(&mut self, count: usize, start: f64) {
        for i in 0..count {
            let g = self.seed(start + 0.5 * i as f64);
            for j in 0..2 {
                let h = self.vertex();
                self.edge(g, h, 0.125 * (j + 1) as f64);
            }
        }
    }
}

fn seed_labels(solver: &mut Solver, craft: &Craft, oracle: &[f64]) {
    for &s in &craft.seeds {
        solver.d[s as usize] = Label::Fin(Lab {
            len: oracle[s as usize],
            hops: 1,
            curr: s,
            pred: 0,
        });
    }
}

fn len_of(l: Option<Lab>) -> String {
    match l {
        Some(l) => format!("{}", l.len),
        None => "-".into(),
    }
}

fn label_len(l: Label) -> f64 {
    match l {
        Label::Fin(x) => x.len,
        Label::Inf => f64::INFINITY,
    }
}

const B_CALL: Label = Label::Fin(Lab {
    len: 52.0,
    hops: 0,
    curr: 0,
    pred: -1,
});

/// The shared core: q1 -> w_s -> v plus the separator fillers. Returns
/// (q1, w_s, v).
fn core(c: &mut Craft) -> (u32, u32, u32) {
    let q1 = c.seed(50.25);
    let w_s = c.vertex();
    c.edge(q1, w_s, 0.8125); // w_s true 51.0625
    let v = c.vertex();
    c.edge(w_s, v, 0.75); // v true 51.8125
    let f1 = c.seed(50.5);
    for j in 0..2 {
        let h = c.vertex();
        c.edge(f1, h, 0.0625 * (j + 1) as f64);
    }
    let f2 = c.seed(51.5);
    for j in 0..2 {
        let h = c.vertex();
        c.edge(f2, h, 0.0625 * (j + 1) as f64);
    }
    let g = c.seed(51.75);
    let h1 = c.vertex();
    c.edge(g, h1, 0.03125);
    let h2 = c.vertex();
    c.edge(g, h2, 0.046875);
    (q1, w_s, v)
}

fn row_r1() -> (Craft, u32, u32) {
    let mut c = Craft::new();
    let (_q1, w_s, v) = core(&mut c);
    (c, w_s, v)
}

fn row_r2() -> (Craft, u32, u32) {
    let mut c = Craft::new();
    c.ballast(8, 15.0);
    let (_q1, w_s, v) = core(&mut c);
    (c, w_s, v)
}

/// One direct level-1 call per engine, wprime.rs pattern.
fn run_direct(
    craft: &Craft,
    name: &'static str,
    inspect: impl Fn(&Solver, Label, &[u32], &beyond_dijkstra::solver::BlockDs) -> String,
) {
    let n = craft.n as usize;
    let oracle = dijkstra_lengths(n, &craft.edges, 0);
    let adj = adj_of(n, &craft.edges);
    println!("Row {name}:");
    for (ename, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)]
    {
        let mut solver = Solver::new(&adj, 2, 3);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        seed_labels(&mut solver, craft, &oracle);
        let (bp, u_log, ds) = solver.bmssp(B_CALL, &craft.seeds, 1);
        let ec = &solver.escort;
        let rc = &solver.restr;
        let dc = &solver.dup_class;
        let dups = dc.base_seeded
            + dc.base_inserted
            + dc.wprime_own_root
            + dc.wprime_own_swept
            + dc.wprime_foreign;
        println!(
            "  [{ename:>5}] B'={} |U|={} settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{} dups={}  {}",
            label_len(bp),
            u_log.len(),
            solver.settle_incomplete,
            solver.viol_bound_base,
            solver.viol_bound_rec,
            solver.base_cap_hits,
            rc.incomplete_base,
            rc.incomplete_rec,
            rc.incomplete_iter,
            rc.stale_incomplete_base,
            rc.stale_incomplete_rec,
            rc.stale_incomplete_iter,
            ec.wards_base,
            ec.wards_rec,
            ec.wards_iter,
            ec.missing_base,
            ec.missing_rec,
            ec.missing_iter,
            dups,
            inspect(&solver, bp, &u_log, &ds),
        );
    }
    println!();
}

/// Full recursion from the super-source, all engines; asserts exactness.
fn run_full(craft: &Craft, name: &str) {
    let n = craft.n as usize;
    let oracle = dijkstra_lengths(n, &craft.edges, 0);
    let adj = adj_of(n, &craft.edges);
    let t = 2u64;
    let log_n = (n.max(2) as f64).log2().max(1.0);
    let l_top = (log_n / t as f64).ceil() as u64;
    println!("  full recursion ({name}, n={n}, l_top={l_top}):");
    for (ename, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)]
    {
        let mut solver = Solver::new(&adj, t, 3);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        solver.d[0] = Label::Fin(Lab {
            len: 0.0,
            hops: 0,
            curr: 0,
            pred: -1,
        });
        let (b_final, _settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
        assert!(
            b_final == Label::Inf && !ds_top.non_empty(),
            "top partial ({name}, {ename})"
        );
        let dist: Vec<f64> = solver.d.iter().map(|&l| label_len(l)).collect();
        assert_eq!(dist, oracle, "distances diverged ({name}, {ename})");
        let ec = &solver.escort;
        let rc = &solver.restr;
        let dc = &solver.dup_class;
        let dups = dc.base_seeded
            + dc.base_inserted
            + dc.wprime_own_root
            + dc.wprime_own_swept
            + dc.wprime_foreign;
        println!(
            "    [{ename:>5}] settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{} dups={} (exact)",
            solver.settle_incomplete,
            solver.viol_bound_base,
            solver.viol_bound_rec,
            solver.base_cap_hits,
            rc.incomplete_base,
            rc.incomplete_rec,
            rc.incomplete_iter,
            rc.stale_incomplete_base,
            rc.stale_incomplete_rec,
            rc.stale_incomplete_iter,
            ec.wards_base,
            ec.wards_rec,
            ec.wards_iter,
            ec.missing_base,
            ec.missing_rec,
            ec.missing_iter,
            dups,
        );
    }
}

fn main() {
    println!("Row I stage 1 — the assembly walls (direct level-1 calls, t=2, k=3, B=len 52):");
    println!();

    let (c1, ws1, v1) = row_r1();
    run_direct(&c1, "R1: custody discharge (the complementarity mint, pulled)", move |s, _bp, u, ds| {
        format!(
            "v: d={} true={} in-U={} entry={}; w_s: d={} in-U={}",
            label_len(s.d[v1 as usize]),
            s.oracle_len.as_ref().unwrap()[v1 as usize],
            u.contains(&v1),
            len_of(ds.get(v1)),
            label_len(s.d[ws1 as usize]),
            u.contains(&ws1),
        )
    });

    let (c2, ws2, v2) = row_r2();
    run_direct(&c2, "R2: the equality wall (cutoff exit, dB[D] == B')", move |s, bp, u, ds| {
        format!(
            "v: d={} true={} in-U={} entry={} (B'={}); w_s: d={} in-U={}",
            label_len(s.d[v2 as usize]),
            s.oracle_len.as_ref().unwrap()[v2 as usize],
            u.contains(&v2),
            len_of(ds.get(v2)),
            label_len(bp),
            label_len(s.d[ws2 as usize]),
            u.contains(&ws2),
        )
    });

    println!("Part 2 -- full recursion, source 0, all engines:");
    run_full(&c1, "row R1");
    run_full(&c2, "row R2");
}
