//! Q14 addendum 10 follow-up: the W′ endgame probes — try to build the escort
//! invariant's regeneration case 4 (a W′-settled escort whose canonical
//! successor falls below the governing bound, so the accepted offer completes
//! the successor but the insert gates create no entry) and record which repair
//! mechanism blocks each attempt.
//!
//! The five direct-call rows, one mechanism each (level-1 call, t = 2,
//! B = len 1000; seeds pre-labeled as if pulled from a parent, super-source 0
//! carries the same labels for the oracle and the full-recursion part):
//!
//! - Row A (clean pocket, k = 3): a failed FindPivots search whose labels are
//!   exact enumerates its whole sub-B region, so the entire pocket W′-settles
//!   complete — closure, nothing left behind.
//! - Row B (stale pocket healed at FindPivots, k = 4): a later FAILED search
//!   improves the pocket's stale labels and, because failed searches pop
//!   everything, re-relaxes the successor chain in place — everything is
//!   complete before the loop starts.
//! - Row C (capped improver, root stranded, k = 3): the improving search hits
//!   its |K| = k cap exactly on the stale vertex z, so z becomes complete
//!   with NO entry and its successor x1 is never re-relaxed (the pocket's
//!   stale candidate crossed B, hiding a true value below it). The case-4
//!   configuration needs dis(z) < B′ — but z's true path runs through the
//!   tree root r, and r's own fresh entry is never pulled, so the pull
//!   separators cap B′ <= d[r] < dis(z): z is NOT in W′ and the leak cannot
//!   fire. z is returned complete/entry-less/unsettled — the custody object
//!   (L1), repaired at full recursion by r's line-33 premature return.
//! - Row D (improver pulled, k = 3): same shape with r cheap enough to be
//!   pulled; the base child re-walks the chain by exact-tie re-stores
//!   (line 23) and settles z by pull; z's successor offer is then refused
//!   inside the child (cand >= b_i) but accepted at the parent sweep with the
//!   complementary gate d >= b_i inserting — x1 ends complete WITH an entry.
//! - Row E (the first live recursive-boundary ward, k = 3): a base-exit
//!   triangle planted in the last child before the parent's |U| limit exit,
//!   so the leaked pair survives merge to the parent's return. wards_rec
//!   fires for the first time on any input, and the escort (the triangle's
//!   z_t, complete, fresh, co-resident) is found: missing_rec = 0.
//!
//! Predictions written before the first run (per engine paper/V5/V8):
//! rows A-D identical across engines — wards/missing all 0, settle-inc 0;
//! A: x, z W′-settled complete; B: r, x, z, x1 W′-settled complete;
//! C: B′ = 21 <= d[r] = 21.75 < dis(z) = 22.75, z complete/no-entry/
//! unsettled, x1 unlabeled; D: z settled by pull, x1 complete with returned
//! entry at 990; E (paper, V8): B′ = 21, wards_iter = wards_rec = 1,
//! missing = 0, viol base/rec = 1/1, settle-inc 0; E (V5): base cap fires,
//! B′ = 20.84375, wards 0. Part 2 (full recursion on the same graphs):
//! distances exact on every engine, settle-inc 0, missing 0 everywhere;
//! wards 0 on rows A-D; row E wards fire at base/iter on paper and V8.
//!
//! ```text
//! cargo run --release --features verify-contract,dup-classify --example wprime
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
    assert!(ok.is_ok(), "oracle rejected the wprime graph");
    dist
}

fn adj_of(n: usize, edges: &[(u32, u32, f64)]) -> Vec<Vec<(u32, f64)>> {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    adj
}

/// Crafted-call graph builder: vertex 0 is the super-source; `seed` adds a
/// vertex with an 0-edge at its intended label, so the direct level-1 call
/// (seeds pre-labeled at hops 1, pred 0) and the full-recursion run share
/// one graph and one oracle.
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
    /// `count` ballast pivot seeds at `start + 0.5*i`, each a (k-1)-leaf star
    /// so its FindPivots search caps at exactly k and the root is a pivot.
    fn ballast(&mut self, count: usize, start: f64, leaves: usize) -> Vec<u32> {
        let mut roots = Vec::new();
        for i in 0..count {
            let g = self.seed(start + 0.5 * i as f64);
            for j in 0..leaves {
                let h = self.vertex();
                self.edge(g, h, 0.125 * (j + 1) as f64);
            }
            roots.push(g);
        }
        roots
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
    len: 1000.0,
    hops: 0,
    curr: 0,
    pred: -1,
});

struct RowResult {
    name: &'static str,
    engines: Vec<String>,
}

/// One direct level-1 call per engine; returns the per-engine report lines
/// produced by `inspect(solver, b_prime, u_log, ds)`.
fn run_direct(
    craft: &Craft,
    k: u64,
    name: &'static str,
    inspect: impl Fn(&Solver, Label, &[u32], &beyond_dijkstra::solver::BlockDs) -> String,
) -> RowResult {
    let n = craft.n as usize;
    let oracle = dijkstra_lengths(n, &craft.edges, 0);
    let adj = adj_of(n, &craft.edges);
    let mut engines = Vec::new();
    for (ename, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)]
    {
        let mut solver = Solver::new(&adj, 2, k);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        seed_labels(&mut solver, craft, &oracle);
        let (bp, u_log, ds) = solver.bmssp(B_CALL, &craft.seeds, 1);
        let ec = &solver.escort;
        let rc = &solver.restr;
        let line = format!(
            "  [{ename:>5}] B'={} |U|={} settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{}  {}",
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
            inspect(&solver, bp, &u_log, &ds),
        );
        engines.push(line);
    }
    RowResult { name, engines }
}

/// Full recursion from the super-source on the same graph, all engines;
/// asserts distance exactness and prints the counter set.
fn run_full(craft: &Craft, k: u64, name: &str) {
    let n = craft.n as usize;
    let oracle = dijkstra_lengths(n, &craft.edges, 0);
    let adj = adj_of(n, &craft.edges);
    let t = 2u64;
    let log_n = (n.max(2) as f64).log2().max(1.0);
    let l_top = (log_n / t as f64).ceil() as u64;
    println!("  full recursion ({name}, n={n}, l_top={l_top}):");
    for (ename, fixed, dedup) in [("paper", false, false), ("V5", true, false), ("V8", false, true)]
    {
        let mut solver = Solver::new(&adj, t, k);
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
        println!(
            "    [{ename:>5}] settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{} (exact)",
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
        );
    }
}

/// Row A: clean pocket. x@20 -> z (+0.5), z a sink; ballast 20 pivots at
/// 15..24.5 (2-leaf stars, k = 3). Six children of 6 settles each exceed the
/// level-1 limit 32; B' = 21. The pocket {x, z} fails FindPivots with exact
/// labels and W'-settles complete.
fn row_a() -> (Craft, u64) {
    let mut c = Craft::new();
    let x = c.seed(20.0);
    let z = c.vertex();
    c.edge(x, z, 0.5);
    c.ballast(20, 15.0, 2);
    (c, 3)
}

/// Row B: stale pocket healed at FindPivots (k = 4). x@20 ->10-> z (stale 30),
/// z ->2-> x1 (stale 32), x1 a sink; r@15 ->1-> z (true z = 16, x1 = 18).
/// Both searches FAIL (3 < 4): x's plants the stale pocket, r's improves and
/// POPS z, re-relaxing x1 in place. Ballast 3-leaf stars at 15.5..25;
/// 5 children x 8 settles exceed 32; B' = 20.5; everything below W'-settles
/// complete.
fn row_b() -> (Craft, u64, u32, u32) {
    let mut c = Craft::new();
    let x = c.seed(20.0);
    let z = c.vertex();
    let x1 = c.vertex();
    c.edge(x, z, 10.0);
    c.edge(z, x1, 2.0);
    let r = c.seed(15.0);
    c.edge(r, z, 1.0);
    c.ballast(20, 15.5, 3);
    (c, 4, z, x1)
}

/// Row C: capped improver, root stranded (k = 3). x@20 ->10-> z (stale 30);
/// z ->975-> x1 (stale cand 1005 >= B refused at FindPivots; true
/// 22.75 + 975 = 997.75 < B hidden); r@21.75 ->0.5-> c1 ->0.5-> z: r's search
/// caps at |K| = 3 exactly on z — z complete at 22.75, never popped, no
/// entry. Ballast as row A; limit exit after 6 children, B' = 21, r@21.75
/// never pulled: B' <= d[r] < dis(z), so z is NOT in W' and case 4 cannot
/// assemble. z returns complete/entry-less/unsettled; x1 stays unlabeled.
fn row_c() -> (Craft, u64, u32, u32, u32) {
    let mut c = Craft::new();
    let x = c.seed(20.0);
    let z = c.vertex();
    let x1 = c.vertex();
    c.edge(x, z, 10.0);
    c.edge(z, x1, 975.0);
    let r = c.seed(21.75);
    let c1 = c.vertex();
    c.edge(r, c1, 0.5);
    c.edge(c1, z, 0.5);
    c.ballast(20, 15.0, 2);
    (c, 3, z, x1, r)
}

/// Row D: improver pulled, complementary gates (k = 3). Same shape as row C
/// with r@14 pulled in the first batch (separator 16): the base child
/// re-walks r -> c1 -> z by exact-tie re-stores and settles z by pull; the
/// child refuses z -> x1 (990 >= 16) but the parent sweep accepts it and the
/// d >= b_i gate INSERTS x1@990 into the parent D. Ballast at 15.5..25;
/// B' = 21.
fn row_d() -> (Craft, u64, u32, u32) {
    let mut c = Craft::new();
    let x = c.seed(20.0);
    let z = c.vertex();
    let x1 = c.vertex();
    c.edge(x, z, 10.0);
    c.edge(z, x1, 975.0);
    let r = c.seed(14.0);
    let c1 = c.vertex();
    c.edge(r, c1, 0.5);
    c.edge(c1, z, 0.5);
    c.ballast(20, 15.5, 2);
    (c, 3, z, x1)
}

/// Row E: the recursive-boundary ward (k = 3). Ballast as row A, except the
/// 12th pivot g11@20.5 carries a 5-chain (+0.0625 steps) ending in u_last
/// whose post-pull sweep plants the base-exit triangle (z_t at +0.03125,
/// y_t at +0.09375 direct / +0.015625 via z_t) exactly at the child's cutoff
/// exit: bound already fixed at b = 21, both inserts leak below it, and the
/// parent's limit exit freezes them into the returned D. wards_rec fires for
/// the first time; the escort z_t is complete, fresh, co-resident: missing 0.
/// Layout: 11 standard 2-leaf ballast pivots at 15..20, then g11@20.5 with
/// the chain, then 8 more standard pivots at 21..24.5 so the separator
/// sequence stays intact.
fn row_e() -> (Craft, u64, u32, u32) {
    let mut c = Craft::new();
    for i in 0..11 {
        let g = c.seed(15.0 + 0.5 * i as f64);
        for j in 0..2 {
            let h = c.vertex();
            c.edge(g, h, 0.125 * (j + 1) as f64);
        }
    }
    let g11 = c.seed(20.5);
    let mut prev = g11;
    for _ in 0..4 {
        let h = c.vertex();
        c.edge(prev, h, 0.0625);
        prev = h;
    }
    let u_last = c.vertex();
    c.edge(prev, u_last, 0.0625); // u_last at 20.8125
    let z_t = c.vertex();
    let y_t = c.vertex();
    c.edge(u_last, z_t, 0.03125); // z_t at 20.84375
    c.edge(u_last, y_t, 0.09375); // y_t direct 20.90625, true 20.859375
    c.edge(z_t, y_t, 0.015625);
    for i in 12..20 {
        let g = c.seed(15.0 + 0.5 * i as f64);
        for j in 0..2 {
            let h = c.vertex();
            c.edge(g, h, 0.125 * (j + 1) as f64);
        }
    }
    (c, 3, z_t, y_t)
}

fn main() {
    println!("W' endgame probes (direct level-1 calls, t=2, B=len 1000):");
    println!();

    // -- Row A ------------------------------------------------------------
    let (ca, ka) = row_a();
    let za = ca.seeds[0] + 1; // z is the vertex right after x
    let ra = run_direct(&ca, ka, "A: clean pocket", move |s, _bp, u, ds| {
        format!(
            "z: d={} settledlist={} entry={}",
            label_len(s.d[za as usize]),
            u.contains(&za),
            len_of(ds.get(za)),
        )
    });
    println!("Row {}:", ra.name);
    for l in &ra.engines {
        println!("{l}");
    }
    println!();

    // -- Row B ------------------------------------------------------------
    let (cb, kb, zb, x1b) = row_b();
    let rb = run_direct(&cb, kb, "B: pocket healed at FindPivots", move |s, _bp, u, _ds| {
        format!(
            "z: d={} in-U={}; x1: d={} in-U={}",
            label_len(s.d[zb as usize]),
            u.contains(&zb),
            label_len(s.d[x1b as usize]),
            u.contains(&x1b),
        )
    });
    println!("Row {}:", rb.name);
    for l in &rb.engines {
        println!("{l}");
    }
    println!();

    // -- Row C ------------------------------------------------------------
    let (cc, kc, zc, x1c, rc) = row_c();
    let rrc = run_direct(&cc, kc, "C: capped improver, root stranded", move |s, bp, u, ds| {
        format!(
            "z: d={} in-U={} entry={}; x1: d={}; r: d={} entry={} (B'={} <= d[r])",
            label_len(s.d[zc as usize]),
            u.contains(&zc),
            len_of(ds.get(zc)),
            label_len(s.d[x1c as usize]),
            label_len(s.d[rc as usize]),
            len_of(ds.get(rc)),
            label_len(bp),
        )
    });
    println!("Row {}:", rrc.name);
    for l in &rrc.engines {
        println!("{l}");
    }
    println!();

    // -- Row D ------------------------------------------------------------
    let (cd, kd, zd, x1d) = row_d();
    let rd = run_direct(&cd, kd, "D: root pulled, complementary gates", move |s, _bp, u, ds| {
        format!(
            "z: d={} in-U={}; x1: d={} entry={}",
            label_len(s.d[zd as usize]),
            u.contains(&zd),
            label_len(s.d[x1d as usize]),
            len_of(ds.get(x1d)),
        )
    });
    println!("Row {}:", rd.name);
    for l in &rd.engines {
        println!("{l}");
    }
    println!();

    // -- Row E ------------------------------------------------------------
    let (ce, ke, zt, yt) = row_e();
    let re = run_direct(&ce, ke, "E: recursive-boundary ward", move |s, _bp, _u, ds| {
        format!(
            "y_t: d={} true={} entry={}; z_t: d={} entry={}",
            label_len(s.d[yt as usize]),
            s.oracle_len.as_ref().unwrap()[yt as usize],
            len_of(ds.get(yt)),
            label_len(s.d[zt as usize]),
            len_of(ds.get(zt)),
        )
    });
    println!("Row {}:", re.name);
    for l in &re.engines {
        println!("{l}");
    }
    println!();

    // -- Part 2: full recursion on the same graphs -------------------------
    println!("Part 2 -- full recursion, source 0, all engines:");
    run_full(&ca, ka, "row A");
    run_full(&cb, kb, "row B");
    run_full(&cc, kc, "row C");
    run_full(&cd, kd, "row D");
    run_full(&ce, ke, "row E");
}
