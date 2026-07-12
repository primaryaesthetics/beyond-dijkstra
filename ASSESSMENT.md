# Assessment: arXiv:2602.07868 (v2, 10 Feb 2026)

Duan, Mao, Shu, Yin — "A Faster Directed Single-Source Shortest Path
Algorithm". Deterministic directed SSSP in O(m√(log n) + √(mn·log n·log log n))
comparison-addition time; O(m√(log n·log log n)) for sparse graphs. Improves
the O(m·log^(2/3) n) sorting-barrier result (DMM+25) that logtwothirds
implements. No implementation or benchmarks exist as of 2026-07-12.

Snapshot in this repo: `paper-2602.07868.pdf` (+ extracted `paper_text.txt`).

## Verdict: GO

The paper is 22 pages with all five algorithms given in explicit pseudocode
and both proofs self-contained (partition and data structure in the
appendix). Structurally it is a refinement of the same BMSSP recursion we
already implemented — roughly half the work is a re-instantiation of
existing logtwothirds infrastructure. Estimated volume: Python reference
~1.5× the first one (more bookkeeping), Rust port comparable to
bmssp.rs + block_queue.rs.

## Component inventory

1. **Degree reduction** (§2.1) — zero-weight cycle substitution with a
   *parameterized* degree bound δ (not the fixed constant-degree transform
   of DMM+25). O(m) edges, O(m/δ) vertices. Only needed for m ≤ n·log n;
   above that Dijkstra is already optimal and the paper says so.
2. **4-tuple tie-breaking** (§2.3, Algorithm 1) — labels are
   ⟨length, nEdges, curr, pred⟩ compared lexicographically. This is the
   authors' answer to the tie ambiguity class we hit in the first paper
   (Remark 3.4 there): `curr` makes labels of distinct vertices never
   exactly tie.
3. **FindPivots via local Dijkstra forests** (§3.1, Algorithm 2) — replaces
   k-round Bellman-Ford: bounded local Dijkstra from each frontier vertex,
   subtree merging on contact, Frederickson-style tree partition into
   Θ(k)-sized edge-disjoint subtrees (Algorithm 5, Appendix A.1); pivot =
   min-label vertex per subtree. This is the paper's core new idea
   (O(log k) instead of O(k) per vertex).
4. **Block BST** (Lemma 3.4, Appendix A.2) — self-balancing BST of Θ(M)
   blocks, unsorted inside blocks; Insert O(log(N/M)) amortized, Merge/Pull
   linear, keyed dedup built into Insert. Simpler than DMM+25's block
   queue. Base case uses a plain BST with M = 1.
5. **BMSSP main loop** (§3.3, Algorithm 3) — same skeleton as DMM+25
   (levels l, bound B, partial/full executions) plus new pivot
   bookkeeping: per-subtree pivot p_j, re-selection set J, the implicit
   b̂P_j argument (Lemma 3.7). This bookkeeping is the fiddliest part.
6. Parameters: k = ⌈t/log t⌉, t ≈ √(log n·log log n/δ),
   δ = ¼·min{m/n, log log n}, l ∈ [0, ⌈(log n)/t⌉], M = t·2^((l−1)t).

## Correctness vs. time-analysis obligations (license list for variants)

Correctness-carrying: Lemma 3.1 contract, Lemma 3.2 (FindPivots frontier),
Lemma 3.6 (frontier split), Lemma 3.7 (frontier/completeness induction —
the heart), Lemma 3.8 (size constraints feed 3.7's induction), Lemma A.1
(partition validity), Lemma A.2 (DS contract), Observation 2.1, the §2.5
framework invariant.

Time-analysis-only (legal to relax in variants): the δ ≤ log k constraint —
the paper's own footnote 1 says the algorithm works without it; Observation
3.5 and Lemma 3.9 (accounting only); the specific t, k, δ, M formulas
("best" is order-analysis only, footnote 3 invites tuning); plausibly the
degree reduction itself — no correctness lemma visibly depends on the
bounded degree (to be re-verified line-by-line during SPEC; this was
exactly the no-transform finding of the first project).

## Practical-parameter reality check (hypothesis to test, not a claim)

At n = 10^6, m = 4n (our standard input): log₂n ≈ 20, log₂log₂n ≈ 4.3, so
the prescribed δ = ¼·min{4, 4.3} = 1 — below the δ ≥ 3 the degree
reduction requires (§2.1), and above the δ ≤ log k the analysis assumes
(k = 3, log₂k ≈ 1.6). For any sparse graph with m/n < 12 the prescribed
optimum is outside the valid parameter range at every size; an
implementation must clamp, and the recursion is then as overhead-dominated
as the first paper's (t ≈ 5, ~4 levels, M = t·2^((l−1)t)). Expected
verdict: same honest story — no crossover at storable sizes. The two
bounds themselves cross only near n ≈ 2^1000.

## Audit seeds (to become QUESTIONS.md entries during SPEC)

1. δ range inconsistency above — is δ = ¼·min{…} meant to be clamped to
   [3, …]? Nothing in the paper says so.
2. Algorithm 2 merging (`F̄_j ← F̄_j ∪ K` on contact): is the merged F̄_j
   still a directed tree (partition needs a tree), given K's frontier
   stopped mid-expansion?
3. Relax keeps `≤` on the 4-tuple; keyed dedup in the DS (Insert keeps
   smaller value per key) is what prevents the duplicate-entry cascade we
   found in the first paper. Any implementation that "optimizes away" the
   keyed dedup reintroduces that bug class — pin this with a tie-rich test
   from day one.
4. Base case (Algorithm 4): pull is M = 1 so B′ is the last pulled key —
   check off-by-one on the |U| ≤ t³ break interaction with returned B′.
5. W′ = {x ∈ W \ U : d[x] < B′} is asserted complete (Lemma 3.7 proof) —
   verify the argument when Q-rooted arborescences overlap F̄ trees
   (Remark 3.3 admits overlap).

## Plan (pipeline stages, as proven on logtwothirds)

1. SPEC.md — restate the algorithm precisely, resolve audit seeds, freeze
   the reference semantics (tie tuple, pull order, dedup) with pinned
   determinism.
2. Python reference implementation, property-tested against Dijkstra.
3. Rust faithful port + differential gate (port harness from logtwothirds).
4. Benchmarks vs. the crate Dijkstra (port bench_fast; reuse
   USA-road-d.NY), tie-rich stress from day one.
5. Variants track: no-degree-reduction first (the first project's biggest
   constant win), then parameter sweeps (footnote 3 explicitly invites
   this), hybrid Dijkstra oracle if the same root-degeneration appears.
6. Write-up: v0.1.0 here, second note, update letter/JOSS story.
