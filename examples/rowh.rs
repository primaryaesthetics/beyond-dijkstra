//! Q14 addendum 15 probe — row H: the V8 wrong-distance embedding. Row G
//! (examples/wprime.rs) falsified Lemma 3.7's completeness clause at
//! direct-call grade: a W-walked vertex re-improved by a cutoff-exiting base
//! child to a still-incomplete value below B′ passes the W′ filter and is
//! settled incomplete. On row G's graph the leak does not embed in full
//! recursion — the failed seed f sits at 995, so by the time f enters any
//! call's S the target is settled complete and the stale walk is refused.
//! Row H makes f VALUE-ADJACENT: the whole leaking configuration is packed
//! into one mid-recursion S-batch, so the incomplete W′ settlement fires
//! inside a full run. The paper engine then heals through the duplicate
//! channel; V8's settled bit blocks every repair relax and the run ends
//! with a WRONG FINAL DISTANCE — the first distance-incorrect V8 run on any
//! input. V5's base cap closes the leak before it settles anything.
//!
//! Construction (t = 2, k = 3, one graph, source 0, n = 70, l_top = 4):
//!
//! - Phase A: 32 sink seeds at 1.0..16.5 (0.5 steps). The first level-1
//!   call (S = {0}, limit 32) consumes exactly these: base child 1 settles
//!   0 + a1..a8 (9 pulls, cutoff exit at separator a9 = 5), then 12 sink
//!   pairs take |U| to 33 > 32 — limit exit at B′ = 20 with every phase-B
//!   key still unsettled in the returned D.
//! - The batch: the level-2 call's second pull (M = 8) extracts exactly
//!   {p1..p6, g11, f} — six 3-leaf ballast stars at 20..22.5 (leaf steps
//!   +0.0625/+0.125/+0.1875), the chain root g11 at 23, and f at 23.75.
//!   The 9th-smallest key, the sink gap at 30, sets the child bound
//!   B_C* = 30.
//! - Inside C* (level 1, S = the batch, B = 30): f's FindPivots search
//!   accepts the stale offer f ->4-> y_t at 27.75 < 30 (y_t unlabeled — the
//!   chain is never walked before this call), y_t is a sink, the search
//!   fails at |K| = 2 < 3: W = {f, y_t}. The six stars cap at k and become
//!   pivots; g11's search caps two steps into the chain. Children: three
//!   star pairs of 8 settles each (|U| = 24), then g11 is pulled ALONE
//!   (|D| = 1), so the pull empties D and the base child's bound is
//!   B_C* = 30. That child walks g11 + h1..h7 + u_last = 9 pulls: the 9th
//!   pull (u_last at 23.5) empties the base D, fixing the returned bound at
//!   30, and u_last's post-pull sweep then inserts z_t at 23.53125 and y_t
//!   at 23.59375 (true 23.546875 via the never-walked z_t) below it — the
//!   Q3 leak. |U| = 33 > 32: C* limit-exits with B′ = 30. At the boundary
//!   y_t ∈ W, y_t ∉ U, d[y_t] = 23.59375 < 30 — settled INCOMPLETE, inside
//!   a full recursion.
//! - Aftermath: C*'s returned D carries z_t and y_t to the level-2 call,
//!   whose next pull hands them to a cleanup call. Paper: z_t's failed
//!   search improves y_t to its true 23.546875 and the W′ boundary settles
//!   it again — a duplicate settlement, distances exact. V8: y_t's settled
//!   bit rejects z_t's repair relax and the pull drops y_t's stale entry —
//!   d[y_t] = 23.59375 forever. V5: the base cap fixes the chain child's
//!   bound at dB[D] = 23.53125, so B′_C* = 23.53125, y_t never passes the
//!   W′ filter, f is re-inserted by line 33, and the cleanup call settles
//!   everything complete.
//!
//! Predictions, written before the first run:
//! - all engines: totality holds (top-level B′ = ∞, top D empty).
//! - paper: distances EXACT; settle-inc = 1 (y_t in C*); viol base/rec =
//!   1/1; cap = 0; incomp b/r/i = 1/1/2; stale-inc = 0/0/0; wards b/r/i =
//!   1/1/2; missing = 0/0/0; dups = 1 (y_t re-settled complete in the
//!   cleanup call); y_t: d = 23.546875, settlements = 2.
//! - V8: y_t is the ONLY wrong vertex: d = 23.59375 vs true 23.546875
//!   (err 0.046875); settle-inc = 1; viol base/rec = 1/1; cap = 0;
//!   incomp b/r/i = 1/1/2; stale-inc = 0/0/0; wards b/r/i = 1/1/2;
//!   missing = 0/0/0; dups = 0; y_t settlements = 1.
//! - V5: distances EXACT; settle-inc = 0; viol base/rec = 0/0; cap = 1;
//!   incomp/stale/wards/missing all 0; dups = 0; y_t settlements = 1.
//!
//! ```text
//! cargo run --release --features verify-contract,dup-classify --example rowh
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
    assert!(ok.is_ok(), "oracle rejected the rowh graph");
    dist
}

fn adj_of(n: usize, edges: &[(u32, u32, f64)]) -> Vec<Vec<(u32, f64)>> {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    adj
}

struct Craft {
    edges: Vec<(u32, u32, f64)>,
    n: u32,
}

impl Craft {
    fn new() -> Self {
        Craft {
            edges: Vec::new(),
            n: 1,
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
        v
    }
    fn edge(&mut self, u: u32, v: u32, w: f64) {
        self.edges.push((u, v, w));
    }
}

fn main() {
    let mut c = Craft::new();
    // Phase A: 32 sink seeds, consumed exactly by the first level-1 call.
    for i in 0..32 {
        c.seed(1.0 + 0.5 * i as f64);
    }
    // Batch ballast: six 3-leaf stars at 20..22.5 (8 settles per pair child).
    for i in 0..6 {
        let p = c.seed(20.0 + 0.5 * i as f64);
        for j in 0..3 {
            let h = c.vertex();
            c.edge(p, h, 0.0625 * (j + 1) as f64);
        }
    }
    // The chain root: 8 steps of 0.0625 down to u_last, then the base-exit
    // triangle — 9 pulls, the 9th (u_last) empties the base D before its
    // sweep plants the leak.
    let g11 = c.seed(23.0);
    let mut prev = g11;
    for _ in 0..7 {
        let h = c.vertex();
        c.edge(prev, h, 0.0625);
        prev = h;
    }
    let u_last = c.vertex();
    c.edge(prev, u_last, 0.0625); // u_last at 23.5
    let z_t = c.vertex();
    let y_t = c.vertex();
    c.edge(u_last, z_t, 0.03125); // z_t at 23.53125
    c.edge(u_last, y_t, 0.09375); // y_t direct 23.59375, true 23.546875
    c.edge(z_t, y_t, 0.015625);
    // f, value-adjacent: pulled into the same batch, fails FindPivots after
    // walking y_t at the stale 27.75 (< B_C* = 30).
    let f = c.seed(23.75);
    c.edge(f, y_t, 4.0);
    // The gap key: 9th-smallest at batch time, sets B_C* = 30.
    c.seed(30.0);

    let n = c.n as usize;
    let oracle = dijkstra_lengths(n, &c.edges, 0);
    let adj = adj_of(n, &c.edges);
    let t = 2u64;
    let k = 3u64;
    let log_n = (n.max(2) as f64).log2().max(1.0);
    let l_top = (log_n / t as f64).ceil() as u64;
    println!(
        "Row H — the V8 wrong-distance embedding (full recursion, source 0, \
         n={n}, l_top={l_top}, t={t}, k={k}):"
    );
    println!();

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
        let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[0], l_top);
        assert!(
            b_final == Label::Inf && !ds_top.non_empty(),
            "top partial ({ename})"
        );
        let dist: Vec<f64> = solver
            .d
            .iter()
            .map(|&l| match l {
                Label::Fin(x) => x.len,
                Label::Inf => f64::INFINITY,
            })
            .collect();
        let mismatches: Vec<(usize, f64, f64)> = dist
            .iter()
            .zip(&oracle)
            .enumerate()
            .filter(|(_, (&got, &want))| got != want)
            .map(|(i, (&got, &want))| (i, got, want))
            .collect();
        let yt_settles = settle.iter().filter(|&&v| v == y_t).count();
        let dc = &solver.dup_class;
        let dups = dc.base_seeded
            + dc.base_inserted
            + dc.wprime_own_root
            + dc.wprime_own_swept
            + dc.wprime_foreign;
        let ec = &solver.escort;
        let rc = &solver.restr;
        println!(
            "  [{ename:>5}] settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{} dups={}",
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
        println!(
            "          y_t (v{}): d={} true={} settlements={}",
            y_t,
            dist[y_t as usize],
            oracle[y_t as usize],
            yt_settles,
        );
        if mismatches.is_empty() {
            println!("          distances: EXACT ({n}/{n})");
        } else {
            println!("          distances: WRONG at {} vertex(es):", mismatches.len());
            for &(v, got, want) in &mismatches {
                println!("            v{v}: got {got} want {want} (err {})", got - want);
            }
        }
        println!();
        if ename != "V8" {
            assert!(mismatches.is_empty(), "{ename} diverged from the oracle");
        }
    }
}
