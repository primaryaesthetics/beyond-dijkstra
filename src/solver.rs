//! Faithful Rust port of the reference implementation
//! (`reference/reference.py`) of arXiv:2602.07868.
//!
//! Semantics are pinned by SPEC.md; this port is gated bit-exactly against
//! the reference (final labels + settlement log) by `tests/differential.rs`.
//! The module follows the reference file section by section so the two can
//! be audited side by side; until the gate is green and the mainline is
//! frozen, clarity and determinism outrank speed everywhere. Known
//! deliberate literalisms carry the same QUESTIONS.md tags as the reference
//! (Q1 clamps, Q2 empty P_j, Q3 base-case bound, Q8 pivot death).

use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

use crate::label::{Lab, Label};

// ---------------------------------------------------------------------------
// Phase attribution (feature `phase-timer`; zero overhead when off).

/// Wall-clock seconds per solver phase. All zeros unless the crate is built
/// with `--features phase-timer`. Buckets are non-overlapping except
/// `partition`, which is a subset of `find_pivots` (reported so the local
/// searches and the tree partition can be separated when reading).
#[derive(Clone, Default, Debug)]
pub struct PhaseTimes {
    pub reduce: f64,
    pub find_pivots: f64,
    pub partition: f64,
    pub base_case: f64,
    pub ds_pull: f64,
    pub ds_merge: f64,
    pub pivot_bookkeeping: f64,
    pub relax_sweep: f64,
    pub boundary_sweeps: f64,
}

/// Time `$e` into `$st.phase.$field` when `phase-timer` is on; just `$e`
/// otherwise. `$e` is evaluated exactly once either way.
macro_rules! phase {
    ($st:expr, $field:ident, $e:expr) => {{
        #[cfg(feature = "phase-timer")]
        let __phase_t0 = std::time::Instant::now();
        let __phase_r = $e;
        #[cfg(feature = "phase-timer")]
        {
            $st.phase.$field += __phase_t0.elapsed().as_secs_f64();
        }
        __phase_r
    }};
}

// ---------------------------------------------------------------------------
// Q11 event trace (feature `q11-trace`; zero overhead when off). One chosen
// vertex; every event touching it is recorded with a call-path snapshot so a
// duplicate settlement can be traced end-to-end (QUESTIONS.md Q11).

#[cfg(feature = "q11-trace")]
fn fmt_label(l: Label) -> String {
    match l {
        Label::Fin(l) => format!("({}, h{}, v{}, p{})", l.len, l.hops, l.curr, l.pred),
        Label::Inf => "INF".into(),
    }
}

// ---------------------------------------------------------------------------
// Duplicate-settlement classification (feature `dup-classify`; zero overhead
// when off). Every duplicate settlement event — a vertex re-entering the
// settlement log — is tagged by the channel that emitted it, per the Q14
// addendum's inventory: base-case pulls consume a D entry (seeded into the
// base D from S, or inserted by an in-base tie relaxation), W′ appends need
// none. For foreign-W′ appends (vertex not in the appending call's S) the
// Lemma D prediction is tested on the spot: some pred-chain ancestor within
// k−1 hops must be a failed root (∈ Q) of the same call.

/// Duplicate settlement events by channel. `base_seeded + base_inserted +
/// wprime_own_root + wprime_own_swept + wprime_foreign` equals the
/// top-level log's duplicate count.
#[cfg(feature = "dup-classify")]
#[derive(Clone, Default, Debug)]
pub struct DupClassCounts {
    /// Base-case pull of an already-settled vertex that was in the base
    /// call's seed S (a re-frontiered chain reaching level 0).
    pub base_seeded: u64,
    /// Base-case pull of an already-settled vertex inserted into the base
    /// D by an in-base relaxation (an exact-tie re-store — the Lemma A/B
    /// bottom-interval insert observed directly).
    pub base_inserted: u64,
    /// W′ append of an already-settled vertex in this call's S that is a
    /// failed root (∈ Q) — the Q11 trace's pattern.
    pub wprime_own_root: u64,
    /// W′ append of an already-settled vertex in this call's S swept into
    /// another root's failed arborescence.
    pub wprime_own_swept: u64,
    /// W′ append of an already-settled vertex NOT in this call's S — the
    /// FindPivots channel with no D entry (Lemma D territory).
    pub wprime_foreign: u64,
    /// Foreign appends whose pred-chain ancestor within k−1 hops is a
    /// failed root (∈ Q) of the same call — Lemma D's prediction (exact
    /// on inputs without cross-walk length-and-hops ties, where the
    /// arborescence path is the canonical pred chain).
    pub foreign_anc_in_q: u64,
    /// Weaker containment for the same appends: some pred-chain ancestor
    /// within k−1 hops is in the call's S (Q ⊆ S).
    pub foreign_anc_in_s: u64,
    /// Iterations whose returned B′_i is strictly below the call's
    /// current B′ — regressions of the loop's progress measure
    /// (QUESTIONS.md Q15). The paper's Lemma 3.7/3.8 proofs implicitly
    /// require B′ to be non-decreasing across iterations.
    pub bprime_regressions: u64,
}

// ---------------------------------------------------------------------------
// Parameters (SPEC "Parameters"; clamps per QUESTIONS.md Q1).

/// Mirror of `choose_params`: all logs base 2, `int()` truncation for delta.
/// Returns (delta, t, k, l_top).
pub fn choose_params(n: usize, m: usize, n2: usize) -> (u64, u64, u64, u64) {
    fn loglog(x: usize) -> f64 {
        let inner = (x.max(2) as f64).log2();
        inner.max(2.0).log2().max(1.0)
    }
    let ratio = if n > 0 { m as f64 / n as f64 } else { 0.0 };
    let delta = ((ratio.min(loglog(n)) / 4.0).trunc() as u64).max(3);
    let log_n2 = (n2.max(2) as f64).log2().max(1.0);
    let t = ((log_n2 * loglog(n2) / delta as f64).sqrt().ceil() as u64).max(2);
    let k = ((t as f64 / (t as f64).log2()).ceil() as u64).max(2);
    let l_top = (log_n2 / t as f64).ceil() as u64;
    (delta, t, k, l_top)
}

/// `2^e` saturating at `u64::MAX`; feeds `M(l)` and the `|U|` cutoffs, which
/// only ever gate comparisons against list lengths (the reference computes
/// them as exact big ints, but any value above every reachable length is
/// equivalent).
pub(crate) fn pow2_sat(e: u64) -> u64 {
    if e >= 63 {
        u64::MAX
    } else {
        1u64 << e
    }
}

// ---------------------------------------------------------------------------
// Degree reduction (SPEC "Degree reduction", paper section 2.1).

pub struct Reduced {
    pub n2: usize,
    /// Reduced adjacency: zero-weight cycle edges first (vertex order), then
    /// cross edges in input edge order — the pinned relaxation order.
    pub adj: Vec<Vec<(u32, f64)>>,
    /// Per original vertex: (first reduced id, cycle size); ids contiguous.
    pub cycle_of: Vec<(u32, u32)>,
}

pub fn degree_reduce(n: usize, edges: &[(u32, u32, f64)], delta: u64) -> Reduced {
    let cap = (delta - 2) as usize;
    let mut deg = vec![0usize; n];
    for &(u, v, _) in edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
    }

    let mut cycle_of = Vec::with_capacity(n);
    let mut n2 = 0usize;
    for v in 0..n {
        let c = deg[v].div_ceil(cap).max(1); // at least one representative
        cycle_of.push((n2 as u32, c as u32));
        n2 += c;
    }

    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n2];
    for &(start, c) in &cycle_of {
        if c > 1 {
            for i in 0..c {
                adj[(start + i) as usize].push((start + (i + 1) % c, 0.0));
            }
        }
    }

    // Slot assignment order mirrors the reference's take_slot calls: u's
    // counter advances before v's slot is read (matters for self-loops).
    let mut slot_ctr = vec![0usize; n];
    for &(u, v, w) in edges {
        let (u, v) = (u as usize, v as usize);
        let xu = cycle_of[u].0 + (slot_ctr[u] / cap) as u32;
        slot_ctr[u] += 1;
        let xv = cycle_of[v].0 + (slot_ctr[v] / cap) as u32;
        slot_ctr[v] += 1;
        adj[xu as usize].push((xv, w));
    }
    Reduced { n2, adj, cycle_of }
}

// ---------------------------------------------------------------------------
// Data structure D (SPEC "Data structure D", Lemma 3.4). Keyed map with
// keep-smaller dedup; Pull returns keys ascending by value plus the tightest
// separating bound. Block internals are complexity-only and not modeled
// (same choice as the reference); the value-ordered tree replaces the
// reference's sort-on-pull — behavior-neutral by OPTIMIZATION.md pass 1
// (stored values are unique across keys because curr is a value component,
// so ascending tree order is exactly the reference's sorted order).

pub struct BlockDs {
    m_cap: u64,
    b: Label,
    by_key: FxHashMap<u32, Lab>,
    by_val: std::collections::BTreeMap<Lab, u32>,
}

impl BlockDs {
    pub fn new(m_cap: u64, b: Label) -> Self {
        BlockDs {
            m_cap,
            b,
            by_key: FxHashMap::default(),
            by_val: std::collections::BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: u32, val: Lab) {
        match self.by_key.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if val < *e.get() {
                    self.by_val.remove(e.get());
                    e.insert(val);
                    let evicted = self.by_val.insert(val, key);
                    debug_assert!(evicted.is_none(), "values are unique across keys");
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(val);
                let evicted = self.by_val.insert(val, key);
                debug_assert!(evicted.is_none(), "values are unique across keys");
            }
        }
    }

    /// Hash-map iteration order is unobservable here: keep-min per key is
    /// order-independent, and the target's own order is never read except
    /// through the value-ordered `pull`.
    pub fn merge(&mut self, other: &BlockDs) {
        for (&k, &v) in &other.by_key {
            self.insert(k, v);
        }
    }

    pub fn pull(&mut self) -> (Vec<u32>, Label) {
        let m = usize::try_from(self.m_cap).unwrap_or(usize::MAX);
        let take = self.by_key.len().min(m);
        let mut keys = Vec::with_capacity(take);
        for _ in 0..take {
            let (_val, key) = self.by_val.pop_first().expect("take <= |D|");
            self.by_key.remove(&key);
            debug_assert_eq!(self.by_key.len(), self.by_val.len());
            keys.push(key);
        }
        // Remainder non-empty iff |D| exceeded M; its minimum is then the
        // (M+1)-th smallest of the pre-pull content — the reference's bound.
        let bound = match self.by_val.first_key_value() {
            Some((&val, _)) => Label::Fin(val),
            None => self.b,
        };
        (keys, bound)
    }

    pub fn non_empty(&self) -> bool {
        !self.by_key.is_empty()
    }

    /// Smallest stored value, if any (the V5 base-case cap reads it once
    /// at return time).
    pub fn peek_min(&self) -> Option<Lab> {
        self.by_val.first_key_value().map(|(&v, _)| v)
    }

    /// Stored value for `key`, if present (trace introspection only).
    pub fn get(&self, key: u32) -> Option<Lab> {
        self.by_key.get(&key).copied()
    }
}

// ---------------------------------------------------------------------------
// Tree partition (SPEC "Tree partition", Algorithm 5), s-sized groups.

/// The reference's literal recursion, run on an explicit stack (merged F̄
/// trees can be deep enough to overflow a test thread's 2 MiB stack).
/// Children are visited in edge insertion order; after each child returns,
/// a ≥ s accumulator is reported and reset to the subtree root; the final
/// leftover merges into the last reported group (dedup against a snapshot
/// of that group, exactly like the reference's `set(groups[-1])`).
pub fn partition_tree(edge_list: &[(u32, u32)], root: u32, s: usize) -> Vec<Vec<u32>> {
    let mut padj: FxHashMap<u32, Vec<u32>> = FxHashMap::default();
    for &(u, v) in edge_list {
        padj.entry(u).or_default().push(v);
        padj.entry(v).or_default().push(u);
    }

    struct Frame {
        r: u32,
        parent: Option<u32>,
        idx: usize,
        acc: Vec<u32>,
    }
    let mut groups: Vec<Vec<u32>> = Vec::new();
    let mut stack = vec![Frame {
        r: root,
        parent: None,
        idx: 0,
        acc: vec![root],
    }];
    let mut returned: Option<Vec<u32>> = None;

    loop {
        let next_child = {
            let f = stack.last_mut().expect("stack never empties mid-loop");
            if let Some(child_acc) = returned.take() {
                f.acc.extend(child_acc);
                if f.acc.len() >= s {
                    let full = std::mem::replace(&mut f.acc, vec![f.r]);
                    groups.push(full);
                }
            }
            let children = padj.get(&f.r).map(Vec::as_slice).unwrap_or(&[]);
            let mut next = None;
            while f.idx < children.len() {
                let c = children[f.idx];
                f.idx += 1;
                if Some(c) != f.parent {
                    next = Some(c);
                    break;
                }
            }
            next
        };
        match next_child {
            Some(c) => {
                let parent = stack.last().expect("non-empty").r;
                stack.push(Frame {
                    r: c,
                    parent: Some(parent),
                    idx: 0,
                    acc: vec![c],
                });
            }
            None => {
                let f = stack.pop().expect("non-empty");
                if stack.is_empty() {
                    if let Some(last) = groups.last_mut() {
                        let have: FxHashSet<u32> = last.iter().copied().collect();
                        last.extend(f.acc.into_iter().filter(|v| !have.contains(v)));
                    } else {
                        groups.push(f.acc);
                    }
                    return groups;
                }
                returned = Some(f.acc);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The solver: BMSSP recursion on the reduced graph.

/// An F̄ tree: only its edge list and root are ever read (the reference's
/// per-tree vertex list is write-only — OPTIMIZATION.md pass 3, point 3).
struct Tree {
    edges: Vec<(u32, u32)>,
    root: u32,
}

/// Reusable FindPivots state (OPTIMIZATION.md pass 3): flat epoch-stamped
/// membership arrays replace the per-call hash maps and sets — every one
/// of them is queried for membership only, never iterated — and the search
/// containers are cleared and reused across searches. Stamp arrays start
/// at 0 and epochs at 1, so a zeroed array matches nothing.
struct FpScratch {
    epoch: u32,
    heap: BinaryHeap<Reverse<(Lab, u32)>>,
    k_vert: Vec<u32>,
    k_parent_order: Vec<(u32, u32)>,
    /// v ∈ K of the current search.
    k_stamp: Vec<u32>,
    /// v has a `k_parent_order` slot in the current search (+ its index).
    kp_stamp: Vec<u32>,
    kp_slot: Vec<u32>,
    /// v lies in an F̄ tree of the current call (+ which one).
    tree_stamp: Vec<u32>,
    tree_id: Vec<u32>,
    /// v ∈ Q / v ∈ W / v consumed by an earlier group / v in current group.
    q_stamp: Vec<u32>,
    w_stamp: Vec<u32>,
    prior_stamp: Vec<u32>,
    group_stamp: Vec<u32>,
}

impl Default for FpScratch {
    fn default() -> Self {
        FpScratch::new(0)
    }
}

impl FpScratch {
    fn new(n: usize) -> Self {
        FpScratch {
            epoch: 0,
            heap: BinaryHeap::new(),
            k_vert: Vec::new(),
            k_parent_order: Vec::new(),
            k_stamp: vec![0; n],
            kp_stamp: vec![0; n],
            kp_slot: vec![0; n],
            tree_stamp: vec![0; n],
            tree_id: vec![0; n],
            q_stamp: vec![0; n],
            w_stamp: vec![0; n],
            prior_stamp: vec![0; n],
            group_stamp: vec![0; n],
        }
    }

    /// Call-boundary epoch hygiene: no stamps are live between calls, so
    /// when less than half the u32 space remains, zero everything and
    /// restart. A single call cannot consume 2^31 epochs without at least
    /// as many heap operations first; `next_epoch` guards the absolute
    /// ceiling loudly rather than alias stamps.
    fn begin_call(&mut self) {
        if self.epoch >= 1 << 31 {
            for a in [
                &mut self.k_stamp,
                &mut self.kp_stamp,
                &mut self.tree_stamp,
                &mut self.q_stamp,
                &mut self.w_stamp,
                &mut self.prior_stamp,
                &mut self.group_stamp,
            ] {
                a.fill(0);
            }
            self.epoch = 0;
        }
    }

    fn next_epoch(&mut self) -> u32 {
        assert!(self.epoch < u32::MAX, "epoch space exhausted within one call");
        self.epoch += 1;
        self.epoch
    }
}

pub struct Solver<'a> {
    adj: &'a [Vec<(u32, f64)>],
    t: u64,
    k: u64,
    pub d: Vec<Label>,
    pub phase: PhaseTimes,
    /// Levels ≤ this run the base case with the level's cutoff instead of
    /// recursing (VARIANTS.md V3). 0 = the paper's algorithm: level 0 is
    /// the base case with cutoff t³·2⁰ = t³, bit-identical to Algorithm 4,
    /// so the mainline differential gate pins this default.
    pub hybrid_level: u64,
    /// VARIANTS.md V5: cap the base case's returned B′ at the minimum
    /// value left in D, restoring Lemma 3.1's contract (closes the Q3
    /// escape channel). false = the paper's Algorithm 4 as written; the
    /// mainline differential gate pins this default.
    pub fixed_base: bool,
    /// VARIANTS.md V8: per-vertex settled bit — settled vertices are
    /// never adopted (relax rejects), re-frontiered (pulls drop them),
    /// or re-settled (W′ excludes them). false = the paper's algorithm;
    /// the mainline differential gate pins this default.
    pub settled_dedup: bool,
    /// V8 state: set at each settlement event when `settled_dedup` is on.
    settled: Vec<bool>,
    /// How many base-case returns the V5 cap actually lowered (0 when
    /// `fixed_base` is off) — experiment instrumentation.
    pub base_cap_hits: u64,
    /// Contract counters (feature `verify-contract`; QUESTIONS.md Q3,
    /// Q10, Q12): per-return violations of dB[D] ≥ B′ over stored values,
    /// split by return site, and of the full-execution dichotomy
    /// (B′ = B ⟺ D = ∅). Counting, not asserting — these states are
    /// audit findings, and the runs that exhibit them still finish with
    /// exact distances.
    #[cfg(feature = "verify-contract")]
    pub viol_bound_base: u64,
    #[cfg(feature = "verify-contract")]
    pub viol_bound_rec: u64,
    #[cfg(feature = "verify-contract")]
    pub viol_dichotomy: u64,
    /// Q14 classification counters (feature `dup-classify`).
    #[cfg(feature = "dup-classify")]
    pub dup_class: DupClassCounts,
    /// Observation-only settled marks for classification (never read by
    /// control flow — the algorithm behaves identically with the feature
    /// on).
    #[cfg(feature = "dup-classify")]
    first_settled: Vec<bool>,
    /// Q11 trace: the vertex whose events are recorded (None = off).
    #[cfg(feature = "q11-trace")]
    pub trace_vertex: Option<u32>,
    /// Q11 trace: recorded events, in order.
    #[cfg(feature = "q11-trace")]
    pub trace_events: Vec<String>,
    #[cfg(feature = "q11-trace")]
    trace_stack: Vec<String>,
    #[cfg(feature = "q11-trace")]
    trace_site: &'static str,
    #[cfg(feature = "q11-trace")]
    trace_settles: usize,
    scratch: FpScratch,
}

#[cfg(feature = "q11-trace")]
impl<'a> Solver<'a> {
    fn tr(&mut self, msg: String) {
        self.trace_events
            .push(format!("{msg}\n        in {}", self.trace_stack.join(" / ")));
    }
}

impl<'a> Solver<'a> {
    pub fn new(adj: &'a [Vec<(u32, f64)>], t: u64, k: u64) -> Self {
        Solver {
            adj,
            t,
            k,
            d: vec![Label::Inf; adj.len()],
            phase: PhaseTimes::default(),
            hybrid_level: 0,
            fixed_base: false,
            settled_dedup: false,
            settled: vec![false; adj.len()],
            base_cap_hits: 0,
            #[cfg(feature = "dup-classify")]
            dup_class: DupClassCounts::default(),
            #[cfg(feature = "dup-classify")]
            first_settled: vec![false; adj.len()],
            #[cfg(feature = "verify-contract")]
            viol_bound_base: 0,
            #[cfg(feature = "verify-contract")]
            viol_bound_rec: 0,
            #[cfg(feature = "verify-contract")]
            viol_dichotomy: 0,
            #[cfg(feature = "q11-trace")]
            trace_vertex: None,
            #[cfg(feature = "q11-trace")]
            trace_events: Vec::new(),
            #[cfg(feature = "q11-trace")]
            trace_stack: Vec::new(),
            #[cfg(feature = "q11-trace")]
            trace_site: "",
            #[cfg(feature = "q11-trace")]
            trace_settles: 0,
            scratch: FpScratch::new(adj.len()),
        }
    }

    fn relax(&mut self, u: u32, v: u32, w: f64, b: Label) -> bool {
        if self.settled_dedup && self.settled[v as usize] {
            // V8: a settled vertex is complete — an improving candidate is
            // impossible and an exact-tie re-store is a no-op; reject both
            // so the vertex is never re-adopted.
            return false;
        }
        let du = self.d[u as usize].fin(); // relaxation source is always reached
        let cand = Label::Fin(Lab {
            len: du.len + w,
            hops: du.hops + 1,
            curr: v,
            pred: u as i64,
        });
        if cand <= self.d[v as usize] && cand < b {
            #[cfg(feature = "q11-trace")]
            if self.trace_vertex == Some(v) {
                let eq = cand == self.d[v as usize];
                self.tr(format!(
                    "RELAX-ACCEPT{} at {}: u={u} w={w} cand={} old={}",
                    if eq { " [exact-tie re-store]" } else { "" },
                    self.trace_site,
                    fmt_label(cand),
                    fmt_label(self.d[v as usize]),
                ));
            }
            self.d[v as usize] = cand;
            true
        } else {
            false
        }
    }

    // -- FindPivots (SPEC "FindPivots", Algorithm 2) ----------------------

    #[allow(clippy::type_complexity)]
    fn find_pivots(&mut self, b: Label, s_list: &[u32]) -> (Vec<Vec<u32>>, Vec<u32>, Vec<u32>) {
        #[cfg(feature = "q11-trace")]
        {
            self.trace_site = "find_pivots";
        }
        let adj = self.adj;
        let mut sc = std::mem::take(&mut self.scratch);
        sc.begin_call();
        let tree_ep = sc.next_epoch();
        let q_ep = sc.next_epoch();
        let w_ep = sc.next_epoch();
        let prior_ep = sc.next_epoch();

        let mut trees: Vec<Tree> = Vec::new();
        let mut q_list: Vec<u32> = Vec::new();
        let mut w_list: Vec<u32> = Vec::new();

        for &x in s_list {
            if sc.tree_stamp[x as usize] == tree_ep {
                continue;
            }
            let k_ep = sc.next_epoch();
            let kp_ep = sc.next_epoch();
            sc.heap.clear();
            sc.k_vert.clear();
            sc.k_parent_order.clear();
            sc.heap.push(Reverse((self.d[x as usize].fin(), x)));
            sc.k_vert.push(x);
            sc.k_stamp[x as usize] = k_ep;
            let mut merged = false;

            // |K| == k_vert.len(): both grow by one on every first insert.
            'search: while !sc.heap.is_empty() && (sc.k_vert.len() as u64) < self.k {
                let Reverse((lab, u)) = sc.heap.pop().expect("heap non-empty");
                if Label::Fin(lab) != self.d[u as usize] {
                    continue; // stale lazy-deletion entry
                }
                for &(v, w) in &adj[u as usize] {
                    if !self.relax(u, v, w, b) {
                        continue;
                    }
                    let vi = v as usize;
                    if sc.k_stamp[vi] != k_ep {
                        sc.k_stamp[vi] = k_ep;
                        sc.k_vert.push(v);
                        #[cfg(feature = "q11-trace")]
                        if self.trace_vertex == Some(v) {
                            self.tr(format!("JOIN-K (local search root x={x})"));
                        }
                    }
                    // v -> (u, v), latest valid tree edge into v; an
                    // overwrite keeps the original slot (dict semantics).
                    if sc.kp_stamp[vi] == kp_ep {
                        sc.k_parent_order[sc.kp_slot[vi] as usize] = (u, v);
                    } else {
                        sc.kp_stamp[vi] = kp_ep;
                        sc.kp_slot[vi] = sc.k_parent_order.len() as u32;
                        sc.k_parent_order.push((u, v));
                    }
                    if sc.tree_stamp[vi] == tree_ep {
                        // Contact: merge K into existing tree j; x abandoned.
                        let j = sc.tree_id[vi];
                        for &kv in &sc.k_vert {
                            sc.tree_stamp[kv as usize] = tree_ep;
                            sc.tree_id[kv as usize] = j;
                        }
                        trees[j as usize]
                            .edges
                            .extend_from_slice(&sc.k_parent_order);
                        merged = true;
                        break 'search;
                    }
                    sc.heap.push(Reverse((self.d[vi].fin(), v)));
                }
            }

            if merged {
                continue; // nothing reported for a merged search
            }
            if sc.k_vert.len() as u64 >= self.k {
                let j = trees.len() as u32;
                for &kv in &sc.k_vert {
                    sc.tree_stamp[kv as usize] = tree_ep;
                    sc.tree_id[kv as usize] = j;
                }
                trees.push(Tree {
                    edges: sc.k_parent_order.clone(),
                    root: x,
                });
            } else {
                q_list.push(x);
                sc.q_stamp[x as usize] = q_ep;
                for &kv in &sc.k_vert {
                    if sc.w_stamp[kv as usize] != w_ep {
                        sc.w_stamp[kv as usize] = w_ep;
                        w_list.push(kv);
                        #[cfg(feature = "q11-trace")]
                        if self.trace_vertex == Some(kv) {
                            let sz = sc.k_vert.len();
                            self.tr(format!("JOIN-W (failed search root x={x}, |K|={sz})"));
                        }
                    }
                }
            }
        }

        let mut p_lists: Vec<Vec<u32>> = Vec::new();
        for tree in &trees {
            let groups = phase!(
                self,
                partition,
                partition_tree(&tree.edges, tree.root, self.k as usize)
            );
            for group in groups {
                let g_ep = sc.next_epoch();
                for &gv in &group {
                    sc.group_stamp[gv as usize] = g_ep;
                }
                let p_j: Vec<u32> = s_list
                    .iter()
                    .copied()
                    .filter(|&v| {
                        let vi = v as usize;
                        sc.group_stamp[vi] == g_ep
                            && sc.q_stamp[vi] != q_ep
                            && sc.prior_stamp[vi] != prior_ep
                    })
                    .collect();
                for &gv in &group {
                    sc.prior_stamp[gv as usize] = prior_ep;
                }
                if !p_j.is_empty() {
                    p_lists.push(p_j); // QUESTIONS.md Q2: empty P_j dropped
                }
            }
        }
        self.scratch = sc;
        (p_lists, q_list, w_list)
    }

    // -- Base case (SPEC "Base case", Algorithm 4) ------------------------

    fn base_case(&mut self, b: Label, s_list: &[u32], cutoff: u64) -> (Label, Vec<u32>, BlockDs) {
        #[cfg(feature = "q11-trace")]
        {
            self.trace_stack
                .push(format!("base(B={}, cutoff={cutoff})", fmt_label(b)));
            self.trace_site = "base_case";
            if let Some(tv) = self.trace_vertex {
                if s_list.contains(&tv) {
                    let d = fmt_label(self.d[tv as usize]);
                    self.tr(format!("SEED into base D at d={d}"));
                }
            }
        }
        let adj = self.adj;
        let mut ds = BlockDs::new(1, b);
        #[cfg(feature = "dup-classify")]
        let seed_set: FxHashSet<u32> = s_list.iter().copied().collect();
        for &x in s_list {
            ds.insert(x, self.d[x as usize].fin());
        }
        let mut u_log: Vec<u32> = Vec::new();
        let mut bound = b;
        while ds.non_empty() && (u_log.len() as u64) <= cutoff {
            let (keys, nb) = ds.pull();
            bound = nb;
            let u = keys[0];
            if self.settled_dedup && self.settled[u as usize] {
                continue; // V8: stale entry of a settled vertex — drop
            }
            u_log.push(u);
            #[cfg(feature = "dup-classify")]
            {
                if self.first_settled[u as usize] {
                    if seed_set.contains(&u) {
                        self.dup_class.base_seeded += 1;
                    } else {
                        self.dup_class.base_inserted += 1;
                    }
                }
                self.first_settled[u as usize] = true;
            }
            if self.settled_dedup {
                self.settled[u as usize] = true;
            }
            #[cfg(feature = "q11-trace")]
            if self.trace_vertex == Some(u) {
                self.trace_settles += 1;
                let n = self.trace_settles;
                let d = fmt_label(self.d[u as usize]);
                let bd = fmt_label(bound);
                self.tr(format!("SETTLE #{n} [base-case pull] d={d} bound-after={bd}"));
            }
            for &(v, w) in &adj[u as usize] {
                if self.relax(u, v, w, b) {
                    // QUESTIONS.md Q3: the insert may undercut the returned bound.
                    ds.insert(v, self.d[v as usize].fin());
                    #[cfg(feature = "q11-trace")]
                    if self.trace_vertex == Some(v) {
                        let d = fmt_label(self.d[v as usize]);
                        self.tr(format!("INSERT base D (post-relax) val={d}"));
                    }
                }
            }
        }
        if self.fixed_base {
            // V5: cap B′ at dB[D] so the returned pair honors Lemma 3.1.
            if let Some(minv) = ds.peek_min() {
                if Label::Fin(minv) < bound {
                    bound = Label::Fin(minv);
                    self.base_cap_hits += 1;
                }
            }
        }
        #[cfg(feature = "verify-contract")]
        {
            if matches!(ds.peek_min(), Some(m) if Label::Fin(m) < bound) {
                self.viol_bound_base += 1;
            }
            if (bound == b) != !ds.non_empty() {
                self.viol_dichotomy += 1;
            }
        }
        #[cfg(feature = "q11-trace")]
        {
            self.trace_stack.pop();
        }
        (bound, u_log, ds)
    }

    // -- BMSSP (SPEC "BMSSP", Algorithm 3) --------------------------------

    pub fn bmssp(&mut self, b: Label, s_list: &[u32], level: u64) -> (Label, Vec<u32>, BlockDs) {
        if level <= self.hybrid_level {
            // The level's own |U| cutoff, t³·2^(l·t); at level 0 this is
            // exactly Algorithm 4's t³.
            let t = self.t;
            let cutoff = (t * t * t).saturating_mul(pow2_sat(level * t));
            return phase!(self, base_case, self.base_case(b, s_list, cutoff));
        }
        #[cfg(feature = "q11-trace")]
        {
            let f = format!("bmssp(l={level}, B={}, |S|={})", fmt_label(b), s_list.len());
            self.trace_stack.push(f);
        }
        let adj = self.adj;
        let t = self.t;
        let mut ds = BlockDs::new(t.saturating_mul(pow2_sat((level - 1) * t)), b);
        let (mut p_lists, _q_list, w_list) = phase!(self, find_pivots, self.find_pivots(b, s_list));

        let mut piv: Vec<Option<u32>> = Vec::with_capacity(p_lists.len()); // None = dead
        let mut piv_of: FxHashMap<u32, usize> = FxHashMap::default(); // current pivot -> j
        let mut member: FxHashMap<u32, usize> = FxHashMap::default(); // vertex -> its P_j
        for (j, p_j) in p_lists.iter().enumerate() {
            let mut p = p_j[0];
            for &v in &p_j[1..] {
                if self.d[v as usize] < self.d[p as usize] {
                    p = v; // unique argmin: tuple order is total
                }
            }
            piv.push(Some(p));
            piv_of.insert(p, j);
            for &v in p_j {
                member.insert(v, j);
            }
            ds.insert(p, self.d[p as usize].fin());
            #[cfg(feature = "q11-trace")]
            if self.trace_vertex == Some(p) {
                let d = fmt_label(self.d[p as usize]);
                self.tr(format!("INSERT parent D as initial pivot of P_{j} val={d}"));
            }
        }

        let mut b_prime = b;
        for p in piv.iter().flatten() {
            b_prime = b_prime.min(self.d[*p as usize]);
        }
        let mut u_log: Vec<u32> = Vec::new();
        let mut u_set: FxHashSet<u32> = FxHashSet::default();
        let limit = (t * t * t).saturating_mul(pow2_sat(level * t));
        let mut j_pending: BTreeSet<usize> = BTreeSet::new(); // the paper's J

        while (u_log.len() as u64) <= limit && ds.non_empty() {
            let (si_keys, b_i) = phase!(self, ds_pull, ds.pull());
            #[cfg(feature = "q11-trace")]
            if let Some(tv) = self.trace_vertex {
                if si_keys.contains(&tv) {
                    let bi = fmt_label(b_i);
                    let d = fmt_label(self.d[tv as usize]);
                    self.tr(format!("PULL into S_i (b_i={bi}) d={d}"));
                }
            }
            let mut si: Vec<u32> = si_keys.clone();
            let mut si_set: FxHashSet<u32> = si_keys.iter().copied().collect();
            for &x in &si_keys {
                // pivot expansion (line 13), P_j stored order
                let Some(&j) = piv_of.get(&x) else { continue };
                for &v in &p_lists[j] {
                    if self.settled_dedup && self.settled[v as usize] {
                        continue; // V8: settled members are not re-frontiered
                    }
                    if self.d[v as usize] < b_i && si_set.insert(v) {
                        si.push(v);
                        #[cfg(feature = "q11-trace")]
                        if self.trace_vertex == Some(v) {
                            let d = fmt_label(self.d[v as usize]);
                            self.tr(format!("EXPAND into S_i via pivot x={x} of P_{j} d={d}"));
                        }
                    }
                }
            }

            if self.settled_dedup {
                // V8: pulled keys that are already settled expanded their
                // P_j above but do not seed the child.
                si.retain(|&x| !self.settled[x as usize]);
            }
            let (b_pi, u_i, ds_i) = self.bmssp(b_i, &si, level - 1);
            phase!(self, ds_merge, ds.merge(&ds_i));
            #[cfg(feature = "q11-trace")]
            if let Some(tv) = self.trace_vertex {
                if let Some(val) = ds_i.get(tv) {
                    let v = fmt_label(Label::Fin(val));
                    self.tr(format!("MERGE-CARRY child D contains v at val={v}"));
                }
            }

            phase!(self, pivot_bookkeeping, {
                for &u in &u_i {
                    // pivot bookkeeping pass (lines 16-20)
                    let Some(j) = member.remove(&u) else { continue };
                    let pos = p_lists[j]
                        .iter()
                        .position(|&x| x == u)
                        .expect("member map mirrors P_j contents");
                    p_lists[j].remove(pos);
                    if piv[j] == Some(u) {
                        piv_of.remove(&u);
                        if !p_lists[j].is_empty() {
                            j_pending.insert(j);
                        } else {
                            piv[j] = None; // pivot dies with its set
                        }
                    }
                }
            });

            phase!(self, relax_sweep, {
                #[cfg(feature = "q11-trace")]
                {
                    self.trace_site = "relax_sweep";
                }
                for &u in &u_i {
                    // relax sweep (lines 21-26), incl. DS inserts
                    for &(v, w) in &adj[u as usize] {
                        if self.relax(u, v, w, b) && self.d[v as usize] >= b_i {
                            ds.insert(v, self.d[v as usize].fin());
                            #[cfg(feature = "q11-trace")]
                            if self.trace_vertex == Some(v) {
                                let d = fmt_label(self.d[v as usize]);
                                let bi = fmt_label(b_i);
                                self.tr(format!(
                                    "INSERT parent D via relax_sweep from u={u} (d={d} >= b_i={bi})"
                                ));
                            }
                            if let Some(&j) = member.get(&v) {
                                if !j_pending.contains(&j) {
                                    if let Some(pj) = piv[j] {
                                        if self.d[pj as usize] > self.d[v as usize] {
                                            piv_of.remove(&pj);
                                            piv[j] = Some(v);
                                            piv_of.insert(v, j);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });

            phase!(self, pivot_bookkeeping, {
                for &j in &j_pending {
                    // re-selection (lines 27-29), ascending j
                    if p_lists[j].is_empty() {
                        // QUESTIONS.md Q8: P_j emptied after j entered J; the
                        // paper's argmin is over an empty set here. Pivot dies.
                        piv[j] = None;
                        continue;
                    }
                    let mut p = p_lists[j][0];
                    for &v in &p_lists[j][1..] {
                        if self.d[v as usize] < self.d[p as usize] {
                            p = v;
                        }
                    }
                    piv[j] = Some(p);
                    piv_of.insert(p, j);
                    ds.insert(p, self.d[p as usize].fin());
                    #[cfg(feature = "q11-trace")]
                    if self.trace_vertex == Some(p) {
                        let d = fmt_label(self.d[p as usize]);
                        self.tr(format!("INSERT parent D as re-selected pivot of P_{j} val={d}"));
                    }
                }
            });

            #[cfg(feature = "dup-classify")]
            if b_pi < b_prime {
                self.dup_class.bprime_regressions += 1;
            }
            b_prime = b_pi;
            u_set.extend(u_i.iter().copied());
            u_log.extend(u_i);
            j_pending.clear();
        }

        let w_prime = phase!(self, boundary_sweeps, {
            for &x in s_list {
                // premature vertices go back to the frontier (line 33)
                if self.settled_dedup && self.settled[x as usize] {
                    continue; // V8
                }
                if b_prime <= self.d[x as usize] && self.d[x as usize] < b {
                    ds.insert(x, self.d[x as usize].fin());
                    #[cfg(feature = "q11-trace")]
                    if self.trace_vertex == Some(x) {
                        let d = fmt_label(self.d[x as usize]);
                        let bp = fmt_label(b_prime);
                        self.tr(format!(
                            "INSERT parent D as premature S-vertex, line 33 (b_prime={bp} <= d={d})"
                        ));
                    }
                }
            }
            let w_prime: Vec<u32> = w_list
                .iter()
                .copied()
                .filter(|&x| {
                    !u_set.contains(&x)
                        && self.d[x as usize] < b_prime
                        && !(self.settled_dedup && self.settled[x as usize])
                })
                .collect();
            if self.settled_dedup {
                for &x in &w_prime {
                    self.settled[x as usize] = true; // V8: W′ settles
                }
            }
            #[cfg(feature = "dup-classify")]
            {
                let s_set: FxHashSet<u32> = s_list.iter().copied().collect();
                let q_set: FxHashSet<u32> = _q_list.iter().copied().collect();
                for &x in &w_prime {
                    if self.first_settled[x as usize] {
                        if s_set.contains(&x) {
                            if q_set.contains(&x) {
                                self.dup_class.wprime_own_root += 1;
                            } else {
                                self.dup_class.wprime_own_swept += 1;
                            }
                        } else {
                            self.dup_class.wprime_foreign += 1;
                            // Lemma D check: walk the pred chain (labels of
                            // complete vertices are final, so the chain is
                            // stable) at most k−1 hops looking for a failed
                            // root of THIS call.
                            let (mut in_q, mut in_s) = (false, false);
                            let mut cur = x;
                            for _ in 0..self.k.saturating_sub(1) {
                                let p = match self.d[cur as usize] {
                                    Label::Fin(l) => l.pred,
                                    Label::Inf => -1,
                                };
                                if p < 0 {
                                    break;
                                }
                                let p = p as u32;
                                if q_set.contains(&p) {
                                    in_q = true;
                                    in_s = true;
                                    break;
                                }
                                if s_set.contains(&p) {
                                    in_s = true;
                                }
                                cur = p;
                            }
                            if in_q {
                                self.dup_class.foreign_anc_in_q += 1;
                            }
                            if in_s {
                                self.dup_class.foreign_anc_in_s += 1;
                            }
                        }
                    }
                    self.first_settled[x as usize] = true;
                }
            }
            #[cfg(feature = "q11-trace")]
            if let Some(tv) = self.trace_vertex {
                if w_prime.contains(&tv) {
                    self.trace_settles += 1;
                    let n = self.trace_settles;
                    let d = fmt_label(self.d[tv as usize]);
                    let bp = fmt_label(b_prime);
                    let in_u = u_set.contains(&tv);
                    self.tr(format!(
                        "SETTLE #{n} [W′ boundary append] d={d} b_prime={bp} in-this-call-U={in_u}"
                    ));
                }
                self.trace_site = "w_prime_sweep";
            }
            for &u in &w_prime {
                // final relax from completed arborescences
                for &(v, w) in &adj[u as usize] {
                    if self.relax(u, v, w, b) && self.d[v as usize] >= b_prime {
                        ds.insert(v, self.d[v as usize].fin());
                        #[cfg(feature = "q11-trace")]
                        if self.trace_vertex == Some(v) {
                            let d = fmt_label(self.d[v as usize]);
                            self.tr(format!("INSERT parent D via w_prime_sweep from u={u} val={d}"));
                        }
                    }
                }
            }
            w_prime
        });
        u_log.extend_from_slice(&w_prime);
        #[cfg(feature = "verify-contract")]
        {
            if matches!(ds.peek_min(), Some(m) if Label::Fin(m) < b_prime) {
                self.viol_bound_rec += 1; // QUESTIONS.md Q12's leak, live
            }
            if (b_prime == b) != !ds.non_empty() {
                self.viol_dichotomy += 1;
            }
        }
        #[cfg(feature = "q11-trace")]
        {
            self.trace_stack.pop();
        }
        (b_prime, u_log, ds)
    }
}

// ---------------------------------------------------------------------------
// Public entry point.

pub struct RunResult {
    pub delta: u64,
    pub t: u64,
    pub k: u64,
    pub l_top: u64,
    pub n2: usize,
    /// Final labels on the reduced graph — the differential gate's payload.
    pub labels: Vec<Label>,
    /// Settlement log: the U list of the top-level call, in order.
    pub settle: Vec<u32>,
    /// Readback: shortest-path lengths for original vertices.
    pub dist: Vec<f64>,
    /// Per-phase wall clock (all zeros unless built with `phase-timer`).
    pub phase_times: PhaseTimes,
    /// Total wall clock of the run (zero unless built with `phase-timer`).
    pub total: f64,
}

/// Full run per SPEC "Output readback": degree-reduce, BMSSP from the
/// source's first cycle vertex, per-original-vertex length readback.
/// `override_params` forces (delta, t, k) — the harness's deep-recursion
/// regime; l_top always follows the formula.
pub fn sssp_run(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    override_params: Option<(u64, u64, u64)>,
) -> RunResult {
    let m = edges.len();
    #[cfg(feature = "phase-timer")]
    let total_t0 = std::time::Instant::now();

    // The delta used for the reduction: the reference computes it before
    // reducing, and the formula does not depend on n2, so hoisting it out
    // of the branch below is a no-op.
    let pre_delta = match override_params {
        None => choose_params(n, m, 0).0,
        Some((df, _, _)) => df,
    };
    #[cfg(feature = "phase-timer")]
    let reduce_t0 = std::time::Instant::now();
    let red = degree_reduce(n, edges, pre_delta);
    #[cfg(feature = "phase-timer")]
    let reduce_secs = reduce_t0.elapsed().as_secs_f64();

    let (delta, t, k, l_top) = match override_params {
        None => choose_params(n, m, red.n2),
        Some((df, tf, kf)) => {
            let log_n2 = (red.n2.max(2) as f64).log2().max(1.0);
            (df, tf, kf, (log_n2 / tf as f64).ceil() as u64)
        }
    };

    let mut solver = Solver::new(&red.adj, t, k);
    #[cfg(feature = "phase-timer")]
    {
        solver.phase.reduce = reduce_secs;
    }
    let s2 = red.cycle_of[source].0;
    solver.d[s2 as usize] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: s2,
        pred: -1,
    });
    let (b_final, settle, ds_top) = solver.bmssp(Label::Inf, &[s2], l_top);
    // Totality assert (QUESTIONS.md Q10): the top-level call must be a
    // full execution. The paper's limit t³·2^(l_top·t) exceeds n, but
    // re-settlements (Q3 escapes) also count against it and no bound on
    // them is proved — if the limit ever fires, distances would be
    // silently wrong; fail loudly instead.
    assert!(
        b_final == Label::Inf && !ds_top.non_empty(),
        "top-level BMSSP exited partially (re-settlement volume exceeded \
         the |U| cutoff) — see QUESTIONS.md Q10"
    );

    let mut dist = Vec::with_capacity(n);
    for &(start, c) in &red.cycle_of {
        let mut best = Label::Inf;
        for i in 0..c {
            best = best.min(solver.d[(start + i) as usize]);
        }
        dist.push(match best {
            Label::Fin(l) => l.len,
            Label::Inf => f64::INFINITY,
        });
    }
    #[cfg(feature = "phase-timer")]
    let total = total_t0.elapsed().as_secs_f64();
    #[cfg(not(feature = "phase-timer"))]
    let total = 0.0;
    RunResult {
        delta,
        t,
        k,
        l_top,
        n2: red.n2,
        labels: solver.d,
        settle,
        dist,
        phase_times: solver.phase.clone(),
        total,
    }
}
