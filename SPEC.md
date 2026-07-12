# SPEC — reference semantics for arXiv:2602.07868

This document pins every semantic decision the paper leaves open, so that
the Python reference (`reference/reference.py`) is deterministic and a
future Rust port can be gated bit-exactly against it. Paper references are
to v2 (10 Feb 2026); pseudocode line numbers follow Algorithms 1–5.

Contents: labels and bounds · degree reduction · parameters · data
structure · FindPivots · tree partition · BMSSP · base case · determinism
pins · output readback.

## Labels and bounds (§2.3, Algorithm 1)

- A distance label is the 4-tuple `(length: float, nEdges: int, curr: int,
  pred: int)`, compared lexicographically. `pred = -1` for the source.
  Unset labels compare as `(+inf, +inf, +inf, +inf)` (`LABEL_INF`).
- Upper bounds `B` are the same 4-tuples; the top-level call uses
  `B_TOP = LABEL_INF`.
- `Relax(u, v, w, B)`: candidate `c = (d[u].length + w, d[u].nEdges + 1,
  v, u)`; if `c <= d[v]` and `c < B`, set `d[v] = c` and return True
  (paper's `<=`; on exact tuple equality the store is a no-op but True is
  still returned, which can re-insert `v` into a DS — harmless because DS
  Insert dedups by key).
- Note the candidate is always **strictly greater** than `d[u]` (nEdges
  grows even at w = 0), which is what makes zero-weight cycles from the
  degree reduction non-looping and makes a vertex settled inside one local
  Dijkstra search impossible to improve within that search.

## Degree reduction (§2.1)

Parameterized by δ ≥ 3. Each original vertex v with degree Δv becomes a
zero-weight directed cycle C_v of `ceil(Δv / (δ-2))` vertices (one vertex
if Δv ≤ δ-2; a single vertex carries no self-loop — a cycle of length 1 is
degenerate and omitted). Slots for incident edges are assigned to cycle
vertices in the order the incident edges appear in the input edge list,
`δ-2` per cycle vertex. Edge (u, v, w) becomes `x_uv → x_vu` with weight w
where `x_uv` is u's slot vertex for this edge and `x_vu` is v's.

Cycle vertex ids are assigned in original-vertex order, then slot order
(deterministic). The source s maps to the first cycle vertex of C_s.

## Parameters (§3, Lemma 3.9)

With n₂ = reduced vertex count. All logs base 2. The paper's prescriptions
are asymptotic; the reference clamps them into their own validity ranges
and records the clamps (see QUESTIONS.md Q1):

- `δ = max(3, floor(min(m/n, loglog n) / 4))` where
  `loglog n = max(1.0, log2(max(2.0, log2(max(2, n)))))`.
- `t = max(2, ceil(sqrt(log2(n₂) · loglog(n₂) / δ)))`.
- `k = max(2, ceil(t / log2(t)))`.
- `l_top = ceil(log2(n₂) / t)`, levels l ∈ [0, l_top].
- Block parameter at level l: `M(l) = t · 2^((l-1)·t)` (exact integer);
  base case (l = 0) uses M = 1.
- Loop cutoffs: recursion `|U| ≤ t³·2^(l·t)` (checked at the top of the
  while loop, per Algorithm 3 line 9); base case `|U| ≤ t³`.

## Data structure D (Lemma 3.4)

Observable semantics do not depend on the block internals — only the
complexity does — so the reference implements D as a keyed map:

- `Insert(key, value)`: keep the smaller value per key (values are label
  tuples; distinct keys never carry equal values because `curr` is part of
  the value).
- `Merge(D')`: insert all pairs of D' (the M'-vs-M/3 constraint is
  complexity-only).
- `Pull()`: remove and return the `min(M, |D|)` pairs with smallest
  values, **keys ordered by ascending value**, plus separating bound `x` =
  the smallest remaining value if D is non-empty after the pull, else the
  B that D was constructed with. This choice of x (the tightest valid
  separator) is a pin; any valid separator satisfies the lemma.

## FindPivots (Algorithm 2)

- Iterate x ∈ S in S's list order. Skip x if it already lies in an
  existing F̄ tree of this call (F̄ state is local to one FindPivots call).
- Local search: lazy-deletion binary heap ordered by full label tuple
  (this reproduces Fibonacci-heap extraction order because the tuple order
  is total; stale entries — popped label ≠ current d — are skipped and do
  not count as settlements). K is a vertex set plus a parent-edge map;
  a valid relaxation of (u, v) sets/overwrites v's parent edge.
- On relaxing into a vertex of an existing F̄_j: merge (F̄_j gains K's
  vertices and parent edges plus the contact edge), abandon x entirely
  (lines 17–18 are NOT executed for a merged search).
- Search ends when heap empties or |K| ≥ k (checked before each pop).
  |K| ≥ k → K becomes a new F̄ (root = x); heap empty → K becomes an
  arborescence W_j (root = x, x joins Q). W is accumulated as a list in
  discovery order.
- Partition each F̄ (in creation order) with Algorithm 5, s = k, root =
  the x that created the F̄; global subtree order = F̄ order then report
  order within a F̄.
- `P_j = [x ∈ S list order : x ∈ vertices(F_j), x ∉ Q, x ∉
  vertices(F_j') for any j' < j]`. Empty P_j are dropped (paper does not
  discuss them; see QUESTIONS.md Q2).

## Tree partition (Algorithm 5, Lemma A.1)

Literal recursion, edges treated as undirected. Children of a node are
visited in the order their edges were added to the tree. After each child
returns, if |U| ≥ s the group is reported and U resets to {r}. The final
leftover U merges into the last reported group, or forms the only group
if none was reported. Groups are vertex lists; they may share boundary
vertices (edge-disjointness is what the lemma guarantees).

## BMSSP (Algorithm 3)

Follows the pseudocode literally with these pins:

- Pivots: `p_j = argmin_{x ∈ P_j} d[x]` (unique — tuple order is total).
  B′₀ = min(B, min_j d[p_j]) (plain B if there are no pivots).
- Pulled S_i is processed in pull order (ascending value). The pivot
  expansion (line 13) appends `{v ∈ P_j : d[v] < B_i}` in P_j's stored
  order, skipping vertices already present in S_i.
- Pivot bookkeeping: removing u ∈ U_i from its P_j precedes the relax
  sweep (two separate passes over U_i, as in the pseudocode). If u was
  p_j and P_j is now empty, the pivot dies silently (no J entry). J is
  processed in ascending j.
- Line 24's condition is `Relax(u, v, B)` **and** the candidate ≥ B_i
  (the candidate equals d[v] after a successful relax).
- U is a list — the settlement log; sub-call results and W′ are appended
  in order. U-membership checks use a parallel set.
- After the loop: re-insert `x ∈ S` (S list order) with d[x] ∈ [B′, B);
  then W′ = `[x ∈ W list order : x ∉ U, d[x] < B′]`, relax its out-edges
  with the ≥ B′ variant of line 24's condition, append W′ to U.

## Base case (Algorithm 4)

D = keyed map with M = 1, seeded with S. While D non-empty and |U| ≤ t³:
pull one smallest (B′ = next-smallest value or B), append to U, relax all
out-edges with bound B, inserting valid targets into D. Return (B′, U, D).
Implemented literally — including the suspicious interaction between the
returned B′ and vertices inserted after the final pull (QUESTIONS.md Q3).

## Determinism pins (summary for the future Rust gate)

1. All iteration orders above (S order, pull order, P_j stored order,
   W order, edge order = input adjacency order, subtree order).
2. Lazy-deletion heap with full-tuple ordering (no secondary tie-break
   needed — tuples are totally ordered).
3. Float arithmetic: label length is built by left-to-right addition
   along the path (`d[u].length + w`); no reassociation allowed in ports.
4. No hash-order dependence anywhere: the reference uses insertion-ordered
   dicts and lists only; a port must use order-preserving structures or
   sort where this SPEC says "ascending value".

The differential gate for the Rust port will compare: final labels
(bit-exact, all four components) and the settlement log (the U list of
the top call, in order).

## Output readback

After the top-level call BMSSP(B_TOP, [s₂], l_top) on the reduced graph,
`dist(v) = min over C_v of d[x].length` (all reached cycle vertices of one
C_v carry equal length; unreached vertices report +inf). The reference
returns lengths for original vertices only.
