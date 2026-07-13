# Variants — algorithm-level research track

The mainline (faithful port, differential-gated, frozen) stays untouched;
everything here changes the algorithm and answers to a different gate:
**bit-exact distances vs the crate's own Dijkstra** over the full property
matrix plus tie-rich stress at scale (settlement order is NOT pinned).
Every entry follows the class-2 protocol: claim → proof sketch naming the
paper invariants → classification → gate → measurement. The sketch is
written before the code and survives here.

## V1 — no-degree-reduction (run on the original graph)

**Claim.** Running BMSSP (Algorithm 3, with the reference semantics of
SPEC.md) directly on the original input graph — no §2.1 zero-weight-cycle
substitution — computes exact shortest-path lengths for every vertex.

**Classification.** Class 2: distance-preserving, structure-changing.
Settlement log and labels change (different vertex space); distances must
stay exact. Gate: Tier 2.

**Which paper invariants the degree bound actually carries.** Every
appearance of the degree bound δ in the paper (verified against v2, all
occurrences of "degree"):

1. §2.1 — the transform's own definition (3 ≤ δ ≤ m/n).
2. Lemma 3.1's time term O(|U|(l·log t + δt)) — time only; the
   correctness contract (U = Ũ(B′, S) complete, ⟨U, D⟩ frontier) does
   not mention δ.
3. Lemma 3.2's time term O((p+|Q|)k(δ+log k)) and the simplification
   δ ≤ log k — time only; footnote 1 verbatim: "The algorithm also works
   when δ > log k. Setting δ = Θ(min{m/n, log log n}) optimizes the time
   bound, so we require δ ≤ log k to simplify the analysis of time
   complexity."
4. Observation 3.5 / Lemma 3.9 accounting — "each search from x failing
   to reach k vertices is attributed to one of x's incoming edges, so it
   occurs at most δ times" — amortization of *running time* only.
5. "the degree of each vertex in V is at most O(δ). Thus the total
   running time is..." (§3 wrap-up) — time only.
6. **Correction found in review:** Lemma 3.8 ("Correctness of BMSSP,
   focusing on size constraints") DOES use δ in its proof — |D_X| ≤
   |S_X| + δ|U_X| — so "no correctness statement mentions δ" is too
   strong. What that bound feeds, however, is Lemma 3.4's per-insert
   O(t) cost (via M ≥ log(N/M)) — a time claim bundled into a
   correctness-labeled lemma; Lemma 3.4's *functional* behavior
   (keep-min, separator) never reads N. The accurate statement: **no
   statement consumed by the completeness induction mentions δ.**

**Why correctness never needs it.**

- The frontier/completeness machinery (Observation 2.1, the §2.5
  invariant, Lemma 3.6, Lemma 3.7's induction) is stated and proved for
  arbitrary non-negatively weighted digraphs; no step counts neighbors.
- Lemma 3.2's correctness half ("⟨W, ∪P_j⟩ is a frontier") argues: a
  complete u ∈ S either lies in some P_j, or its search stopped with
  < k vertices, in which case every Ũ-vertex whose shortest path visits
  u lies in that subtree and is swept into W. The k-cutoff of the local
  search does the work — unlike DMM+25, where the analogous W bound came
  from k Bellman-Ford rounds over bounded-degree vertices. |W| ≤ k|Q|
  and |P_j| = O(k) fall out of the cutoff and the Θ(k) partition
  (Lemma A.1), both degree-free.
- The size invariants feeding Lemma 3.7/3.8's induction (|S_child| via
  M-sized pulls plus O(k)-sized pivot expansions, the |U| ≤ t³2^lt
  break, p ≤ |U|/k) are enforced by the algorithm's own control flow
  and the partition — none reads a vertex degree.
- Zero-weight edges: the transform *introduces* zero-weight cycles, so
  the label machinery (4-tuple with strictly increasing nEdges, keyed
  dedup) already must handle w = 0 correctly on the reduced graph; the
  original graph is no worse. Q7's nEdges < (working-graph n) bound only
  tightens without the cycle detours.

**What the variant still takes from the parameter recipe.** δ survives
only inside the t-formula (t = ⌈√(log n·loglog n/δ)⌉ with n now the
original vertex count, l_top = ⌈log₂n/t⌉). Without the transform δ has
no structural meaning — it is a free knob of a valid-parameter family
(footnote 3 explicitly invites tuning); V1 keeps the mainline formula
unchanged so the V1-vs-mainline comparison isolates the transform's cost.
Parameter sweeps are a separate class-3 entry.

**Expected win (hypothesis).** On the random m = 4n family the transform
inflates the working graph to n₂ ≈ 8n (measured: n₂ = 8 000 342 for
n = 10⁶); FindPivots — 82.8% of mainline time after pass 3 — then runs
its searches over 8× the vertices with the extra zero-weight cycle hops.
First project's biggest constant win was exactly this variant.

**Falsification review** (independent fresh-context pass over claim +
sketch + paper; verdict HOLDS-BUT-GAP). What it added beyond the sketch:

- Sharper W-bound mechanism: Algorithm 2's `|K| < k` check runs per
  extracted vertex, and one hub scan can push |K| from 1 to n−1 — the
  paper's margin note "Recall δ ≤ k" exists exactly because reduction
  bounds that overshoot. But a W_j is reported only when the heap
  empties with |K| < k still true; overshooting searches report an F̄
  instead. So |W_j| ≤ k−1 unconditionally — the correctness half never
  sees the overshoot. Nothing anywhere needs |K| = Θ(k).
- Lemma A.1's partition survives unbounded fanout: the |U| ≥ s check
  runs after each child subtree, so groups stay in [s, 2s) (final merge
  < 3s) even under a degree-(n−1) root.
- On hubs the raw partition-piece count is Θ(n/k) ≫ |S| and most P_j
  are empty — Q2's "drop empty P_j" fix stops being a corner case and
  becomes load-bearing on every hub (any port looping j = 1..p per the
  paper crashes on the first star).
- The lemma statements do NOT transfer verbatim as standalone
  contracts: there is a 4-vertex instance satisfying Lemma 3.7's
  stated precondition on which its conclusion is false (QUESTIONS.md
  Q9, degree-independent, affects the paper as written — not this
  variant specifically). The property that actually holds is the
  reachable-state closure along the live call stack; consequence for
  testing: drive the top-level entry only — synthetic mid-tree calls
  can "fail" without indicting anything.
- The only genuinely δ-dependent size bound is |D_X| ≤ |S_X| + δ|U_X|
  (Lemma 3.8 proof; also Algorithm 4's "|D| ≤ t³δ ≤ t⁴" comment). On a
  hub |D| reaches Θ(n) at any level and M ≥ log(N/M) is violated —
  time-only, PROVIDED the implementation never pre-sizes or asserts
  those bounds. **Port constraint: V1 (and the mainline) must not
  assert |D| ≤ |S| + δ|U|, |K| ≤ 2k, p ≤ |S|, or base-case |D| ≤ t⁴.**
  The current code is clean (only the |U| cutoffs exist, degree-free).
- Top-level totality re-derived degree-free: t³·2^(l_top·t) ≥ t³·n > n
  ≥ |U| for every n ≥ 1 under the recipe, so the top call can only
  exit with D empty — a full execution settling every reachable vertex.

Targeted tests it prescribed (all in `tests/variants_correctness.rs`):
star with hub = source and hub ≠ source at sizes straddling t³ and
t³·2^t; hub inside a child call so the parent D absorbs Θ(n) inserts at
M = t; hub reached by a failed (< k) search with a competing shorter
path outside S; zero-weight star.

**Result.** Tier-2 gate green: 3 generator regimes (360 graphs incl.
forced deep recursion), tie-rich 1.02e6-edge integer stress, star/hub
family (hub = source, spoke source, zero-weight star, hub-behind-
competing-paths at 100 and 10⁴ leaves), all bit-exact vs the crate's
Dijkstra. Raw records `benchmarks/results/bench_fast_v1_*.txt`
(mainline column = pass-3 epoch, same binary):

| input | dijkstra | mainline | V1 no-transform | V1 vs mainline |
|---|---|---|---|---|
| random 1e5 | 0.009 s | 184× | **5.78×** | 32× faster |
| random 1e6 | 0.193 s | 191× | **5.85×** | 33× faster |
| USA-road-d.NY | 0.010 s | 40× | **9.26×** | 4.3× faster |

The degree reduction was the dominant constant cost of the entire
implementation — the same verdict as the first project, now at a larger
magnitude (8× vertex blowup + zero-weight cycle hops feeding the
FindPivots volume that pass-3 profiling showed at 82.8%). V1's 5.8× is
the new working floor for further variants (parameter sweeps next;
still no crossover with Dijkstra in sight, consistent with the
ASSESSMENT.md hypothesis — the bounds cross near n ≈ 2^1000).

V1 phase profile (`profile_phases_v1.txt`): relax sweep 53.0% @1e6 /
42.6% NY, find_pivots down to 21.5% / 12.2%, remainder spread. No
pathological hotspot — the variant spends its time on O(m)-per-level
relaxation work plus DS inserts, i.e. on the algorithm itself.

## V2 — parameter sweep on V1 (class 3)

**Claim.** None beyond validity: any t ≥ 2, k ≥ 2 with l_top = ⌈log₂n/t⌉
is a valid instance of the schema (footnote 3 calls the prescribed
values "best" in order-analysis terms only). The knob-sweep rule
applies: show where the prescription sits relative to the real optimum.

**Gate.** Tier 2 — every sweep cell cross-checks distances bit-exactly
against the crate's Dijkstra inside the sweep binary itself.

**Result.** Grid t ∈ {2..12} × k ∈ {2..12}, median of 3 after warmup,
δ fixed at 3. Raw records `benchmarks/results/sweep_params_v1_{1e6,NY}.txt`:

| input | prescription (t=6, k=3) | best cell | spread |
|---|---|---|---|
| random 1e6 | 5.82× | 5.29× (t=12, k=6, l_top=2) | prescription within 10% |
| USA-road-d.NY | 8.65× | 6.97× (t=10, k=4, l_top=2) | prescription ~24% off |

The structure of both tables is the same monotone story: performance
improves as t grows and the recursion gets SHALLOWER, flattening once
l_top hits 2; k is nearly irrelevant (except k = 2 with t = 2, which is
catastrophic — 33× @1e6 at l_top = 10). The optimum sits at the edge of
the grid where the algorithm degenerates toward "one level over base
cases": the recursion never pays for itself at storable sizes. That is
the same root-degeneration signature the first project ended on, and it
is the honest headline of this sweep — parameter tuning buys 10–24%,
not a different verdict. Hybrid-oracle-style variants (replace the
bottom of the recursion with plain Dijkstra) are the natural next probe
of exactly this signature.

## V3 — hybrid oracle: base case promoted to levels ≤ L (on V1)

**Claim.** For any level l, replacing the recursive BMSSP call by the
paper's own base case (Algorithm 4: bounded lazy Dijkstra over the
keyed M = 1 structure) run with cutoff |U| ≤ t³·2^(l·t) — instead of
the l = 0 cutoff t³ — preserves the Lemma 3.1 contract the parent
consumes, hence final distances. The hybrid knob L ∈ [0, l_top] replaces
all levels ≤ L; L = 0 is the paper's algorithm, L = l_top is bounded
tuple-label Dijkstra over the whole graph.

**Classification.** Class 2 (structure-changing: recursion levels are
removed; distances must stay exact). Gate: Tier 2.

**Proof sketch.**

1. Algorithm 4 IS the level-0 instance of the family: its cutoff t³
   equals t³·2^(l·t) at l = 0, and the while-condition has the same
   shape as Algorithm 3's |U| ≤ t³·2^(l·t) check. The paper's base-case
   argument (settling in non-decreasing 4-tuple order from the seeds;
   on cutoff, B′ = the next-smallest stored value separates settled
   from pending; returned D = relaxed-but-unsettled keys) nowhere uses
   the numeric value of the cutoff — it only needs cutoff ≥ 1 so the
   loop makes progress. The value t³ enters Lemma 3.9's charging and
   the |U| = O(t³) size bound — the parent consumes those only in time
   accounting; its control flow (extend U, check own limit, propagate
   B′) is size-agnostic.
2. The parent's Lemma 3.1 precondition on the child (⟨∅, S_i⟩ frontier
   for Ũ(B_i, S_i)) is exactly what the base case requires; nothing in
   the substitution changes how S_i is formed.
3. Q3's known contract wrinkle (D may hold values < B′ after the final
   relax sweep) exists at l = 0 already and self-heals in the parent
   pull loop for the same reason at any level; the volume grows with
   the cutoff but the mechanism is unchanged. (Watched by the Tier-2
   tie-rich and zero-weight tests.)
4. Termination/totality: unchanged — the top-level cutoff still exceeds
   n, so the top call remains a full execution.

**Falsification review** (independent fresh-context pass; verdict
HOLDS-BUT-GAP, no break). What it established beyond the sketch:

- The S re-insert asymmetry (Algorithm 3 line 33 has no Algorithm 4
  counterpart) is benign in the safe direction: the base case seeds ALL
  of S into D and keys leave only via Pull-into-U, so D ⊇ S \ U holds
  structurally — the exact property Lemma 3.7's induction consumes; a
  Q3-class member (d[x] < B′, x ∉ U) is dropped by Algorithm 3's guard
  but retained by Algorithm 4. Strictly more conservative.
- The parent sweep's `d[v] ≥ B_i` insert guard stays exhaustive over a
  huge base-case child because pulled 4-tuples strictly increase within
  a base case (nEdges grows even at w = 0), so any sub-B_i parent
  relaxation is an exact-equality no-op on a key the child already
  covered.
- L = l_top is provable outright: the cutoff never fires (|U| ≤ n),
  no re-settlement is possible, no reliance on Q3 self-healing — the
  degenerate member is the safest.
- THE GAP is mainline-inherited, now named as QUESTIONS.md **Q10**:
  top-level totality is unproved because re-settlements count against
  the |U| cutoff (empirical margin ~5× at the worst sawtooth point,
  248 adversarial cells clean). Closed operationally: both entry
  points now assert B′ = ∞ ∧ D = ∅ at top level (silent-failure mode
  made loud, zero cost), plus a zero-heavy tie-rich stress family at
  the minimum-slack size in the Tier-2 gate.

**Result.** Tier-2 gate green (hybrid levels 1/2/total across all
regimes incl. forced deep recursion and the 1e6-edge tie-rich stress;
mainline differential gate re-verified — the hybrid_level=0 default is
bit-neutral). L-sweep, median of 3, every cell distance-checked; raw
records `benchmarks/results/sweep_hybrid_{1e6,NY}.txt`:

| input | params | L=0 (V1) | interior L | L=l_top (pure base case) |
|---|---|---|---|---|
| random 1e6 | prescription (l_top=4) | 5.50× | 6.19 / 5.24 / 4.45× | **3.76×** |
| random 1e6 | t=12, k=6 (l_top=2) | 5.18× | 4.45× | **3.83×** |
| USA-road-d.NY | prescription (l_top=4) | 9.39× | 8.04 / 6.60 / 5.50× | **4.49×** |
| USA-road-d.NY | t=12, k=6 (l_top=2) | 7.84× | 5.64× | **4.54×** |

Monotone to the end on both inputs and both parameter settings: every
level of recursion removed makes it faster, and the family's optimum is
its degenerate member — Algorithm 4 alone over the whole graph, i.e. a
bounded Dijkstra on 4-tuple labels over the keyed M = 1 structure. The
residual 3.8–4.5× against the crate's Dijkstra is the price of the
label machinery itself (32-byte lexicographic tuples + ordered-map
operations vs a flat f64 4-ary heap), not of any algorithmic idea. This
completes the verdict chain: 191× (faithful mainline) → 5.8× (drop the
degree reduction) → 3.8× (drop the recursion) → what remains is
Dijkstra with more expensive bookkeeping. No configuration of this
implementation family beats Dijkstra at storable sizes — consistent
with the ASSESSMENT.md hypothesis (bounds cross near n ≈ 2^1000).

## V4 — fast engine: packed u128 labels, lazy SoA heaps, CSR (on V1/V3)

Epoch note: V1–V3 numbers above are their own recorded epoch and stay
authoritative for the verdict chain; V4 attacks the label-machinery
residual named by V3 and is measured as a NEW epoch. The frozen
mainline is untouched.

**Claim A (packed order).** The 4-tuple lexicographic order is realized
exactly by a single u128 key `(len.to_bits() << 64) | (hops << 32) |
curr` plus a pred side-array consulted only on key equality:

1. IEEE-754: for non-negative, non-NaN doubles, the total order of bit
   patterns (as unsigned) equals the numeric order; lengths are finite
   sums of non-negative weights, so `len.to_bits()` is order-exact.
   (-0.0 cannot arise: no negative weights, and 0.0 + 0.0 = +0.0.)
2. hops < n ≤ 2³² and curr < n ≤ 2³² (asserted loudly at entry), so the
   packed fields never overlap; lexicographic (len, hops, curr) compare
   equals the u128 unsigned compare.
3. pred participates in the 4-tuple order only when (len, hops, curr)
   are all equal — i.e. between two labels of the SAME vertex. Cross-
   vertex comparisons (heap order, DS value order, bound separators
   between distinct vertices) are always decided by curr at the latest.
   So a side-array `pred+1` (0 encoding the source's −1, which is
   smaller than every real pred, matching −1 < any id) consulted only
   on u128 equality reproduces every comparison the reference makes:
   relax's cand-vs-d[v] (curr always equal there), and the rare
   bound-vs-candidate tie where the bound is a label of the same
   vertex.

**Claim B (lazy-heap D).** The keyed keep-min structure D is realized
by (key→value hash map) + (SoA min-heap of (u128 value, vertex) with
lazy deletion), where an entry is live iff its value equals the map's
CURRENT stored value for that vertex:

1. The map is unchanged from the gated pass-1 representation — dedup
   semantics (keep smallest inserted value per key) are its property.
2. The heap holds one entry per insert; superseded entries (a smaller
   value was inserted later) go stale because the map moved on. Pull =
   pop live entries ascending — the map's value set in ascending order,
   which is exactly the reference's sorted-pull.
3. **The trap this design must and does avoid:** liveness is checked
   against the map's stored value, NOT against d[v]. d[v] can drop
   below the stored value without a DS insert (the parent sweep's
   `d[v] ≥ B_i` guard skips the insert) — the reference still pulls v
   at the stored value, so a d-based staleness check would silently
   drop v from a child's frontier and break completeness. Map-based
   liveness reproduces the reference exactly.
4. Merge = fold of inserts (push on improve-or-new), order-independent
   per key as before. Separator = the next live minimum, peeked by
   popping dead entries first — the same "smallest remaining stored
   value, else construction B" the reference returns.

**Other engineering (no semantic content):** CSR adjacency (flat
indptr/head/weight arrays) instead of Vec<Vec>; SoA 4-ary heaps for the
FindPivots searches keyed by u128 (stale check `key != dk[u]` is the
reference's `lab != d[u]` verbatim); the epoch-stamp scratch of
mainline pass 3 carried over.

**Classification.** Class 2 relative to the reference observables
(settlement order may shift only where pred used to order same-key
duplicates inside heaps — unobservable in Tier 2); distances must stay
bit-exact vs Dijkstra. Gate: Tier 2, full matrix (all generator
regimes, 1e6-edge tie-rich, star family, Q10 minimum-slack stress),
plus the top-level totality assert.

**Falsification review** (independent fresh-context pass; done). Claim
A: HOLDS-AS-ARGUED. Claim B: HOLDS-AS-ARGUED. What the review added:

- The hops < 2³² packing bound needs (and has) a real argument: every
  stored label's pred chain is a simple path — a revisit would store a
  lexicographically larger label over a smaller one, which relax's
  `cand ≤ d[v]` forbids and exact equality cannot rescue (hops differ).
  Hence hops ≤ n − 1.
- Nit: the weight guard `w ≥ 0.0` admits −0.0 (IEEE `-0.0 ≥ 0.0` is
  true). Provably harmless: no label length can become −0.0 (source is
  +0.0; x + (−0.0) = x for x > 0 and +0.0 + (−0.0) = +0.0), so no
  stored key ever carries the −0.0 bit pattern and bit order remains
  value order. Recorded rather than "fixed" — normalizing would change
  nothing observable.
- Claim B's load-bearing invariant, verified against the code: a map
  write always leaves a not-yet-popped heap entry carrying the written
  value, destroyed only by the live pop that also removes the map key;
  the separator loop pops only dead entries. Both engines pass only
  vertex ids out of Pull, so the child re-reads current d identically.
  Merge's hash-order iteration is provably unobservable (single
  occurrence per vertex; acceptance depends only on the target's entry).

**Result.** Tier-2 gate green across the full matrix (all generator
regimes at L ∈ {0, 1, 99} incl. forced deep recursion, 1e6-edge
tie-rich at L ∈ {0, 2, 99}, star family, Q10 minimum-slack stress).
Same-process medians, raw records `benchmarks/results/bench_fast_v4_*`:

| input | V1 engine (same run) | fast L=0 | fast L=l_top |
|---|---|---|---|
| random 1e5 | 7.18× | 5.21× | **2.61×** |
| random 1e6 | 5.81× | 5.27× | **2.75×** |
| USA-road-d.NY | 8.96× | 7.16× | **2.46×** |

The packed-label engine cuts the full-recursion algorithm by 9–27% and
the label-machinery residual (V3's headline) from 3.8–4.5× to
**2.5–2.8×**: more than a third of what V3 attributed to "the price of
4-tuple labels" was representation, not principle. The verdict itself
does not move — even the best engineered member of the family stays
~2.5× behind the crate's Dijkstra at every storable size.

## V5 — fixed base case: cap B′ so the Lemma 3.1 contract actually holds

This one is a proposed correction to the paper's math, not an
engineering variant. Q3 recorded the defect: Algorithm 4 returns the B′
of its final Pull, but the relaxations performed after that pull can
insert labels BELOW B′ (zero/tiny weights do it systematically), so the
returned pair violates both dB[D] ≥ B′ and U = Ũ(B′, S); Q10 recorded
the systemic cost (unproved top-level totality via re-settlements).

**The fix.** Return B′⁺ = min(B′_pull, dB[D_final]) — one extra
min-peek of D at return time, O(1) amortized.

**Claim.** With B′⁺, Algorithm 4 satisfies Lemma 3.1's contract
exactly: U = Ũ(B′⁺, S) is complete, and dB[D] ≥ B′⁺.

**Proof sketch.**

1. dB[D] ≥ B′⁺ by construction (B′⁺ is capped at dB[D]).
2. Every settled label is strictly below B′⁺. Settled labels are
   non-decreasing along pulls (each pull takes the global minimum of
   D's stored values, and every insert exceeds the label of the vertex
   whose relaxation produced it — the 4-tuple grows strictly, hops
   increments even at w = 0). The final D's values each strictly exceed
   the last settled label: leftovers were ≥ B′_pull > label(u_last) by
   the pull invariant, and post-final-relax inserts strictly exceed
   label(u_last) by tuple growth. Hence label(u) ≤ label(u_last) <
   dB[D] and label(u) < B′_pull for every u ∈ U, so label(u) < B′⁺.
3. Completeness of U up to B′⁺ (as repaired by the falsification
   review — the first draft's inequality ran the wrong way): suppose
   x ∉ U with dis-label(x) < B′⁺ and x's shortest path visits S. The
   4-tuple order makes the shortest path canonical (pred is a label
   component, so dis-label prefixes are optimal). Walk the canonical
   path from its last COMPLETE S-vertex (the frontier precondition
   supplies one); let y be the first vertex not in U. Its predecessor
   p on the path is settled with d[p] = dis-label(p), so relax(p, y)
   offered cand = prefix-label(y) ≤ dis-label(x) < B′⁺. Either the
   offer was accepted — then y entered D at a stored value ≤ cand
   (keep-min, and the always-insert-on-accept makes "modified during
   the call ⟹ present in D" airtight) — or it was rejected because
   d[y] was already ≤ cand; but every d value is the label of a real
   walk, and no walk-label of y is below its canonical dis-label, so
   a rejection forces d[y] = dis-label(y) and y had already been
   inserted when that value was set. Either way y sits in D at a value
   < B′⁺ ≤ dB[D] — impossible — unless it was pulled, i.e. y ∈ U.
   Contradiction.
4. Full executions are unaffected: D = ∅ ⇒ the cap is vacuous and
   B′⁺ = B as before. Better: V5 restores the full-execution dichotomy
   outright — the paper's Algorithm 4 can return B′ = B with D ≠ ∅
   (last pull empties D, post-pull relaxes refill it, cutoff fires),
   violating Lemma 3.1's "full ⇒ D = ∅"; with the cap, B′⁺ = B ⟺
   D = ∅ in both directions.

Consequence for Q10: inside base cases the Q3 escape channel closes —
B′ never advances past an unsettled label, so a vertex settles at most
once per base case for the Dijkstra reason. Whether Algorithm 3's OWN
return path still leaks (its line-33 re-insert and W′ sweep are guarded
by d ≥ B′, but merged child structures are now capped) is measured, not
claimed: the experiment reports duplicate settlement counts in the
top-level log for V1 vs V5 on the Q10 stress family.

**Classification.** Class 2 (changes the returned bound — observable
order/recursion shape changes; distances must stay exact). Gate:
Tier 2 full matrix.

**Falsification review** (done; verdict HOLDS-BUT-GAP, gap = the
original step-3 sketch, repaired above per its findings). Highlights
beyond the repairs: leftovers replaced by post-final-pull keep-min
values stay strictly above the last settled label (the only relaxer
after the last pull is u_last, and its candidates strictly exceed its
own label), so step 2 survives replacement; unpulled seeds cannot sit
below the last settled label (global-min pulls); and the priority
attack — a SMALLER child B′ hurting the parent — found nothing,
because Lemma 3.7's proof consumes B′ only through the contract
clauses, never as "the last pull's separator". Sharper still: under
the broken original, dB[D ∪ P] ≥ B′ was false after merges, so the
wider W′'s completeness proof never held — the vertices V5 defers are
exactly the ones the original was not entitled to settle. The review's
scope note: the same analysis supports restoring Algorithm 3's
contract transitively by induction (its own return path preserves
dB[D] ≥ B′ once child bounds are valid) — likely provable in prose,
queued for the write-up rather than another experiment.

**Result.** Tier-2 gate green (both engines, all regimes incl. Q10
stress). The experiment (`benchmarks/results/dup_settlements_v5.txt`,
`examples/dup_settlements.rs`) produced a finding better than the
expected one:

| input | n | paper dups | V5 dups | V5 cap fires |
|---|---|---|---|---|
| zero-heavy min-slack | 4 094 | 5 | 5 | 0 |
| zero-heavy min-slack | 16 382 | 608 | 608 | 0 |
| tie-rich 0..3 | 170 000 | 5 074 | 5 074 | 0 |
| zero-heavy large | 500 000 | 149 | 149 | 0 |

The cap NEVER fires: the base-case B′ undercut (Q3's channel, which
Q10 presumed to drive re-settlements) does not materialize at return
time on any adversarial input tried — while duplicates persist
unchanged. The re-settlement channel is Algorithm 3's own, since
traced end-to-end (QUESTIONS.md Q11, CONFIRMED;
`benchmarks/results/q11_trace.txt`): a stale entry in an outer D —
left behind when the vertex's label improved and settled in a sibling
branch — is pulled into a child frontier, exact-tie `≤`-re-acceptances
keep the completed vertex structurally alive through FindPivots, and
the W′ boundary append (whose u_set filter is call-local) emits the
second U event. It contradicts Lemma 3.8's disjoint-union accounting
as stated. V5 therefore stands as a provable contract restoration with
zero observed runtime effect — the missing re-settlement lemma the
paper needs is about stale-entry re-frontierings in Algorithm 3, not
about the base case.

## Algorithm 3's contract, transitively: what V5 does and does not restore

The write-up the V5 review's scope note asked for. Throughout, dB[D]
means the minimum over D's STORED values (the values the data structure
holds; current labels of stored keys can drift below them — that drift
is exactly Q11's stale-entry phenomenon and is accounted separately).

**Proposition (bound validity propagates).** Assume every child call
returns dB[D_child] ≥ B′_child — V5 gives this for base cases; take it
as the induction hypothesis for recursive children. Then Algorithm 3's
own return satisfies dB[D] ≥ B′, EXCEPT through one channel: a line-29
re-selection insert in the final iteration storing a value below B′_f
(QUESTIONS.md Q12).

*Proof.* Enumerate what D can hold at return.
1. Leftovers never pulled: every pull removes the globally smallest
   stored values, and the last pull's separating bound B_f lower-bounds
   the remainder; B′ = B′_f ≤ B_f, so leftovers ≥ B′. This covers
   low-valued inserts from EARLIER iterations too: a stored value below
   B_f cannot survive to the end — some later pull consumes it.
2. Merged child structures: the final iteration's D_f has
   dB[D_f] ≥ B′_f by the hypothesis; earlier iterations' merges fall
   under item 1.
3. Line-24 sweep inserts: gated by candidate ≥ B_i; in the final
   iteration B_f ≥ B′_f, earlier iterations under item 1.
4. Line 31–33 and 36–37 post-loop inserts: gated by d ≥ B′ directly.
5. Line-29 re-selection inserts: UNGUARDED — the only hole, live only
   in the final iteration (earlier leaks are consumed by later pulls),
   and reachable per Q12's two channels. ∎

**Corollary (full executions).** A full execution's return always
satisfies dB[D] ≥ B′: at loop exit D was empty, so only the gated
post-loop inserts (item 4) populate the returned D. No appeal to
line 29 is needed. Combined with V5 at the leaves, the bound clause of
Lemma 3.7 holds at every level of a recursion whose loop always drains
D — in particular along the top-level call's own spine under the
paper's set semantics for U (QUESTIONS.md Q13: the top-level cutoff
cannot fire, so the top call is always a full execution).

**What is still missing for partial executions is closing Q12 — and
no local return-time repair closes it.** The first draft of this
section proposed capping Algorithm 3's returned B′ at dB[D] the way V5
caps Algorithm 4's; working out the sketch killed the idea, and the
failure is itself a finding. Suppose the Q12 state occurs: the final
iteration's re-selection stored X < B′_f while some label L with
X < L < B′_f was already settled (by the final child or into W′).
Then the three clauses of Lemma 3.7 are JOINTLY unsatisfiable at this
return, whatever B′ the call reports: any B′ > X violates
B′ ≤ dB[D]; any B′ ≤ X < L violates max dis(U) < B′. Gating the
line-29 insert instead of capping fails in the same corner from the
completeness side: the un-inserted pivot pj has dis(pj) ≤ d[pj] < B′,
and if its shortest path visits S it belongs to Ũ(B′, S), which
U = Ũ(B′, S) then requires the call to have settled — it didn't.
Unlike Algorithm 4's Q3 defect (repaired by V5's cap with all clauses
intact), the Q12 state admits no repair at the return boundary; a
coherent fix must prevent the state upstream — the prose reading of
line 23 (relax only candidates in [Bi, B)) closes the sweep channel
but not the child-interior one — or weaken the contract statement to
carve out final-iteration premature entries. Both options are
statement-level mathematics, recorded in QUESTIONS.md Q12; nothing to
implement until one is proved. Whether the state is reachable at all
remains unobserved (the verify-mode counters stayed zero on the
adversarial family).

**What no bound cap restores.** The duplicate U events (Q11) are
untouched by V5 — measured bit-identical — and would be untouched by
V6: the stale-entry re-frontier channel operates entirely below the
bound clauses (the trace's v5 re-settled with a CORRECT label; no
returned bound was ever violated in that run — the cap never fired).
So the transitive restoration upgrades Q10 only partially: the bound
clause of Lemma 3.7 becomes provable (V5 + V6 + full-execution
corollary), top-level totality is unconditional under the paper's own
U semantics (Q13), but Lemma 3.8's disjoint-union accounting — and
with it the paper's time analysis on tie-rich inputs — still lacks the
re-settlement bound. That bound is the genuinely open mathematics; the
concrete failure states to bound are stale-entry pulls and the pivot
re-inserts they enable (Q11's trace).
