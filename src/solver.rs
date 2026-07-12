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
fn pow2_sat(e: u64) -> u64 {
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

struct Tree {
    vert: Vec<u32>,
    set: FxHashSet<u32>,
    edges: Vec<(u32, u32)>,
    root: u32,
}

pub struct Solver<'a> {
    adj: &'a [Vec<(u32, f64)>],
    t: u64,
    k: u64,
    pub d: Vec<Label>,
}

impl<'a> Solver<'a> {
    pub fn new(adj: &'a [Vec<(u32, f64)>], t: u64, k: u64) -> Self {
        Solver {
            adj,
            t,
            k,
            d: vec![Label::Inf; adj.len()],
        }
    }

    fn relax(&mut self, u: u32, v: u32, w: f64, b: Label) -> bool {
        let du = self.d[u as usize].fin(); // relaxation source is always reached
        let cand = Label::Fin(Lab {
            len: du.len + w,
            hops: du.hops + 1,
            curr: v,
            pred: u as i64,
        });
        if cand <= self.d[v as usize] && cand < b {
            self.d[v as usize] = cand;
            true
        } else {
            false
        }
    }

    // -- FindPivots (SPEC "FindPivots", Algorithm 2) ----------------------

    #[allow(clippy::type_complexity)]
    fn find_pivots(&mut self, b: Label, s_list: &[u32]) -> (Vec<Vec<u32>>, Vec<u32>, Vec<u32>) {
        let adj = self.adj;
        let mut tree_of: FxHashMap<u32, usize> = FxHashMap::default();
        let mut trees: Vec<Tree> = Vec::new();
        let mut q_list: Vec<u32> = Vec::new();
        let mut q_set: FxHashSet<u32> = FxHashSet::default();
        let mut w_list: Vec<u32> = Vec::new();
        let mut w_set: FxHashSet<u32> = FxHashSet::default();

        for &x in s_list {
            if tree_of.contains_key(&x) {
                continue;
            }
            let mut heap: BinaryHeap<Reverse<(Lab, u32)>> = BinaryHeap::new();
            heap.push(Reverse((self.d[x as usize].fin(), x)));
            let mut k_vert: Vec<u32> = vec![x];
            let mut k_set: FxHashSet<u32> = FxHashSet::default();
            k_set.insert(x);
            // v -> (u, v), latest valid tree edge into v. Insertion-ordered
            // like the reference's dict: an overwrite keeps the original slot.
            let mut k_parent_order: Vec<(u32, u32)> = Vec::new();
            let mut k_parent_idx: FxHashMap<u32, usize> = FxHashMap::default();
            let mut merged = false;

            'search: while !heap.is_empty() && (k_set.len() as u64) < self.k {
                let Reverse((lab, u)) = heap.pop().expect("heap non-empty");
                if Label::Fin(lab) != self.d[u as usize] {
                    continue; // stale lazy-deletion entry
                }
                for &(v, w) in &adj[u as usize] {
                    if !self.relax(u, v, w, b) {
                        continue;
                    }
                    if k_set.insert(v) {
                        k_vert.push(v);
                    }
                    match k_parent_idx.entry(v) {
                        std::collections::hash_map::Entry::Occupied(e) => {
                            k_parent_order[*e.get()] = (u, v);
                        }
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(k_parent_order.len());
                            k_parent_order.push((u, v));
                        }
                    }
                    if let Some(&j) = tree_of.get(&v) {
                        // Contact: merge K into existing tree j; x abandoned.
                        let tree = &mut trees[j];
                        for &kv in &k_vert {
                            tree_of.insert(kv, j);
                            if tree.set.insert(kv) {
                                tree.vert.push(kv);
                            }
                        }
                        tree.edges.extend(k_parent_order.iter().copied());
                        merged = true;
                        break 'search;
                    }
                    heap.push(Reverse((self.d[v as usize].fin(), v)));
                }
            }

            if merged {
                continue; // nothing reported for a merged search
            }
            if k_set.len() as u64 >= self.k {
                let j = trees.len();
                for &kv in &k_vert {
                    tree_of.insert(kv, j);
                }
                trees.push(Tree {
                    vert: k_vert,
                    set: k_set,
                    edges: k_parent_order,
                    root: x,
                });
            } else {
                q_list.push(x);
                q_set.insert(x);
                for &kv in &k_vert {
                    if w_set.insert(kv) {
                        w_list.push(kv);
                    }
                }
            }
        }

        let mut p_lists: Vec<Vec<u32>> = Vec::new();
        let mut prior: FxHashSet<u32> = FxHashSet::default();
        for tree in &trees {
            for group in partition_tree(&tree.edges, tree.root, self.k as usize) {
                let gset: FxHashSet<u32> = group.iter().copied().collect();
                let p_j: Vec<u32> = s_list
                    .iter()
                    .copied()
                    .filter(|v| gset.contains(v) && !q_set.contains(v) && !prior.contains(v))
                    .collect();
                prior.extend(gset);
                if !p_j.is_empty() {
                    p_lists.push(p_j); // QUESTIONS.md Q2: empty P_j dropped
                }
            }
        }
        (p_lists, q_list, w_list)
    }

    // -- Base case (SPEC "Base case", Algorithm 4) ------------------------

    fn base_case(&mut self, b: Label, s_list: &[u32]) -> (Label, Vec<u32>, BlockDs) {
        let adj = self.adj;
        let mut ds = BlockDs::new(1, b);
        for &x in s_list {
            ds.insert(x, self.d[x as usize].fin());
        }
        let mut u_log: Vec<u32> = Vec::new();
        let mut bound = b;
        let cutoff = self.t * self.t * self.t;
        while ds.non_empty() && (u_log.len() as u64) <= cutoff {
            let (keys, nb) = ds.pull();
            bound = nb;
            let u = keys[0];
            u_log.push(u);
            for &(v, w) in &adj[u as usize] {
                if self.relax(u, v, w, b) {
                    // QUESTIONS.md Q3: the insert may undercut the returned bound.
                    ds.insert(v, self.d[v as usize].fin());
                }
            }
        }
        (bound, u_log, ds)
    }

    // -- BMSSP (SPEC "BMSSP", Algorithm 3) --------------------------------

    pub fn bmssp(&mut self, b: Label, s_list: &[u32], level: u64) -> (Label, Vec<u32>, BlockDs) {
        if level == 0 {
            return self.base_case(b, s_list);
        }
        let adj = self.adj;
        let t = self.t;
        let mut ds = BlockDs::new(t.saturating_mul(pow2_sat((level - 1) * t)), b);
        let (mut p_lists, _q, w_list) = self.find_pivots(b, s_list);

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
            let (si_keys, b_i) = ds.pull();
            let mut si: Vec<u32> = si_keys.clone();
            let mut si_set: FxHashSet<u32> = si_keys.iter().copied().collect();
            for &x in &si_keys {
                // pivot expansion (line 13), P_j stored order
                let Some(&j) = piv_of.get(&x) else { continue };
                for &v in &p_lists[j] {
                    if self.d[v as usize] < b_i && si_set.insert(v) {
                        si.push(v);
                    }
                }
            }

            let (b_pi, u_i, ds_i) = self.bmssp(b_i, &si, level - 1);
            ds.merge(&ds_i);

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

            for &u in &u_i {
                // relax sweep (lines 21-26)
                for &(v, w) in &adj[u as usize] {
                    if self.relax(u, v, w, b) && self.d[v as usize] >= b_i {
                        ds.insert(v, self.d[v as usize].fin());
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
            }

            b_prime = b_pi;
            u_set.extend(u_i.iter().copied());
            u_log.extend(u_i);
            j_pending.clear();
        }

        for &x in s_list {
            // premature vertices go back to the frontier (line 33)
            if b_prime <= self.d[x as usize] && self.d[x as usize] < b {
                ds.insert(x, self.d[x as usize].fin());
            }
        }
        let w_prime: Vec<u32> = w_list
            .iter()
            .copied()
            .filter(|&x| !u_set.contains(&x) && self.d[x as usize] < b_prime)
            .collect();
        for &u in &w_prime {
            // final relax from completed arborescences
            for &(v, w) in &adj[u as usize] {
                if self.relax(u, v, w, b) && self.d[v as usize] >= b_prime {
                    ds.insert(v, self.d[v as usize].fin());
                }
            }
        }
        u_log.extend_from_slice(&w_prime);
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
    let (delta, t, k, l_top, red) = match override_params {
        None => {
            let (d0, _, _, _) = choose_params(n, m, 0);
            let r = degree_reduce(n, edges, d0);
            let (d1, t1, k1, l1) = choose_params(n, m, r.n2);
            (d1, t1, k1, l1, r)
        }
        Some((df, tf, kf)) => {
            let r = degree_reduce(n, edges, df);
            let log_n2 = (r.n2.max(2) as f64).log2().max(1.0);
            let lf = (log_n2 / tf as f64).ceil() as u64;
            (df, tf, kf, lf, r)
        }
    };

    let mut solver = Solver::new(&red.adj, t, k);
    let s2 = red.cycle_of[source].0;
    solver.d[s2 as usize] = Label::Fin(Lab {
        len: 0.0,
        hops: 0,
        curr: s2,
        pred: -1,
    });
    let (_, settle, _) = solver.bmssp(Label::Inf, &[s2], l_top);

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
    RunResult {
        delta,
        t,
        k,
        l_top,
        n2: red.n2,
        labels: solver.d,
        settle,
        dist,
    }
}
