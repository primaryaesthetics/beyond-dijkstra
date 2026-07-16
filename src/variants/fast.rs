//! V4 — fast engine (VARIANTS.md): the same algorithm as the V1/V3
//! variants, engineered hard. Labels are packed into a u128 key
//! `(len_bits << 64) | (hops << 32) | curr` with pred kept in a side
//! array (encoded pred+1, 0 = the source's −1); heap entries and DS
//! values carry (key, pred+1) so every comparison the reference makes —
//! including pred tie-breaks and the exact staleness checks — is
//! reproduced verbatim. The DS is (key→value map) + a lazy SoA binary
//! heap where an entry is live iff it equals the map's CURRENT value for
//! its vertex (map-based, NOT d-based: d can drop below the stored value
//! without a re-insert, and the reference still pulls at the stored
//! value). Adjacency is CSR. Claims and the falsification-review record
//! are in VARIANTS.md V4.

use rustc_hash::FxHashMap;

use crate::solver::{choose_params, partition_tree, pow2_sat};

const UNSET: u128 = u128::MAX;

#[inline]
fn pack(len: f64, hops: u32, curr: u32) -> u128 {
    ((len.to_bits() as u128) << 64) | ((hops as u128) << 32) | curr as u128
}

#[inline]
fn len_of(key: u128) -> f64 {
    f64::from_bits((key >> 64) as u64)
}

#[inline]
fn hops_of(key: u128) -> u32 {
    (key >> 32) as u32
}

/// A full label value: packed key + encoded pred (pred+1; 0 = −1).
/// Ordered lexicographically — exactly the reference 4-tuple, because
/// pred is the tuple's last component.
type Val = (u128, u32);

const VAL_INF: Val = (UNSET, u32::MAX);

/// SoA binary min-heap of (key, predp, vertex), ordered by (key, predp) —
/// the reference heap's (label, vertex) order: vertex only ever tied
/// after the full label, and full-label ties make the entries identical.
#[derive(Default)]
struct VHeap {
    k: Vec<u128>,
    p: Vec<u32>,
    v: Vec<u32>,
}

impl VHeap {
    #[inline]
    fn clear(&mut self) {
        self.k.clear();
        self.p.clear();
        self.v.clear();
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.k.is_empty()
    }

    #[inline]
    fn less(&self, a: usize, b: usize) -> bool {
        (self.k[a], self.p[a]) < (self.k[b], self.p[b])
    }

    fn push(&mut self, key: u128, predp: u32, vert: u32) {
        self.k.push(key);
        self.p.push(predp);
        self.v.push(vert);
        let mut i = self.k.len() - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.less(i, parent) {
                self.k.swap(i, parent);
                self.p.swap(i, parent);
                self.v.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    fn pop(&mut self) -> Option<(u128, u32, u32)> {
        if self.k.is_empty() {
            return None;
        }
        let last = self.k.len() - 1;
        self.k.swap(0, last);
        self.p.swap(0, last);
        self.v.swap(0, last);
        let out = (
            self.k.pop().expect("non-empty"),
            self.p.pop().expect("non-empty"),
            self.v.pop().expect("non-empty"),
        );
        let len = self.k.len();
        let mut i = 0usize;
        loop {
            let l = 2 * i + 1;
            if l >= len {
                break;
            }
            let mut c = l;
            let r = l + 1;
            if r < len && self.less(r, l) {
                c = r;
            }
            if self.less(c, i) {
                self.k.swap(i, c);
                self.p.swap(i, c);
                self.v.swap(i, c);
                i = c;
            } else {
                break;
            }
        }
        Some(out)
    }

    fn peek(&self) -> Option<(u128, u32, u32)> {
        if self.k.is_empty() {
            None
        } else {
            Some((self.k[0], self.p[0], self.v[0]))
        }
    }
}

/// SoA lazy min-heap of (key, vertex) — V9's heap: pred dropped from the
/// order (a full key tie is the same vertex; internal duplicate order is
/// unobservable under the settled bit), entries 20 bytes, one u128
/// comparison.
#[derive(Default)]
struct KHeap {
    k: Vec<u128>,
    v: Vec<u32>,
}

impl KHeap {
    fn push(&mut self, key: u128, vert: u32) {
        self.k.push(key);
        self.v.push(vert);
        let mut i = self.k.len() - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.k[i] < self.k[parent] {
                self.k.swap(i, parent);
                self.v.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    fn pop(&mut self) -> Option<(u128, u32)> {
        if self.k.is_empty() {
            return None;
        }
        let last = self.k.len() - 1;
        self.k.swap(0, last);
        self.v.swap(0, last);
        let out = (
            self.k.pop().expect("non-empty"),
            self.v.pop().expect("non-empty"),
        );
        let len = self.k.len();
        let mut i = 0usize;
        loop {
            let l = 2 * i + 1;
            if l >= len {
                break;
            }
            let mut c = l;
            let r = l + 1;
            if r < len && self.k[r] < self.k[l] {
                c = r;
            }
            if self.k[c] < self.k[i] {
                self.k.swap(i, c);
                self.v.swap(i, c);
                i = c;
            } else {
                break;
            }
        }
        Some(out)
    }
}

/// The keyed keep-min structure D: map + lazy heap (VARIANTS.md claim B).
struct FastDs {
    m_cap: u64,
    b: Val,
    map: FxHashMap<u32, Val>,
    heap: VHeap,
}

impl FastDs {
    fn new(m_cap: u64, b: Val) -> Self {
        FastDs {
            m_cap,
            b,
            map: FxHashMap::default(),
            heap: VHeap::default(),
        }
    }

    fn insert(&mut self, vert: u32, val: Val) {
        let push = match self.map.entry(vert) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if val < *e.get() {
                    e.insert(val);
                    true
                } else {
                    false
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(val);
                true
            }
        };
        if push {
            self.heap.push(val.0, val.1, vert);
        }
    }

    fn merge(&mut self, other: &FastDs) {
        for (&vert, &val) in &other.map {
            self.insert(vert, val);
        }
    }

    /// Entry live iff it equals the map's current value for its vertex
    /// (removal on pull makes stale duplicates at an equal value dead).
    fn pull(&mut self) -> (Vec<u32>, Val) {
        let m = usize::try_from(self.m_cap).unwrap_or(usize::MAX);
        let take = self.map.len().min(m);
        let mut keys = Vec::with_capacity(take);
        while keys.len() < take {
            let (k, p, vert) = self.heap.pop().expect("map non-empty => live entry exists");
            if self.map.get(&vert) == Some(&(k, p)) {
                self.map.remove(&vert);
                keys.push(vert);
            }
        }
        let bound = loop {
            match self.heap.peek() {
                None => break self.b,
                Some((k, p, vert)) => {
                    if self.map.get(&vert) == Some(&(k, p)) {
                        break (k, p);
                    }
                    self.heap.pop();
                }
            }
        };
        (keys, bound)
    }

    fn non_empty(&self) -> bool {
        !self.map.is_empty()
    }

    /// Smallest live stored value, if any (V5's return-time cap). Pops
    /// dead entries encountered on the way — pure garbage removal.
    fn peek_min(&mut self) -> Option<Val> {
        loop {
            match self.heap.peek() {
                None => return None,
                Some((k, p, vert)) => {
                    if self.map.get(&vert) == Some(&(k, p)) {
                        return Some((k, p));
                    }
                    self.heap.pop();
                }
            }
        }
    }
}

/// Epoch-stamped FindPivots scratch — the pass-3 pattern of the mainline.
struct Scratch {
    epoch: u32,
    heap: VHeap,
    k_vert: Vec<u32>,
    k_parent_order: Vec<(u32, u32)>,
    k_stamp: Vec<u32>,
    kp_stamp: Vec<u32>,
    kp_slot: Vec<u32>,
    tree_stamp: Vec<u32>,
    tree_id: Vec<u32>,
    q_stamp: Vec<u32>,
    w_stamp: Vec<u32>,
    prior_stamp: Vec<u32>,
    group_stamp: Vec<u32>,
}

impl Scratch {
    fn new(n: usize) -> Self {
        Scratch {
            epoch: 0,
            heap: VHeap::default(),
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

struct Tree {
    edges: Vec<(u32, u32)>,
    root: u32,
}

struct Fast {
    // CSR adjacency, input edge order per vertex.
    indptr: Vec<u32>,
    head: Vec<u32>,
    wt: Vec<f64>,
    t: u64,
    k: u64,
    hybrid_level: u64,
    fixed_base: bool,
    /// VARIANTS.md V8: settled vertices are never re-adopted (relax
    /// rejects), re-frontiered (pulls drop them), or re-settled (W′
    /// excludes them).
    settled_dedup: bool,
    settled: Vec<bool>,
    dk: Vec<u128>,
    dp: Vec<u32>,
    scratch: Scratch,
}

impl Fast {
    #[inline]
    fn d_val(&self, v: u32) -> Val {
        (self.dk[v as usize], self.dp[v as usize])
    }

    /// Relax with the reference's exact `cand ≤ d[v] && cand < B` on the
    /// 4-tuple: key compare first, pred only on key equality.
    #[inline]
    fn relax(&mut self, u: u32, v: u32, w: f64, b: Val) -> bool {
        if self.settled_dedup && self.settled[v as usize] {
            return false; // V8: settled targets are never re-adopted
        }
        let du = self.dk[u as usize];
        debug_assert!(du != UNSET, "relax from an unreached vertex");
        let ck = pack(len_of(du) + w, hops_of(du) + 1, v);
        let cp = u + 1;
        let vi = v as usize;
        let dv = self.dk[vi];
        let le = ck < dv || (ck == dv && cp <= self.dp[vi]);
        if !le {
            return false;
        }
        let lt_b = ck < b.0 || (ck == b.0 && cp < b.1);
        if !lt_b {
            return false;
        }
        self.dk[vi] = ck;
        self.dp[vi] = cp;
        true
    }

    #[allow(clippy::type_complexity)]
    fn find_pivots(&mut self, b: Val, s_list: &[u32]) -> (Vec<Vec<u32>>, Vec<u32>, Vec<u32>) {
        let mut sc = std::mem::replace(&mut self.scratch, Scratch::new(0));
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
            sc.heap.push(self.dk[x as usize], self.dp[x as usize], x);
            sc.k_vert.push(x);
            sc.k_stamp[x as usize] = k_ep;
            let mut merged = false;

            'search: while !sc.heap.is_empty() && (sc.k_vert.len() as u64) < self.k {
                let (lk, lp, u) = sc.heap.pop().expect("heap non-empty");
                if (lk, lp) != self.d_val(u) {
                    continue; // stale lazy-deletion entry (full-label check)
                }
                let (start, end) = (self.indptr[u as usize], self.indptr[u as usize + 1]);
                for e in start..end {
                    let (v, w) = (self.head[e as usize], self.wt[e as usize]);
                    if !self.relax(u, v, w, b) {
                        continue;
                    }
                    let vi = v as usize;
                    if sc.k_stamp[vi] != k_ep {
                        sc.k_stamp[vi] = k_ep;
                        sc.k_vert.push(v);
                    }
                    if sc.kp_stamp[vi] == kp_ep {
                        sc.k_parent_order[sc.kp_slot[vi] as usize] = (u, v);
                    } else {
                        sc.kp_stamp[vi] = kp_ep;
                        sc.kp_slot[vi] = sc.k_parent_order.len() as u32;
                        sc.k_parent_order.push((u, v));
                    }
                    if sc.tree_stamp[vi] == tree_ep {
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
                    sc.heap.push(self.dk[vi], self.dp[vi], v);
                }
            }

            if merged {
                continue;
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
                    }
                }
            }
        }

        let mut p_lists: Vec<Vec<u32>> = Vec::new();
        for tree in &trees {
            for group in partition_tree(&tree.edges, tree.root, self.k as usize) {
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
                    p_lists.push(p_j);
                }
            }
        }
        self.scratch = sc;
        (p_lists, q_list, w_list)
    }

    fn base_case(&mut self, b: Val, s_list: &[u32], cutoff: u64) -> (Val, Vec<u32>, FastDs) {
        let mut ds = FastDs::new(1, b);
        for &x in s_list {
            ds.insert(x, self.d_val(x));
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
            if self.settled_dedup {
                self.settled[u as usize] = true;
            }
            let (start, end) = (self.indptr[u as usize], self.indptr[u as usize + 1]);
            for e in start..end {
                let (v, w) = (self.head[e as usize], self.wt[e as usize]);
                if self.relax(u, v, w, b) {
                    ds.insert(v, self.d_val(v));
                }
            }
        }
        if self.fixed_base {
            // V5: cap B′ at dB[D] so the returned pair honors Lemma 3.1.
            if let Some(minv) = ds.peek_min() {
                bound = bound.min(minv);
            }
        }
        (bound, u_log, ds)
    }

    fn bmssp(&mut self, b: Val, s_list: &[u32], level: u64) -> (Val, Vec<u32>, FastDs) {
        let t = self.t;
        if level <= self.hybrid_level {
            let cutoff = (t * t * t).saturating_mul(pow2_sat(level * t));
            return self.base_case(b, s_list, cutoff);
        }
        let mut ds = FastDs::new(t.saturating_mul(pow2_sat((level - 1) * t)), b);
        let (mut p_lists, _q, w_list) = self.find_pivots(b, s_list);

        let mut piv: Vec<Option<u32>> = Vec::with_capacity(p_lists.len());
        let mut piv_of: FxHashMap<u32, usize> = FxHashMap::default();
        let mut member: FxHashMap<u32, usize> = FxHashMap::default();
        for (j, p_j) in p_lists.iter().enumerate() {
            let mut p = p_j[0];
            for &v in &p_j[1..] {
                if self.d_val(v) < self.d_val(p) {
                    p = v;
                }
            }
            piv.push(Some(p));
            piv_of.insert(p, j);
            for &v in p_j {
                member.insert(v, j);
            }
            ds.insert(p, self.d_val(p));
        }

        let mut b_prime = b;
        for p in piv.iter().flatten() {
            b_prime = b_prime.min(self.d_val(*p));
        }
        let mut u_log: Vec<u32> = Vec::new();
        let mut u_set: FxHashMap<u32, ()> = FxHashMap::default();
        let limit = (t * t * t).saturating_mul(pow2_sat(level * t));
        let mut j_pending: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();

        while (u_log.len() as u64) <= limit && ds.non_empty() {
            let (si_keys, b_i) = ds.pull();
            let mut si: Vec<u32> = si_keys.clone();
            let mut si_set: FxHashMap<u32, ()> = si_keys.iter().map(|&x| (x, ())).collect();
            for &x in &si_keys {
                let Some(&j) = piv_of.get(&x) else { continue };
                for &v in &p_lists[j] {
                    if self.settled_dedup && self.settled[v as usize] {
                        continue; // V8: settled members are not re-frontiered
                    }
                    if self.d_val(v) < b_i && !si_set.contains_key(&v) {
                        si_set.insert(v, ());
                        si.push(v);
                    }
                }
            }
            if self.settled_dedup {
                // V8: pulled keys that are already settled expanded their
                // P_j above but do not seed the child.
                si.retain(|&x| !self.settled[x as usize]);
            }

            let (b_pi, u_i, ds_i) = self.bmssp(b_i, &si, level - 1);
            ds.merge(&ds_i);

            for &u in &u_i {
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
                        piv[j] = None;
                    }
                }
            }

            for &u in &u_i {
                let (start, end) = (self.indptr[u as usize], self.indptr[u as usize + 1]);
                for e in start..end {
                    let (v, w) = (self.head[e as usize], self.wt[e as usize]);
                    if self.relax(u, v, w, b) && self.d_val(v) >= b_i {
                        ds.insert(v, self.d_val(v));
                        if let Some(&j) = member.get(&v) {
                            if !j_pending.contains(&j) {
                                if let Some(pj) = piv[j] {
                                    if self.d_val(pj) > self.d_val(v) {
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
                if p_lists[j].is_empty() {
                    piv[j] = None; // Q8: pivot dies with its emptied set
                    continue;
                }
                let mut p = p_lists[j][0];
                for &v in &p_lists[j][1..] {
                    if self.d_val(v) < self.d_val(p) {
                        p = v;
                    }
                }
                piv[j] = Some(p);
                piv_of.insert(p, j);
                ds.insert(p, self.d_val(p));
            }

            b_prime = b_pi;
            for &u in &u_i {
                u_set.insert(u, ());
            }
            u_log.extend(u_i);
            j_pending.clear();
        }

        for &x in s_list {
            if self.settled_dedup && self.settled[x as usize] {
                continue; // V8
            }
            let dx = self.d_val(x);
            if b_prime <= dx && dx < b {
                ds.insert(x, dx);
            }
        }
        let w_prime: Vec<u32> = w_list
            .iter()
            .copied()
            .filter(|&x| {
                !u_set.contains_key(&x)
                    && self.d_val(x) < b_prime
                    && !(self.settled_dedup && self.settled[x as usize])
            })
            .collect();
        if self.settled_dedup {
            for &x in &w_prime {
                self.settled[x as usize] = true; // V8: W′ settles
            }
        }
        for &u in &w_prime {
            let (start, end) = (self.indptr[u as usize], self.indptr[u as usize + 1]);
            for e in start..end {
                let (v, w) = (self.head[e as usize], self.wt[e as usize]);
                if self.relax(u, v, w, b) && self.d_val(v) >= b_prime {
                    ds.insert(v, self.d_val(v));
                }
            }
        }
        u_log.extend_from_slice(&w_prime);
        (b_prime, u_log, ds)
    }
}

pub struct FastRun {
    pub delta: u64,
    pub t: u64,
    pub k: u64,
    pub l_top: u64,
    pub dist: Vec<f64>,
    /// Settlement events in the top-level log minus distinct vertices —
    /// the Q10 re-settlement count the V5 experiment reports.
    pub dup_settlements: usize,
}

/// V9 (VARIANTS.md): the flat top runner — the L = l_top degenerate
/// member (one base-case call over the whole graph) with the three
/// provably-dead components removed: no keyed map (at top level the
/// stored value always equals the current label, because every accepted
/// relaxation immediately inserts and no parent sweep exists), no
/// per-pull separator (the top call is a full execution and returns
/// B′ = B under both bound semantics; nobody reads the intermediate
/// bounds), and no pred in the heap (a full key tie is the same vertex,
/// whose internal duplicate order is unobservable — the settled bit
/// skips every later pop). What remains is lazy Dijkstra over the
/// packed 4-tuple keys: dk/dp labels + settled bits + a (u128, vertex)
/// min-heap. Distances and the settlement log are bit-identical to
/// `sssp_dedup` at hybrid_level ≥ l_top.
pub fn sssp_flat(n: usize, edges: &[(u32, u32, f64)], source: usize) -> FastRun {
    assert!(
        (n as u64) < u32::MAX as u64,
        "packed labels require n < 2^32 - 1"
    );
    let (delta, t, k, l_top) = choose_params(n, edges.len(), n);

    let mut indptr = vec![0u32; n + 1];
    for &(u, _, _) in edges {
        indptr[u as usize + 1] += 1;
    }
    for i in 0..n {
        indptr[i + 1] += indptr[i];
    }
    let mut head = vec![0u32; edges.len()];
    let mut wt = vec![0f64; edges.len()];
    let mut next = indptr.clone();
    for &(u, v, w) in edges {
        assert!(w >= 0.0 && w.is_finite(), "weights must be finite non-negative");
        let slot = next[u as usize] as usize;
        head[slot] = v;
        wt[slot] = w;
        next[u as usize] += 1;
    }

    let mut dk = vec![UNSET; n];
    let mut dp = vec![u32::MAX; n];
    let mut settled = vec![false; n];
    let mut heap = KHeap::default();

    dk[source] = pack(0.0, 0, source as u32);
    dp[source] = 0; // encoded -1
    heap.push(dk[source], source as u32);

    let mut settle_count = 0usize;
    while let Some((key, u)) = heap.pop() {
        let ui = u as usize;
        // The settled bit subsumes staleness: the first pop of a vertex
        // carries its minimal key (heap order), every later pop is dead.
        if settled[ui] {
            continue;
        }
        debug_assert_eq!(key, dk[ui], "first pop must carry the current label");
        settled[ui] = true;
        settle_count += 1;
        let (start, end) = (indptr[ui], indptr[ui + 1]);
        for e in start..end {
            let (v, w) = (head[e as usize], wt[e as usize]);
            let vi = v as usize;
            if settled[vi] {
                continue; // V8: settled targets are never re-adopted
            }
            let ck = pack(len_of(key) + w, hops_of(key) + 1, v);
            let cp = u + 1;
            let dv = dk[vi];
            // Strict improvement on the full 4-tuple writes and pushes;
            // an exact tie is idempotent and the keyed dedup of the map
            // engine would not push either — parity with sssp_dedup.
            if ck < dv || (ck == dv && cp < dp[vi]) {
                dk[vi] = ck;
                dp[vi] = cp;
                heap.push(ck, v);
            }
        }
    }
    assert!(settle_count <= n, "settled bit guarantees distinct settles");

    let dist = dk
        .iter()
        .map(|&kk| if kk == UNSET { f64::INFINITY } else { len_of(kk) })
        .collect();
    FastRun {
        delta,
        t,
        k,
        l_top,
        dist,
        dup_settlements: 0,
    }
}

/// V4 entry: original graph (V1 semantics), packed-label engine,
/// `hybrid_level` as in V3. Panics loudly if the top-level call is not a
/// full execution (QUESTIONS.md Q10).
pub fn sssp(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> FastRun {
    sssp_full(n, edges, source, hybrid_level, false, false, override_params)
}

/// V5 entry: V4 with the fixed base case (B′ capped at dB[D]).
pub fn sssp_fixed(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> FastRun {
    sssp_full(n, edges, source, hybrid_level, true, false, override_params)
}

/// V8 entry: V4 with the per-vertex settled bit (VARIANTS.md V8) —
/// settled vertices are never re-adopted, re-frontiered, or re-settled.
pub fn sssp_dedup(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    override_params: Option<(u64, u64, u64)>,
) -> FastRun {
    sssp_full(n, edges, source, hybrid_level, false, true, override_params)
}

fn sssp_full(
    n: usize,
    edges: &[(u32, u32, f64)],
    source: usize,
    hybrid_level: u64,
    fixed_base: bool,
    settled_dedup: bool,
    override_params: Option<(u64, u64, u64)>,
) -> FastRun {
    assert!(
        (n as u64) < u32::MAX as u64,
        "packed labels require n < 2^32 - 1"
    );
    let (delta, t, k, l_top) = match override_params {
        None => choose_params(n, edges.len(), n),
        Some((df, tf, kf)) => {
            let log_n = (n.max(2) as f64).log2().max(1.0);
            (df, tf, kf, (log_n / tf as f64).ceil() as u64)
        }
    };

    // CSR in input edge order per vertex (counting sort is stable here).
    let mut indptr = vec![0u32; n + 1];
    for &(u, _, _) in edges {
        indptr[u as usize + 1] += 1;
    }
    for i in 0..n {
        indptr[i + 1] += indptr[i];
    }
    let mut head = vec![0u32; edges.len()];
    let mut wt = vec![0f64; edges.len()];
    let mut next = indptr.clone();
    for &(u, v, w) in edges {
        assert!(w >= 0.0 && w.is_finite(), "weights must be finite non-negative");
        let slot = next[u as usize] as usize;
        head[slot] = v;
        wt[slot] = w;
        next[u as usize] += 1;
    }

    let mut fast = Fast {
        indptr,
        head,
        wt,
        t,
        k,
        hybrid_level,
        fixed_base,
        settled_dedup,
        settled: vec![false; n],
        dk: vec![UNSET; n],
        dp: vec![u32::MAX; n],
        scratch: Scratch::new(n),
    };
    fast.dk[source] = pack(0.0, 0, source as u32);
    fast.dp[source] = 0; // encoded -1

    let (b_final, settle, ds_top) = fast.bmssp(VAL_INF, &[source as u32], l_top);
    assert!(
        b_final == VAL_INF && !ds_top.non_empty(),
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

    let dist = fast
        .dk
        .iter()
        .map(|&k| if k == UNSET { f64::INFINITY } else { len_of(k) })
        .collect();
    FastRun {
        delta,
        t,
        k,
        l_top,
        dist,
        dup_settlements,
    }
}
