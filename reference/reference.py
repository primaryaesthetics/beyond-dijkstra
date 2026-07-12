"""Reference implementation of the SSSP algorithm from arXiv:2602.07868.

Semantics are pinned by SPEC.md: this file is the ground truth a future
Rust port will be gated against (bit-exact labels + settlement log), so
clarity and determinism outrank speed everywhere. No hash-order dependence:
only lists and insertion-ordered dicts.

Known deliberate literalisms are tracked in QUESTIONS.md (Q1 clamps,
Q2 empty P_j, Q3 base-case bound).
"""

import heapq
import math
import sys

sys.setrecursionlimit(100_000)

INF = math.inf
LABEL_INF = (INF, INF, INF, INF)


# ---------------------------------------------------------------------------
# Baseline: textbook Dijkstra on the original graph (lengths only).

def dijkstra_lengths(n, edges, s):
    adj = [[] for _ in range(n)]
    for u, v, w in edges:
        adj[u].append((v, w))
    dist = [INF] * n
    dist[s] = 0.0
    heap = [(0.0, s)]
    while heap:
        du, u = heapq.heappop(heap)
        if du > dist[u]:
            continue
        for v, w in adj[u]:
            nd = du + w
            if nd < dist[v]:
                dist[v] = nd
                heapq.heappush(heap, (nd, v))
    return dist


# ---------------------------------------------------------------------------
# Degree reduction (SPEC "Degree reduction", paper section 2.1).

def degree_reduce(n, edges, delta):
    """Returns (n2, adj2, cycle_of) where cycle_of[v] = [reduced ids of C_v].

    Slot capacity per cycle vertex is delta - 2. Reduced ids are assigned in
    original-vertex order, then slot order. Reduced adjacency lists hold the
    zero-weight cycle edges first (vertex order), then cross edges in input
    edge order.
    """
    cap = delta - 2
    deg = [0] * n
    for u, v, _ in edges:
        deg[u] += 1
        deg[v] += 1

    base = [0] * n
    n2 = 0
    cycle_of = []
    for v in range(n):
        base[v] = n2
        c = max(1, -(-deg[v] // cap))  # ceil, at least one representative
        cycle_of.append(list(range(n2, n2 + c)))
        n2 += c

    adj2 = [[] for _ in range(n2)]
    for v in range(n):
        cyc = cycle_of[v]
        if len(cyc) > 1:
            for i, x in enumerate(cyc):
                adj2[x].append((cyc[(i + 1) % len(cyc)], 0.0))

    slot_ctr = [0] * n

    def take_slot(v):
        x = cycle_of[v][slot_ctr[v] // cap]
        slot_ctr[v] += 1
        return x

    for u, v, w in edges:
        xu = take_slot(u)
        xv = take_slot(v)
        adj2[xu].append((xv, w))
    return n2, adj2, cycle_of


# ---------------------------------------------------------------------------
# Parameters (SPEC "Parameters"; clamps per QUESTIONS.md Q1).

def choose_params(n, m, n2):
    def loglog(x):
        return max(1.0, math.log2(max(2.0, math.log2(max(2, x)))))

    ratio = m / n if n > 0 else 0.0
    delta = max(3, int(min(ratio, loglog(n)) / 4))
    log_n2 = max(1.0, math.log2(max(2, n2)))
    t = max(2, math.ceil(math.sqrt(log_n2 * loglog(n2) / delta)))
    k = max(2, math.ceil(t / math.log2(t)))
    l_top = math.ceil(log_n2 / t)
    return delta, t, k, l_top


# ---------------------------------------------------------------------------
# Data structure D (SPEC "Data structure D", Lemma 3.4). Keyed map with
# keep-smaller dedup; Pull returns keys ascending by value plus the tightest
# separating bound. Block internals are complexity-only and not modeled.

class BlockDS:
    __slots__ = ("M", "B", "map")

    def __init__(self, M, B):
        self.M = M
        self.B = B
        self.map = {}

    def insert(self, key, val):
        old = self.map.get(key)
        if old is None or val < old:
            self.map[key] = val

    def merge(self, other):
        for k, v in other.map.items():
            self.insert(k, v)

    def pull(self):
        items = sorted(self.map.items(), key=lambda kv: kv[1])
        take = items[: self.M]
        for key, _ in take:
            del self.map[key]
        bound = items[self.M][1] if len(items) > self.M else self.B
        return [key for key, _ in take], bound

    def non_empty(self):
        return bool(self.map)


# ---------------------------------------------------------------------------
# Tree partition (SPEC "Tree partition", Algorithm 5), s-sized groups.

def partition_tree(edge_list, root, s):
    padj = {}
    for u, v in edge_list:
        padj.setdefault(u, []).append(v)
        padj.setdefault(v, []).append(u)
    groups = []

    def rec(r, parent):
        acc = [r]
        for c in padj.get(r, ()):  # child order = edge insertion order
            if c == parent:
                continue
            acc.extend(rec(c, r))
            if len(acc) >= s:
                groups.append(acc)
                acc = [r]
        return acc

    leftover = rec(root, None)
    if groups:
        last = set(groups[-1])
        groups[-1].extend(v for v in leftover if v not in last)
    else:
        groups.append(leftover)
    return groups


# ---------------------------------------------------------------------------
# The solver: BMSSP recursion on the reduced graph.

class Solver:
    def __init__(self, n2, adj2, t, k):
        self.n2 = n2
        self.adj = adj2
        self.t = t
        self.k = k
        self.d = [LABEL_INF] * n2

    def relax(self, u, v, w, B):
        du = self.d[u]
        cand = (du[0] + w, du[1] + 1, v, u)
        if cand <= self.d[v] and cand < B:
            self.d[v] = cand
            return True
        return False

    # -- FindPivots (SPEC "FindPivots", Algorithm 2) ----------------------

    def find_pivots(self, B, S):
        d = self.d
        tree_of = {}
        trees = []  # each: [vertex list, vertex set, edge list, root]
        Q_list, Q_set = [], set()
        W_list, W_set = [], set()

        for x in S:
            if x in tree_of:
                continue
            heap = [(d[x], x)]
            k_vert = [x]
            k_set = {x}
            k_parent = {}  # v -> (u, v), latest valid tree edge into v
            merged = False
            while heap and len(k_set) < self.k and not merged:
                lab, u = heapq.heappop(heap)
                if lab != d[u]:
                    continue  # stale lazy-deletion entry
                for v, w in self.adj[u]:
                    if not self.relax(u, v, w, B):
                        continue
                    if v not in k_set:
                        k_set.add(v)
                        k_vert.append(v)
                    k_parent[v] = (u, v)
                    j = tree_of.get(v)
                    if j is not None:  # contact: merge K into existing tree
                        tvert, tset, tedges, _ = trees[j]
                        for kv in k_vert:
                            tree_of[kv] = j
                            if kv not in tset:
                                tset.add(kv)
                                tvert.append(kv)
                        tedges.extend(k_parent.values())
                        merged = True
                        break
                    heapq.heappush(heap, (d[v], v))
            if merged:
                continue  # x abandoned, nothing reported
            if len(k_set) >= self.k:
                j = len(trees)
                trees.append([k_vert, k_set, list(k_parent.values()), x])
                for kv in k_vert:
                    tree_of[kv] = j
            else:
                Q_list.append(x)
                Q_set.add(x)
                for kv in k_vert:
                    if kv not in W_set:
                        W_set.add(kv)
                        W_list.append(kv)

        p_lists = []
        prior = set()
        for tvert, _tset, tedges, root in trees:
            for group in partition_tree(tedges, root, self.k):
                gset = set(group)
                p_j = [v for v in S
                       if v in gset and v not in Q_set and v not in prior]
                prior |= gset
                if p_j:  # QUESTIONS.md Q2: empty P_j dropped
                    p_lists.append(p_j)
        return p_lists, Q_list, W_list

    # -- Base case (SPEC "Base case", Algorithm 4) ------------------------

    def base_case(self, B, S):
        d = self.d
        ds = BlockDS(1, B)
        for x in S:
            ds.insert(x, d[x])
        U = []
        bound = B
        while ds.non_empty() and len(U) <= self.t ** 3:
            keys, bound = ds.pull()
            u = keys[0]
            U.append(u)
            for v, w in self.adj[u]:
                if self.relax(u, v, w, B):
                    ds.insert(v, d[v])  # QUESTIONS.md Q3: may undercut bound
        return bound, U, ds

    # -- BMSSP (SPEC "BMSSP", Algorithm 3) --------------------------------

    def bmssp(self, B, S, level):
        if level == 0:
            return self.base_case(B, S)
        d = self.d
        t = self.t
        ds = BlockDS(t * (2 ** ((level - 1) * t)), B)
        p_lists, _Q, W_list = self.find_pivots(B, S)

        piv = []          # j -> current pivot vertex or None (dead)
        piv_of = {}       # current pivot vertex -> j
        member = {}       # vertex -> j for every vertex in some P_j
        for j, p_j in enumerate(p_lists):
            p = min(p_j, key=lambda v: d[v])
            piv.append(p)
            piv_of[p] = j
            for v in p_j:
                member[v] = j
            ds.insert(p, d[p])

        b_prime = min([B] + [d[p] for p in piv])
        U, U_set = [], set()
        limit = t ** 3 * (2 ** (level * t))
        J = set()

        while len(U) <= limit and ds.non_empty():
            si_keys, b_i = ds.pull()
            si = list(si_keys)
            si_set = set(si)
            for x in si_keys:  # pivot expansion (line 13)
                j = piv_of.get(x)
                if j is None:
                    continue
                for v in p_lists[j]:
                    if d[v] < b_i and v not in si_set:
                        si.append(v)
                        si_set.add(v)

            b_pi, u_i, ds_i = self.bmssp(b_i, si, level - 1)
            ds.merge(ds_i)

            for u in u_i:  # pivot bookkeeping pass (lines 16-20)
                j = member.pop(u, None)
                if j is None:
                    continue
                p_lists[j].remove(u)
                if piv[j] == u:
                    del piv_of[u]
                    if p_lists[j]:
                        J.add(j)
                    else:
                        piv[j] = None  # pivot dies with its set

            for u in u_i:  # relax sweep (lines 21-26)
                for v, w in self.adj[u]:
                    if self.relax(u, v, w, B) and d[v] >= b_i:
                        ds.insert(v, d[v])
                        j = member.get(v)
                        if (j is not None and j not in J
                                and piv[j] is not None and d[piv[j]] > d[v]):
                            del piv_of[piv[j]]
                            piv[j] = v
                            piv_of[v] = j

            for j in sorted(J):  # re-selection (lines 27-29)
                if not p_lists[j]:
                    # QUESTIONS.md Q8: P_j emptied after j entered J; the
                    # paper's argmin is over an empty set here. Pivot dies.
                    piv[j] = None
                    continue
                p = min(p_lists[j], key=lambda v: d[v])
                piv[j] = p
                piv_of[p] = j
                ds.insert(p, d[p])

            b_prime = b_pi
            U.extend(u_i)
            U_set.update(u_i)
            J.clear()

        for x in S:  # premature vertices go back to the frontier (line 33)
            if b_prime <= d[x] < B:
                ds.insert(x, d[x])
        w_prime = [x for x in W_list if x not in U_set and d[x] < b_prime]
        for u in w_prime:  # final relax from completed arborescences
            for v, w in self.adj[u]:
                if self.relax(u, v, w, B) and d[v] >= b_prime:
                    ds.insert(v, d[v])
        U.extend(w_prime)
        return b_prime, U, ds


# ---------------------------------------------------------------------------
# Public entry points.

def sssp_instrumented(n, edges, s, override=None):
    """Full run with the differential-gate observables exposed.

    Returns (lengths, labels, settle, params, n2): per-original-vertex
    lengths, the final label array over the reduced graph, the top-level
    call's U list (the settlement log), and (delta, t, k, l_top).

    ``override`` forces (delta, t, k) — the harness's deep-recursion
    regime; l_top always follows the formula.
    """
    if override is None:
        delta, _, _, _ = choose_params(n, len(edges), 0)
        n2, adj2, cycle_of = degree_reduce(n, edges, delta)
        delta, t, k, l_top = choose_params(n, len(edges), n2)
    else:
        delta, t, k = override
        n2, adj2, cycle_of = degree_reduce(n, edges, delta)
        log_n2 = max(1.0, math.log2(max(2, n2)))
        l_top = math.ceil(log_n2 / t)

    solver = Solver(n2, adj2, t, k)
    s2 = cycle_of[s][0]
    solver.d[s2] = (0.0, 0, s2, -1)
    _, settle, _ = solver.bmssp(LABEL_INF, [s2], l_top)

    out = []
    for v in range(n):
        best = min(solver.d[x] for x in cycle_of[v])
        out.append(best[0])
    return out, solver.d, settle, (delta, t, k, l_top), n2


def sssp_lengths(n, edges, s):
    """Shortest-path lengths from s per arXiv:2602.07868 (reference)."""
    return sssp_instrumented(n, edges, s)[0]
