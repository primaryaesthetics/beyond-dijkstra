//! Q14 row J — the pred-slot scope probes and the lh-oracle face
//! (QUESTIONS.md addendum 27's design, executed with the corrections
//! recorded in addendum 30).
//!
//! Addendum 27 designed face (a) as "the drift face": a two-pred diamond
//! whose smaller-id tight pred is gated until after the target settles,
//! predicted to exhibit settle-time pred != final pred. The construction
//! attempts (addendum 30) closed that face NEGATIVE against the audited
//! machinery: a pred-improving candidate sits full-tuple below its
//! target's stored value, so every bound that admits the target's
//! settlement admits the candidate (the refusal-floor argument is
//! slot-free), the candidate's source settles strictly earlier (Fact Z)
//! and its settlement echo delivers the offer under the first covering
//! bound before the target settles there, and W' settlers are
//! walked-at-final (the drop-lemma corollary), so co-mate sweeps carry
//! nothing new. On V5 — and on the paper engine in the first-violation-
//! free regime — the settlement-time label is bit-final, pred included.
//! The two rows here are the nearest misses, run as exhibits:
//!
//! Shared graph (super-source 0): 0 -> u_small(v1) w=50.5;
//! 0 -> u_big(v2) w=50.25; u_small -> y(v3) w=0.25; u_big -> y w=0.5.
//! Both y-paths are tight at (len, hops) = (50.75, 2); the canonical
//! pred is u_small = v1 (min id). Direct level-1 calls, t=2, k=3,
//! B = len 52; pre-labeled vertices carry their oracle labels.
//!
//! - Row J1 (the unarrived offer): S = [u_big]; u_small is pre-labeled
//!   complete but NOT in S — no search roots at it, nothing ever relaxes
//!   into it, so its tight offer never fires. u_big's FindPivots search
//!   fails (|K| = 2 < 3) after labeling y at (50.75, h2, v3, p2); no
//!   pivots exist, the loop runs zero iterations, B' = B, and
//!   W' = [u_big, y]: y settles at the boundary append carrying
//!   pred = u_big — the min id among ARRIVED tight offers, not the
//!   canonical argmin. The W' sweep re-fires u_big -> y as an exact-tie
//!   re-store (paper/V5; V8 refuses it at the settled-dedup gate).
//! - Row J2 (the arrival-order face, S2's accept live): same graph,
//!   S = [u_big, u_small] — the larger-id search runs FIRST, so y is
//!   labeled at pred u_big and then pred-improved to u_small by the
//!   second search BEFORE any settlement: the pred-only tie-break
//!   accept, live in the trace. W' = [u_big, y, u_small]; y settles
//!   already carrying the canonical label. The u_big -> y sweep re-fire
//!   is label-refused (pred 2 > 1); u_small -> y is an exact-tie
//!   re-store (paper/V5 only, as above).
//!
//! Predictions, written before the first run (all three engines
//! identical except the traced post-settlement tie re-stores, which V8's
//! dedup gate refuses):
//! - J1: B' = 52 (= B, zero iterations), |U| = 2; settle-inc = 0,
//!   viol base/rec = 0/0, cap = 0, incomp/stale/wards/missing all 0,
//!   dups = 0; y settle-time label == final label == (50.75, h2, v3,
//!   p2) — pred u_big, NOT canonical; u_small: label (50.5, h1, v1,
//!   p0) untouched, unsettled, no entry. Trace (vertex y): one
//!   find_pivots RELAX-ACCEPT from u=2 (old INF), SETTLE #1 [W'
//!   boundary append], then on paper/V5 only an [exact-tie re-store]
//!   at w_prime_sweep from u=2.
//! - J2: B' = 52, |U| = 3; same zero counters; y settle-time == final
//!   == (50.75, h2, v3, p1) — canonical, pred u_small. Trace: two
//!   find_pivots RELAX-ACCEPTs (u=2 old INF, then u=1 with cand p1
//!   over old p2 — the pred-only improvement), SETTLE #1 [W' boundary
//!   append], then on paper/V5 only an [exact-tie re-store] at
//!   w_prime_sweep from u=1.
//! - Part 2 (full recursion, source 0, both S-shapes share the graph):
//!   distances EXACT on every engine, settle-inc = 0, wards/missing =
//!   0, dups = 0; y's final label = (50.75, h2, v3, p1) — canonical
//!   including pred on ALL engines (the pred improvement arrives before
//!   y's pull, so even V8 accepts it).
//!
//! Face (b) — the lh-oracle (addendum 27's design, unchanged): a
//! driver-side hops-aware Dijkstra (lex-min (len, hops)) against the
//! final labels across the standard families (zero-heavy minslack
//! n=4094 and n=16382 at (t,k)=(2,2); tie-rich 0..3 n=170000;
//! zero-heavy large n=500000). Predictions: reach and (len, hops)
//! mismatches = 0 on paper and V5 (for V5 a consequence of (H_lh | V5)
//! plus (Floor) plus labels-only-improve; for paper the corpus's
//! zero-counter regime), and final pred == the canonical tight pred
//! (min id u with lh(u) + edge == lh(v)) on paper and V5 (every settled
//! vertex's edges are re-swept at the top call's infinite bound, and a
//! label-refusal of the canonical offer forces pred equality — the
//! refusal-floor argument at the pred slot). V8 columns are RECORDED,
//! not predicted: its dedup gate freezes labels at settlement, so
//! post-settlement hops- or pred-healing accepts are refused and
//! mismatches are possible within its distance-exact contract.
//!
//! ```text
//! cargo run --release --features verify-contract,dup-classify,q11-trace --example rowj
//! ```

use beyond_dijkstra::label::{Lab, Label};
use beyond_dijkstra::solver::{choose_params, Solver};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

/// Same generator as `verify_contract.rs` so face (b) runs the recorded
/// corpus's inputs.
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

/// Lexicographic (len, hops) heap key; lens are finite and non-negative.
#[derive(Clone, Copy, PartialEq)]
struct LhKey(f64, u64);

impl Eq for LhKey {}

impl Ord for LhKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .total_cmp(&other.0)
            .then_with(|| self.1.cmp(&other.1))
    }
}

impl PartialOrd for LhKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Driver-side hops-aware Dijkstra: per-vertex lex-min (len, hops) over
/// paths from `source`; `None` = unreached.
fn lh_dijkstra(n: usize, adj: &[Vec<(u32, f64)>], source: usize) -> Vec<Option<(f64, u64)>> {
    let mut lh: Vec<Option<(f64, u64)>> = vec![None; n];
    let mut heap: BinaryHeap<Reverse<(LhKey, u32)>> = BinaryHeap::new();
    lh[source] = Some((0.0, 0));
    heap.push(Reverse((LhKey(0.0, 0), source as u32)));
    while let Some(Reverse((LhKey(len, hops), u))) = heap.pop() {
        if lh[u as usize] != Some((len, hops)) {
            continue; // stale
        }
        for &(v, w) in &adj[u as usize] {
            let cand = (len + w, hops + 1);
            let better = match lh[v as usize] {
                None => true,
                Some(cur) => LhKey(cand.0, cand.1) < LhKey(cur.0, cur.1),
            };
            if better {
                lh[v as usize] = Some(cand);
                heap.push(Reverse((LhKey(cand.0, cand.1), v)));
            }
        }
    }
    lh
}

/// Canonical tight pred per reached non-source vertex: the min id `u`
/// with an edge (u, v, w) satisfying lh(u) + (w, 1) == lh(v) exactly.
fn canonical_preds(
    n: usize,
    edges: &[(u32, u32, f64)],
    lh: &[Option<(f64, u64)>],
    source: usize,
) -> Vec<Option<i64>> {
    let mut pred: Vec<Option<i64>> = vec![None; n];
    for &(u, v, w) in edges {
        let (Some((ul, uh)), Some((vl, vh))) = (lh[u as usize], lh[v as usize]) else {
            continue;
        };
        if ul + w == vl && uh + 1 == vh {
            let cand = u as i64;
            if pred[v as usize].map_or(true, |p| cand < p) {
                pred[v as usize] = Some(cand);
            }
        }
    }
    pred[source] = None;
    pred
}

fn adj_of(n: usize, edges: &[(u32, u32, f64)]) -> Vec<Vec<(u32, f64)>> {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u as usize].push((v, w));
    }
    adj
}

fn fmt(l: Label) -> String {
    match l {
        Label::Fin(l) => format!("({}, h{}, v{}, p{})", l.len, l.hops, l.curr, l.pred),
        Label::Inf => "INF".into(),
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

const U_SMALL: u32 = 1;
const U_BIG: u32 = 2;
const Y: u32 = 3;

fn diamond_edges() -> Vec<(u32, u32, f64)> {
    vec![
        (0, U_SMALL, 50.5),
        (0, U_BIG, 50.25),
        (U_SMALL, Y, 0.25),
        (U_BIG, Y, 0.5),
    ]
}

const ENGINES: [(&str, bool, bool); 3] =
    [("paper", false, false), ("V5", true, false), ("V8", false, true)];

/// One direct level-1 call per engine with the q11 trace on y.
fn run_direct(name: &str, s_list: &[u32]) {
    let edges = diamond_edges();
    let n = 4usize;
    let adj = adj_of(n, &edges);
    let lh = lh_dijkstra(n, &adj, 0);
    let oracle: Vec<f64> = lh
        .iter()
        .map(|l| l.map_or(f64::INFINITY, |(len, _)| len))
        .collect();
    println!("Row {name}:");
    for (ename, fixed, dedup) in ENGINES {
        let mut solver = Solver::new(&adj, 2, 3);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        solver.trace_vertex = Some(Y);
        for &s in s_list {
            solver.d[s as usize] = Label::Fin(Lab {
                len: oracle[s as usize],
                hops: 1,
                curr: s,
                pred: 0,
            });
        }
        if !s_list.contains(&U_SMALL) {
            // J1: complete but outside S — its offer never arrives.
            solver.d[U_SMALL as usize] = Label::Fin(Lab {
                len: oracle[U_SMALL as usize],
                hops: 1,
                curr: U_SMALL,
                pred: 0,
            });
        }
        let (bp, u_log, ds) = solver.bmssp(B_CALL, s_list, 1);
        let ec = &solver.escort;
        let rc = &solver.restr;
        let dc = &solver.dup_class;
        let dups = dc.base_seeded
            + dc.base_inserted
            + dc.wprime_own_root
            + dc.wprime_own_swept
            + dc.wprime_foreign;
        println!(
            "  [{ename:>5}] B'={} |U|={} settle-inc={} viol base/rec={}/{} cap={} incomp b/r/i={}/{}/{} stale-inc={}/{}/{} wards b/r/i={}/{}/{} missing={}/{}/{} dups={}",
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
        );
        println!(
            "          y final={} (canonical pred p{U_SMALL}); u_small: d={} in-U={} entry={}",
            fmt(solver.d[Y as usize]),
            fmt(solver.d[U_SMALL as usize]),
            u_log.contains(&U_SMALL),
            ds.get(U_SMALL)
                .map_or("none".into(), |l| fmt(Label::Fin(l))),
        );
        for e in &solver.trace_events {
            println!("          | {}", e.replace('\n', "\n          | "));
        }
    }
    println!();
}

/// Full recursion from the super-source on the diamond graph.
fn run_full_diamond() {
    let edges = diamond_edges();
    let n = 4usize;
    let adj = adj_of(n, &edges);
    let lh = lh_dijkstra(n, &adj, 0);
    let oracle: Vec<f64> = lh
        .iter()
        .map(|l| l.map_or(f64::INFINITY, |(len, _)| len))
        .collect();
    println!("Part 2 -- full recursion on the diamond, source 0, all engines:");
    for (ename, fixed, dedup) in ENGINES {
        let mut solver = Solver::new(&adj, 2, 3);
        solver.fixed_base = fixed;
        solver.settled_dedup = dedup;
        solver.oracle_len = Some(oracle.clone());
        solver.d[0] = Label::Fin(Lab {
            len: 0.0,
            hops: 0,
            curr: 0,
            pred: -1,
        });
        let (b_final, _settle, ds_top) = solver.bmssp(Label::Inf, &[0], 1);
        assert!(
            b_final == Label::Inf && !ds_top.non_empty(),
            "top partial ({ename})"
        );
        let dist: Vec<f64> = solver.d.iter().map(|&l| label_len(l)).collect();
        assert_eq!(dist, oracle, "distances diverged ({ename})");
        let dc = &solver.dup_class;
        let dups = dc.base_seeded
            + dc.base_inserted
            + dc.wprime_own_root
            + dc.wprime_own_swept
            + dc.wprime_foreign;
        println!(
            "  [{ename:>5}] settle-inc={} dups={} y final={} (exact)",
            solver.settle_incomplete,
            dups,
            fmt(solver.d[Y as usize]),
        );
    }
    println!();
}

/// Face (b): final labels vs the lh oracle and canonical preds.
fn lh_oracle_sweep() {
    println!("Face (b) -- lh-oracle across the standard families:");
    println!(
        "{:>28} {:>8} {:>6} {:>8} {:>10} {:>10} {:>10}",
        "input", "n", "engine", "reach", "len-mism", "hops-mism", "pred-mism"
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
        let (t, k) = match ov {
            Some(tk) => tk,
            None => {
                let (_, t, k, _) = choose_params(n, edges.len(), n);
                (t, k)
            }
        };
        let adj = adj_of(n, &edges);
        let lh = lh_dijkstra(n, &adj, 0);
        let cpred = canonical_preds(n, &edges, &lh, 0);
        let log_n = (n.max(2) as f64).log2().max(1.0);
        let l_top = (log_n / t as f64).ceil() as u64;
        for (ename, fixed, dedup) in ENGINES {
            let mut solver = Solver::new(&adj, t, k);
            solver.fixed_base = fixed;
            solver.settled_dedup = dedup;
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
            let mut reach = 0u64;
            let mut len_mism = 0u64;
            let mut hops_mism = 0u64;
            let mut pred_mism = 0u64;
            for v in 0..n {
                match (lh[v], solver.d[v]) {
                    (None, Label::Inf) => {}
                    (None, Label::Fin(_)) | (Some(_), Label::Inf) => reach += 1,
                    (Some((len, hops)), Label::Fin(l)) => {
                        if l.len != len {
                            len_mism += 1;
                        } else if l.hops != hops {
                            hops_mism += 1;
                        } else if v != 0 && Some(l.pred) != cpred[v] {
                            pred_mism += 1;
                        }
                    }
                }
            }
            println!(
                "{:>28} {:>8} {:>6} {:>8} {:>10} {:>10} {:>10}",
                name, n, ename, reach, len_mism, hops_mism, pred_mism
            );
        }
    }
}

fn main() {
    println!("Row J -- pred-slot scope exhibits (direct level-1 calls, t=2, k=3, B=len 52):");
    println!();
    run_direct("J1: the unarrived offer (final pred = min ARRIVED, not canonical)", &[U_BIG]);
    run_direct("J2: the arrival-order face (pre-settlement pred-only accept, canonical)", &[U_BIG, U_SMALL]);
    run_full_diamond();
    lh_oracle_sweep();
}
