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
